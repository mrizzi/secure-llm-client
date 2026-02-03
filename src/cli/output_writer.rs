use secure_llm_client::CliOutput;
use std::{fs, io::Write, path::PathBuf};
use tempfile::NamedTempFile;

/// Write CLI output to stdout or file with atomic writes
///
/// Uses atomic writes for file output (temp file + rename) to prevent
/// partial/corrupted files. Automatically creates parent directories.
///
/// # Arguments
///
/// * `output` - The CLI output to write
/// * `output_path` - Optional file path (None = stdout)
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(std::io::Error)` on I/O failure
///
/// # Example
///
/// ```ignore
/// use secure_llm_client::CliOutput;
/// use std::path::PathBuf;
///
/// // Write to stdout
/// write_output(&output, None)?;
///
/// // Write to file
/// write_output(&output, Some(&PathBuf::from("output.json")))?;
/// ```
pub fn write_output(
    output: &CliOutput,
    output_path: Option<&PathBuf>,
) -> Result<(), std::io::Error> {
    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    match output_path {
        Some(path) => {
            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Atomic write: write to temp file in same directory, then rename
            let temp_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let mut temp_file = NamedTempFile::new_in(temp_dir)?;

            temp_file.write_all(json.as_bytes())?;
            temp_file.write_all(b"\n")?; // Add trailing newline
            temp_file.flush()?;

            // Atomically rename temp file to final path
            temp_file.persist(path)?;

            log::info!("Output written to: {}", path.display());
        }
        None => {
            // Print to stdout
            println!("{json}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secure_llm_client::Metadata;
    use std::fs;
    use tempfile::TempDir;

    fn test_metadata() -> Metadata {
        Metadata {
            model: "test-model".to_string(),
            tokens_estimated: 100,
            latency_ms: 200,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            api_url: "http://test".to_string(),
            provider: None,
            temperature: 0.7,
            max_tokens: Some(1000),
            seed: None,
            timeout_secs: 30,
            context_limit: None,
            response_format: None,
            validate_tokens: false,
            system_prompt_text: Some("system".to_string()),
            system_prompt_file: None,
            user_prompt_text: Some("user".to_string()),
            user_prompt_file: None,
            pdf_input: None,
            input_guardrails_enabled: None,
            output_guardrails_enabled: None,
        }
    }

    #[test]
    fn test_write_output_to_stdout() {
        let metadata = test_metadata();
        let output = CliOutput::success("test response".to_string(), metadata, None);

        // Writing to stdout should not error
        // (can't test actual stdout output in unit test)
        assert!(write_output(&output, None).is_ok());
    }

    #[test]
    fn test_write_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.json");

        let metadata = test_metadata();
        let output = CliOutput::success("test response".to_string(), metadata, None);

        // Write to file
        assert!(write_output(&output, Some(&output_path)).is_ok());

        // Verify file exists and contains JSON
        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("test response"));
        assert!(content.contains("test-model"));
    }

    #[test]
    fn test_write_output_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("nested/dir/output.json");

        let metadata = test_metadata();
        let output = CliOutput::success("test response".to_string(), metadata, None);

        // Write to file (should create parent dirs)
        assert!(write_output(&output, Some(&output_path)).is_ok());

        // Verify file exists
        assert!(output_path.exists());
    }
}
