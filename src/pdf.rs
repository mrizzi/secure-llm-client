use crate::error::CliError;
use std::{path::Path, process::Command};
use tokio::task;

const DOCLING_COMMAND: &str = "docling";

/// Output format for extracted content
#[derive(Debug, Clone, PartialEq)]
pub enum ContentFormat {
    Markdown,
    PlainText,
}

// Note: Quality assessment removed - Docling is the only extractor and we have no way
// to reliably assess transformation quality. Users should rely on:
// - format field (Markdown vs PlainText)
// - warnings field (pipeline info, any issues detected)
// - actual content inspection

/// Extracted PDF content
#[derive(Debug, Clone)]
pub struct PdfContent {
    /// Extracted text content
    pub text: String,

    /// Tool used for extraction (currently always "docling")
    pub extractor_used: &'static str,

    /// Output format (Markdown or PlainText)
    pub format: ContentFormat,

    /// Warnings or notes about extraction (e.g., pipeline info, issues detected)
    pub warnings: Vec<String>,

    /// File size in bytes (if available)
    pub file_size_bytes: Option<u64>,
}

/// Check if Docling CLI is available
pub fn is_docling_available() -> bool {
    Command::new(DOCLING_COMMAND)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Extract markdown from PDF using Docling CLI
async fn extract_with_docling(path: &Path) -> Result<PdfContent, CliError> {
    let path_buf = path.to_path_buf();

    task::spawn_blocking(move || {
        // Create unique temporary output directory
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let unique_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!(
            "docling_output_{}_{}",
            std::process::id(),
            unique_id
        ));
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            CliError::PdfProcessingFailed(format!("Failed to create temp directory: {e}"))
        })?;

        // Run Docling CLI (note: --output is a directory, and --to uses 'md' not 'markdown')
        // Pipeline can be configured via DOCLING_PIPELINE env var (standard, vlm, legacy, asr)
        let pipeline = std::env::var("DOCLING_PIPELINE").unwrap_or_else(|_| "standard".to_string());

        let mut cmd = Command::new(DOCLING_COMMAND);
        cmd.arg(&path_buf)
            .arg("--to")
            .arg("md")
            .arg("--pipeline")
            .arg(&pipeline)
            .arg("--output")
            .arg(&temp_dir);

        log::debug!("Running Docling with pipeline: {pipeline}");

        let output = cmd
            .output()
            .map_err(|e| CliError::PdfProcessingFailed(format!("Failed to run Docling: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(CliError::PdfProcessingFailed(format!(
                "Docling conversion failed.\nStderr: {stderr}\nStdout: {stdout}"
            )));
        }

        // Docling saves output with same base name as input PDF
        let input_filename = path_buf
            .file_stem()
            .ok_or_else(|| CliError::PdfProcessingFailed("Invalid PDF filename".to_string()))?;
        let output_path = temp_dir.join(format!("{}.md", input_filename.to_string_lossy()));

        // Read the generated markdown file
        let markdown_text = std::fs::read_to_string(&output_path).map_err(|e| {
            CliError::PdfProcessingFailed(format!("Failed to read Docling output: {e}"))
        })?;

        // Clean up temporary directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Get file size
        let file_size_bytes = std::fs::metadata(&path_buf).ok().map(|m| m.len());

        let warnings = vec![format!("Docling pipeline: {pipeline}")];

        Ok(PdfContent {
            text: markdown_text,
            extractor_used: "docling",
            format: ContentFormat::Markdown,
            warnings,
            file_size_bytes,
        })
    })
    .await
    .map_err(|e| CliError::PdfProcessingFailed(format!("Task join error: {e}")))?
}

/// Extract text from PDF using Docling CLI
///
/// Docling is REQUIRED for PDF processing. This function will fail if docling is not installed.
///
/// **Why no fallbacks?**
/// - pdfium-render: Requires external C library, produces plain text with poor quality
/// - pdf-extract: Produces plain text with spacing issues (e.g., "i n l i n e   c o d e")
/// - Both fallbacks produce unusable output for LLM-based document processing
///
/// **Installation:**
/// ```bash
/// pip install docling
/// ```
///
/// **Returns:**
/// - `Ok(PdfContent)` - Extracted markdown content with structure preserved
/// - `Err(CliError)` - If docling is not installed or extraction fails
pub async fn extract_text_from_pdf(path: &Path) -> Result<PdfContent, CliError> {
    // Check if Docling is available
    if !is_docling_available() {
        return Err(CliError::PdfProcessingFailed(format!(
            "PDF processing requires Docling CLI.\n\n\
                 Docling command '{DOCLING_COMMAND}' is not available in PATH.\n\n\
                 Install with:\n  pip install docling\n\n\
                 After installation, ensure the docling binary is in your PATH:\n  \
                 export PATH=\"$HOME/.local/bin:$PATH\"  # Linux/macOS\n  \
                 export PATH=\"$HOME/Library/Python/3.x/bin:$PATH\"  # macOS with user install\n\n\
                 Why required? Docling provides AI-powered PDF→Markdown conversion with:\n\
                 • Structure preservation (headings, tables, lists, code blocks)\n\
                 • Markdown format (crucial for LLM evaluation quality)\n\
                 • 97.9% table extraction accuracy"
        )));
    }

    // Extract with Docling
    match extract_with_docling(path).await {
        Ok(content) => {
            log::info!("PDF extracted successfully with Docling (markdown format)");
            Ok(content)
        }
        Err(e) => {
            log::error!("Docling extraction failed: {e}");
            Err(e)
        }
    }
}

/// Convert PDF content to markdown (optional enhancement)
pub fn to_markdown(content: &PdfContent) -> String {
    // Basic cleanup: normalize whitespace, add headers
    let mut markdown = String::from("# Extracted PDF Content\n\n");

    // Metadata
    markdown.push_str(&format!("**Characters**: {}\n", content.text.len()));
    markdown.push_str(&format!(
        "**Words**: {}\n",
        content.text.split_whitespace().count()
    ));
    markdown.push_str(&format!("**Extractor**: {}\n", content.extractor_used));
    markdown.push_str(&format!("**Format**: {:?}\n", content.format));

    if let Some(size) = content.file_size_bytes {
        markdown.push_str(&format!(
            "**File Size**: {} bytes ({:.2} KB)\n",
            size,
            size as f64 / 1024.0
        ));
    }

    if !content.warnings.is_empty() {
        markdown.push_str("\n**Warnings**:\n");
        for warning in &content.warnings {
            markdown.push_str(&format!("- {warning}\n"));
        }
    }

    markdown.push_str("\n---\n\n");
    markdown.push_str(&content.text);
    markdown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_markdown() {
        let content = PdfContent {
            text: "Test content".to_string(),
            extractor_used: "test",
            format: ContentFormat::PlainText,
            warnings: vec!["Test warning".to_string()],
            file_size_bytes: Some(1024),
        };

        let markdown = to_markdown(&content);
        assert!(markdown.contains("# Extracted PDF Content"));
        assert!(markdown.contains("**Characters**: 12")); // "Test content" = 12 chars
        assert!(markdown.contains("**Words**: 2")); // "Test content" = 2 words
        assert!(markdown.contains("**Extractor**: test"));
        assert!(markdown.contains("**File Size**: 1024 bytes"));
        assert!(markdown.contains("**Warnings**:"));
        assert!(markdown.contains("- Test warning"));
        assert!(markdown.contains("Test content"));
    }

    #[test]
    fn test_docling_available() {
        // This test just checks the function doesn't panic
        let _ = is_docling_available();
    }
}
