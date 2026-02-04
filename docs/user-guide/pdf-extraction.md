---
layout: default
title: PDF Extraction
parent: User Guide
nav_order: 4
---

# PDF Extraction

Extract text from PDF files and analyze them with LLMs.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Overview

Fortified LLM Client can extract text from PDF files using the external **Docling CLI** tool, then send the extracted content to an LLM for analysis, summarization, or question-answering.

**Key features**:
- Automatic text extraction from PDFs
- File size validation for resource protection
- Direct integration into LLM prompts
- Supports complex PDFs (multi-column, tables, images with text)

## Prerequisites

### Install Docling CLI

PDF extraction requires Docling to be installed and available in your PATH.

**Install via pip**:
```bash
pip install docling
```

**Verify installation**:
```bash
docling --version
```

{: .note }
> Without Docling, PDF extraction will fail with an error. All other features work without it.

## Basic Usage

### CLI

Use the `--pdf-file` flag to provide a PDF instead of text input:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file document.pdf \
  --system-text "Summarize the key points"
```

**How it works**:
1. Docling extracts text from `document.pdf`
2. Extracted text replaces the user prompt
3. LLM receives: system prompt + extracted text
4. Response contains summary/analysis

{: .warning }
> `--pdf-file` conflicts with `--user-text` and `--user-file`. Use only one input source.

### Library

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        system_prompt: Some("Summarize the key points from this document.".to_string()),
        user_prompt: String::new(),  // Will be replaced by PDF content
        pdf_input: Some("document.pdf".to_string()),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    println!("Summary: {}", result.content);

    Ok(())
}
```

## Use Cases

### Use Case 1: Document Summarization

Extract main points from a research paper:

```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --pdf-file research-paper.pdf \
  --system-text "Summarize this research paper in 3-5 bullet points" \
  --output summary.json
```

### Use Case 2: Question Answering

Ask specific questions about a PDF:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file contract.pdf \
  --system-text "Answer this question based on the contract: What are the termination conditions?"
```

### Use Case 3: Data Extraction

Extract structured data from forms or invoices:

```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --pdf-file invoice.pdf \
  --system-text "Extract: invoice number, date, total amount, vendor name" \
  --response-format json-schema \
  --response-format-schema schemas/invoice.json
```

**Schema** (`schemas/invoice.json`):
```json
{
  "type": "object",
  "properties": {
    "invoice_number": {"type": "string"},
    "date": {"type": "string"},
    "total_amount": {"type": "number"},
    "vendor_name": {"type": "string"}
  },
  "required": ["invoice_number", "date", "total_amount", "vendor_name"]
}
```

### Use Case 4: Translation

Translate PDF content to another language:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file document-french.pdf \
  --system-text "Translate this French document to English"
```

### Use Case 5: Content Analysis

Analyze sentiment or themes:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file customer-feedback.pdf \
  --system-text "Analyze the sentiment and identify main themes"
```

## Advanced Configuration

### With Token Validation

Prevent exceeding context limits for large PDFs:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file large-document.pdf \
  --system-text "Summarize" \
  --validate-tokens \
  --context-limit 8192
```

If the PDF text exceeds the limit, the request fails before calling the API.

### With Guardrails

Validate extracted content before sending to LLM:

`config.toml`:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.input]
type = "patterns"
max_length_bytes = 5242880  # 5MB limit

[guardrails.input.patterns]
detect_pii = true  # Check for sensitive data in PDF
```

Usage:
```bash
fortified-llm-client -c config.toml \
  --pdf-file sensitive-document.pdf \
  --system-text "Summarize (redact PII)"
```

### Batch Processing (Library)

Process multiple PDFs programmatically:

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};
use std::path::Path;

async fn batch_summarize(pdf_files: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    for pdf in pdf_files {
        let config = EvaluationConfig {
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            model: "llama3".to_string(),
            system_prompt: Some("Summarize this document in 2-3 sentences.".to_string()),
            user_prompt: String::new(),
            pdf_input: Some(pdf.to_string()),
            ..Default::default()
        };

        match evaluate(config).await {
            Ok(result) => {
                println!("File: {}", pdf);
                println!("Summary: {}\n", result.content);
            }
            Err(e) => {
                eprintln!("Failed to process {}: {}", pdf, e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdfs = vec!["doc1.pdf", "doc2.pdf", "doc3.pdf"];
    batch_summarize(pdfs).await?;
    Ok(())
}
```

## How Docling Works

