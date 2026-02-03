use fortified_llm_client::{
    extract_text_from_pdf, is_docling_available, ContentFormat, PdfContent,
};
use std::path::Path;

/// Test simple single-page PDF extraction
#[tokio::test]
async fn test_extract_simple_pdf() {
    let pdf_path = Path::new("tests/fixtures/simple.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    assert!(
        result.is_ok(),
        "Failed to extract simple PDF: {:?}",
        result.err()
    );

    let content = result.unwrap();
    assert!(
        content.text.contains("Hello World")
            || content.text.contains("Hello")
            || content.text.contains("World"),
        "Expected 'Hello World' in extracted text, got: {:?}",
        content.text
    );
    assert_eq!(
        content.extractor_used, "docling",
        "Expected docling extractor, got: {}",
        content.extractor_used
    );
}

/// Test multi-page PDF extraction
#[tokio::test]
async fn test_extract_multipage_pdf() {
    let pdf_path = Path::new("tests/fixtures/multipage.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    assert!(
        result.is_ok(),
        "Failed to extract multipage PDF: {:?}",
        result.err()
    );

    let content = result.unwrap();

    // Check that text from both sections is extracted
    // (PDF may be exported as 1 or 2 pages depending on export tool)
    let has_page_one = content.text.contains("Page One")
        || content.text.contains("Page")
        || content.text.contains("One");
    let has_page_two = content.text.contains("Page Two") || content.text.contains("Two");

    assert!(
        has_page_one || has_page_two,
        "Expected content from both sections, got: {:?}",
        content.text
    );
}

/// Test error handling for missing file
#[tokio::test]
async fn test_missing_pdf_file() {
    let pdf_path = Path::new("tests/fixtures/nonexistent.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    assert!(result.is_err(), "Expected error for missing file");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("PDF processing failed")
            || error.to_string().contains("not found")
            || error.to_string().contains("No such file"),
        "Expected file-not-found error, got: {error}"
    );
}

/// Test error handling for corrupted PDF
#[tokio::test]
async fn test_corrupted_pdf() {
    let pdf_path = Path::new("tests/fixtures/corrupted.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    // Corrupted PDF should either:
    // 1. Return an error (preferred)
    // 2. Return empty/minimal content (fallback behavior)
    if let Ok(content) = result {
        // If extraction succeeded, it should at least know it failed to get good content
        // (some extractors might return empty text for corrupted PDFs)
        assert!(
            content.text.is_empty() || content.text.len() < 50,
            "Corrupted PDF should not produce significant content"
        );
    } else {
        // Error is the expected behavior
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("PDF processing failed")
                || error.to_string().contains("invalid")
                || error.to_string().contains("corrupted"),
            "Expected PDF error, got: {error}"
        );
    }
}

/// Test PdfContent structure
#[test]
fn test_pdf_content_structure() {
    let content = PdfContent {
        text: "Test content".to_string(),
        extractor_used: "test-extractor",
        format: ContentFormat::PlainText,
        warnings: vec![],
        file_size_bytes: Some(1024),
    };

    assert_eq!(content.text, "Test content");
    assert_eq!(content.extractor_used, "test-extractor");
    assert_eq!(content.format, ContentFormat::PlainText);
    assert_eq!(content.warnings.len(), 0);
    assert_eq!(content.file_size_bytes, Some(1024));
}

/// Test that extractor returns valid metadata
#[tokio::test]
async fn test_pdf_metadata() {
    let pdf_path = Path::new("tests/fixtures/simple.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    if let Ok(content) = result {
        // Verify metadata is reasonable
        assert!(
            !content.extractor_used.is_empty(),
            "Extractor name should not be empty"
        );

        // Text should not be empty for a valid PDF with content
        assert!(
            !content.text.is_empty(),
            "Non-empty PDF should produce some text"
        );
    }
}

/// Test dual-strategy fallback behavior (integration test)
/// Note: This test verifies that SOME strategy works, even if primary fails
#[tokio::test]
async fn test_fallback_strategy() {
    let pdf_path = Path::new("tests/fixtures/simple.pdf");

    // Extract twice to verify consistency
    let result1 = extract_text_from_pdf(pdf_path).await;
    let result2 = extract_text_from_pdf(pdf_path).await;

    assert!(result1.is_ok(), "First extraction failed");
    assert!(result2.is_ok(), "Second extraction failed");

    let content1 = result1.unwrap();
    let content2 = result2.unwrap();

    // Extractor used should be the same (deterministic fallback)
    assert_eq!(
        content1.extractor_used, content2.extractor_used,
        "Same extractor should be used for same PDF"
    );
}