Fortified LLM Client invokes the Docling CLI as an external process:

```bash
docling <pdf-file> --output <temp-dir> --format markdown
```

**Output format**: Markdown (preserves structure, headings, lists, tables)

**Extraction process**:
1. Validate PDF file size (see [Security Limits](#security-limits))
2. Create temporary directory
3. Run `docling` command
4. Read extracted Markdown text
5. Use as user prompt (replaces `user_prompt` field)
6. Clean up temporary files

## Security Limits

### Maximum PDF Size

**Default limit**: `52428800` bytes (50MB)

**Location**: `src/constants.rs` (`pdf_limits::MAX_PDF_SIZE_BYTES`)

**Why**: Prevents resource exhaustion from extremely large files.

**Error when exceeded**:
```json
{
  "status": "error",
  "error": {
    "code": "PdfError",
    "message": "PDF file size (60MB) exceeds maximum allowed size (50MB)"
  }
}
```

**Override** (requires recompiling):

Edit `src/constants.rs`:
```rust
pub mod pdf_limits {
    /// Maximum PDF file size in bytes (100MB)
    pub const MAX_PDF_SIZE_BYTES: u64 = 104_857_600;
}
```

Then rebuild:
```bash
cargo build --release
```

### Token Limits

Large PDFs may generate text exceeding model context limits. Use `--validate-tokens`:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file huge-book.pdf \
  --validate-tokens \
  --context-limit 8192
```

Fails early if extracted text + system prompt + response buffer > 8192 tokens.

## Troubleshooting

### Error: "docling not found"

**Cause**: Docling CLI not installed or not in PATH.

**Fix**:
```bash
pip install docling
```

Verify:
```bash
which docling
docling --version
```

### Error: "PDF file size exceeds maximum"

**Cause**: PDF larger than 50MB (default limit).

**Fix**: Split the PDF or recompile with higher limit (see [Security Limits](#security-limits)).

### Error: "Failed to extract text from PDF"

**Possible causes**:
- Corrupted PDF file
- Password-protected PDF
- Scanned images without OCR

**Fix**: Ensure PDF is valid and not password-protected. For scanned PDFs, run OCR first.

### Error: "Token limit exceeded"

**Cause**: Extracted text too long for model's context window.

**Fixes**:
1. Use a model with larger context (e.g., GPT-4 Turbo 128K)
2. Split the PDF into smaller sections
3. Preprocess text to extract only relevant parts

### Slow Extraction

**Cause**: Complex PDFs with many images/graphics.

**Fix**: Increase timeout:
```bash
fortified-llm-client \
  --pdf-file complex-report.pdf \
  --system-text "Summarize" \
  --timeout 600  # 10 minutes
```

## Limitations

### 1. Text-Only Extraction

Docling extracts text, not images or graphics. For visual content, consider:
- Image-to-text OCR preprocessing
- Vision-capable LLMs (GPT-4V, LLaVA) - not yet supported

### 2. Formatting Preservation

While Docling preserves structure (headings, lists, tables), complex layouts may lose formatting. Best effort conversion to Markdown.

### 3. Performance

Large PDFs (>10MB) take longer to extract. Consider:
- Preprocessing/splitting large files
- Caching extracted text if analyzing the same PDF multiple times

### 4. External Dependency

Requires Docling installation. If portability is critical, consider alternative approaches (embed PDF parsing library).

## Best Practices

### 1. Validate Before Extraction

Check file size before processing:

```rust
use std::fs;

fn validate_pdf(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    if size > 52_428_800 {  // 50MB
        return Err("PDF too large".into());
    }

    Ok(())
}
```

### 2. Combine with Guardrails

Extracted PDFs may contain sensitive data:

```toml
[guardrails.input]
type = "patterns"

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = false  # PDFs unlikely to have injection attacks
```

### 3. Use Specific System Prompts

Guide the LLM on what to extract:

```bash
# Good
--system-text "Extract only the executive summary section"

# Better
--system-text "Extract the executive summary. Focus on: business objectives, key findings, recommendations. Ignore appendices."
```

### 4. Structure Output with JSON Schema

For data extraction, enforce structure:

```bash
--response-format json-schema \
--response-format-schema schemas/summary.json
```

## Next Steps

- [Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %}) - Structure LLM output
- [Token Management]({{ site.baseurl }}{% link user-guide/token-management.md %}) - Handle large PDFs
- [Examples]({{ site.baseurl }}{% link examples/pdf-analysis.md %}) - More PDF workflows