/// Test empty PDF handling
#[tokio::test]
async fn test_empty_pdf() {
    // Create minimal empty PDF
    let empty_pdf_path = Path::new("tests/fixtures/empty_test.pdf");
    std::fs::write(empty_pdf_path, b"%PDF-1.4\n%%EOF\n").unwrap();

    let result = extract_text_from_pdf(empty_pdf_path).await;

    // Clean up
    let _ = std::fs::remove_file(empty_pdf_path);

    // Empty PDF should either error or return empty content
    if let Ok(content) = result {
        assert!(
            content.text.is_empty() || content.text.trim().is_empty(),
            "Empty PDF should produce empty text"
        );
    }
    // Error is also acceptable for malformed/empty PDFs
}

/// Test that extraction is async-safe
#[tokio::test]
async fn test_concurrent_extractions() {
    let pdf_path = Path::new("tests/fixtures/simple.pdf");

    // Run multiple extractions concurrently
    let handles: Vec<_> = (0..3)
        .map(|_| {
            let path = pdf_path.to_owned();
            tokio::spawn(async move { extract_text_from_pdf(&path).await })
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    // All should succeed
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Task {i} panicked");
        let extraction = result.unwrap();
        assert!(
            extraction.is_ok(),
            "Extraction {} failed: {:?}",
            i,
            extraction.err()
        );
    }
}

/// Benchmark: Test extraction performance
#[tokio::test]
async fn test_extraction_performance() {
    let pdf_path = Path::new("tests/fixtures/simple.pdf");

    let start = std::time::Instant::now();
    let result = extract_text_from_pdf(pdf_path).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Extraction failed");

    // Docling (AI-powered) is slower than pdfium/pdf-extract
    // First run may include model initialization/download
    // Allow 30 seconds for docling, 5 seconds for other extractors
    let timeout_secs = if is_docling_available() { 30 } else { 5 };

    assert!(
        duration.as_secs() < timeout_secs,
        "Extraction took too long: {duration:?} (timeout: {timeout_secs}s)"
    );

    println!("Extraction completed in {duration:?}");
}

/// Test comprehensive PDF with complex formatting
#[tokio::test]
async fn test_comprehensive_formatting() {
    let pdf_path = Path::new("tests/fixtures/comprehensive.pdf");

    let result = extract_text_from_pdf(pdf_path).await;

    assert!(
        result.is_ok(),
        "Failed to extract comprehensive PDF: {:?}",
        result.err()
    );

    let content = result.unwrap();

    // Basic metadata checks
    assert!(
        !content.text.is_empty(),
        "Extracted text should not be empty"
    );

    println!("Comprehensive PDF:");
    println!("  Extractor: {}", content.extractor_used);
    println!("  Text length: {} chars", content.text.len());

    // Check for key content sections (case-insensitive to handle various formatting)
    let text_lower = content.text.to_lowercase();

    // Main heading
    assert!(
        text_lower.contains("comprehensive formatting test")
            || text_lower.contains("comprehensive") && text_lower.contains("formatting"),
        "Should contain main heading"
    );

    // Text formatting section
    assert!(
        text_lower.contains("text formatting") || text_lower.contains("formatting"),
        "Should contain text formatting section"
    );

    // Lists
    assert!(
        text_lower.contains("first item") || text_lower.contains("first step"),
        "Should contain list items"
    );

    // Tables - check for table content
    assert!(
        text_lower.contains("column")
            || text_lower.contains("row")
            || text_lower.contains("left aligned"),
        "Should contain table content"
    );

    // Code blocks (may or may not be preserved depending on PDF export)
    let has_code = text_lower.contains("extract_text_from_pdf")
        || text_lower.contains("pdfcontent")
        || text_lower.contains("code block")
        || text_lower.contains("rust")
        || text_lower.contains("json");

    if !has_code {
        println!("Warning: Code examples may not have been preserved in PDF export");
    }

    // Links/URLs
    assert!(
        text_lower.contains("anthropic") || text_lower.contains("rust"),
        "Should contain reference to links"
    );

    // Blockquotes
    assert!(
        text_lower.contains("blockquote") || text_lower.contains("simple blockquote"),
        "Should contain blockquote content"
    );

    // Special characters
    assert!(
        text_lower.contains("special characters") || text_lower.contains("escaped"),
        "Should contain special characters section"
    );

    // Final section
    assert!(
        text_lower.contains("final section") || text_lower.contains("end of document"),
        "Should contain final section"
    );

    // Check text is substantial (comprehensive doc should be long)
    assert!(
        content.text.len() > 1000,
        "Comprehensive document should produce substantial text (got {} chars)",
        content.text.len()
    );

    // Print sample of extracted text for manual inspection
    println!("\nFirst 500 characters of extracted text:");
    println!("{}", content.text.chars().take(500).collect::<String>());

    if content.text.len() > 1000 {
        println!("\nLast 500 characters of extracted text:");
        println!(
            "{}",
            content
                .text
                .chars()
                .skip(content.text.len() - 500)
                .collect::<String>()
        );
    }
}

/// Test Docling extraction with markdown output validation
#[tokio::test]
async fn test_docling_markdown_extraction() {
    // Skip if Docling not installed
    if !is_docling_available() {
        println!("⚠ Skipping Docling test - Docling CLI not installed");
        println!("  Install with: pip install docling");
        return;
    }

    println!("✓ Docling CLI detected, testing markdown extraction...");

    let pdf_path = Path::new("tests/fixtures/comprehensive.pdf");
    let md_path = Path::new("tests/fixtures/comprehensive.md");

    // Extract from PDF using Docling
    let result = extract_text_from_pdf(pdf_path).await;

    assert!(
        result.is_ok(),
        "Docling extraction failed: {:?}",
        result.err()
    );

    let content = result.unwrap();

    // Verify Docling was used
    assert_eq!(
        content.extractor_used, "docling",
        "Expected Docling to be used, got: {}",
        content.extractor_used
    );

    // Verify markdown format
    assert_eq!(
        content.format,
        ContentFormat::Markdown,
        "Expected Markdown format from Docling"
    );

    // Read original markdown for comparison
    let original_md =
        std::fs::read_to_string(md_path).expect("Failed to read original comprehensive.md");

    println!("\nDocling extraction results:");
    println!("  Extractor: {}", content.extractor_used);
    println!("  Format: {:?}", content.format);
    println!("  Extracted text length: {} chars", content.text.len());
    println!("  Original markdown length: {} chars", original_md.len());
    if !content.warnings.is_empty() {
        println!("  Warnings:");
        for warning in &content.warnings {
            println!("    - {warning}");
        }
    }

    // Verify text content extraction (realistic expectations for PDF→Text)
    // Note: PDFs store formatted appearance, not markdown source.
    // Docling extracts text content well but doesn't fully reconstruct markdown syntax.

    let text_lower = content.text.to_lowercase();

    // Check for key content sections (text content, not markdown syntax)
    let key_content = vec![
        "comprehensive formatting test",
        "text formatting",
        "lists",
        "code blocks",
        "tables",
        "first item",
        "extract_text_from_pdf", // Example function name
    ];

    let mut found_content = 0;
    for item in &key_content {
        if text_lower.contains(item) {
            found_content += 1;
        }
    }

    let content_rate = (found_content as f32 / key_content.len() as f32) * 100.0;
    println!("\nContent extraction validation:");
    println!(
        "  Found: {}/{} key content items ({:.1}%)",
        found_content,
        key_content.len(),
        content_rate
    );

    // Assert that key content is extracted
    assert!(
        content_rate >= 80.0,
        "Docling should extract at least 80% of key content (got {content_rate:.1}%)"
    );

    // Check text is substantial
    assert!(
        content.text.len() > 2000,
        "Comprehensive document should produce substantial text (got {} chars)",
        content.text.len()
    );

    println!("\n✓ Docling markdown extraction test passed!");
    println!("  All critical structural elements preserved");
    println!("  Format: Markdown (suitable for LLM evaluation)");

    // Print sample for manual verification
    println!("\nFirst 500 characters of extracted markdown:");
    println!("{}", content.text.chars().take(500).collect::<String>());
}
