---
layout: default
title: Testing
parent: Architecture
nav_order: 4
---

# Testing Strategy

Test organization and best practices.

## Test Organization

Tests are organized by purpose in the `tests/` directory:

```
tests/
├── unit_tests.rs              # Core functionality tests
├── provider_tests.rs          # Provider-specific behavior
├── config_tests.rs            # Config merging tests
├── config_file_request_tests.rs  # Config parsing tests
├── guardrail_*.rs             # Guardrail validation tests
├── integration_tests.rs       # End-to-end workflows
└── fixtures/                  # Test data
    ├── pdfs/
    ├── schemas/
    └── configs/
```

## Running Tests

### All Tests

```bash
cargo test
```

{: .note }
> Requires Docling CLI for PDF extraction tests: `pip install docling`

### Specific Test File

```bash
cargo test --test unit_tests
cargo test --test provider_tests
```

### Specific Test Function

```bash
cargo test test_evaluate_basic
cargo test test_token_estimation
```

### With Output

```bash
cargo test -- --nocapture
```

## Test Categories

### Unit Tests

**Location**: `tests/unit_tests.rs`

**Coverage**:
- Token estimation accuracy
- Config builder functionality
- Error handling
- Response parsing

**Example**:
```rust
#[tokio::test]
async fn test_token_estimation() {
    let text = "This is a test prompt";
    let tokens = estimate_tokens(text, "gpt-4");
    assert!(tokens > 0 && tokens < 100);
}
```

### Provider Tests

**Location**: `tests/provider_tests.rs`

**Coverage**:
- Provider detection logic
- OpenAI/Ollama specific behavior
- Request/response formatting
- Error handling per provider

**Example**:
```rust
#[test]
fn test_detect_openai() {
    let url = "https://api.openai.com/v1/chat/completions";
    assert_eq!(detect_provider(url), Provider::OpenAI);
}

#[test]
fn test_detect_ollama() {
    let url = "http://localhost:11434/v1/chat/completions";
    assert_eq!(detect_provider(url), Provider::Ollama);
}
```

### Configuration Tests

**Location**: `tests/config_tests.rs`, `tests/config_file_request_tests.rs`

**Coverage**:
- TOML/JSON parsing
- Figment merging (CLI override)
- Config validation
- Guardrails configuration

**Example**:
```rust
#[test]
fn test_config_merge_priority() {
    // Config file has temperature=0.5
    // CLI args have temperature=0.9
    let merged = merge_config(&cli_args, &config_file);
    assert_eq!(merged.temperature, Some(0.9)); // CLI wins
}
```

### Guardrail Tests

**Location**: `tests/guardrail_*.rs`

**Coverage**:
- Pattern-based validation (PII, prompt injection)
- LLM-based guardrails (mocked)
- Hybrid execution modes
- Aggregation strategies

**Example**:
```rust
#[tokio::test]
async fn test_pii_detection() {
    let guardrail = create_input_guardrail(/* ... */);
    let result = guardrail.validate("My SSN is 123-45-6789").await;
    assert!(result.is_err()); // Should detect PII
}
```

### Integration Tests

**Location**: `tests/integration_tests.rs`

**Coverage**:
- End-to-end workflows
- PDF extraction + LLM analysis
- Guardrails + LLM + validation
- Complete pipeline execution

**Example**:
```rust
#[tokio::test]
async fn test_pdf_extraction_pipeline() {
    let config = EvaluationConfig {
        pdf_input: Some("tests/fixtures/pdfs/sample.pdf".to_string()),
        // ...
    };
    let result = evaluate(config).await.unwrap();
    assert!(!result.content.is_empty());
}
```

## Test Fixtures

**Location**: `tests/fixtures/`

### PDF Fixtures

```
tests/fixtures/pdfs/
├── sample.pdf           # Simple text document
├── multi_column.pdf     # Complex layout
└── large.pdf            # Near size limit
```

### JSON Schema Fixtures

```
tests/fixtures/schemas/
├── product.json         # Simple object schema
├── catalog.json         # Array of objects
└── strict.json          # additionalProperties: false
```

### Config Fixtures

```
tests/fixtures/configs/
├── basic.toml           # Minimal config
├── guardrails.toml      # With guardrails
└── hybrid.toml          # Hybrid guardrails
```

## Mocking Strategies

### HTTP Mocking (Provider Tests)

Use `mockito` for LLM API mocking:

```rust
use mockito::{mock, server_url};

#[tokio::test]
async fn test_llm_invocation() {
    let _m = mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"
            {
                "choices": [{
                    "message": {"content": "Mocked response"}
                }]
            }
        "#)
        .create();

    let config = EvaluationConfig {
        api_url: server_url(),
        // ...
    };

    let result = evaluate(config).await.unwrap();
    assert_eq!(result.content, "Mocked response");
}
```

### External Tool Mocking (PDF Tests)

PDF tests require actual Docling CLI (not mocked):

```rust
#[tokio::test]
#[ignore] // Skip if docling not installed
async fn test_pdf_extraction() {
    // Requires: pip install docling
    let result = extract_pdf_text("tests/fixtures/pdfs/sample.pdf").await;
    assert!(result.is_ok());
}
```

## CI/CD Integration

Tests run in GitHub Actions with strict settings:

```bash
# Warnings treated as errors
RUSTFLAGS="-D warnings" cargo test
```

### Pre-Push Checklist

Run locally before pushing:

```bash
# 1. Format
cargo +nightly fmt --check

# 2. Compile check
RUSTFLAGS="-D warnings" cargo check

# 3. Linting
cargo clippy -- -D warnings

# 4. Tests
RUSTFLAGS="-D warnings" cargo test
```

## Property-Based Testing

**Location**: Uses `proptest` crate

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_token_estimation_never_negative(s in "\\PC*") {
        let tokens = estimate_tokens(&s, "gpt-4");
        prop_assert!(tokens >= 0);
    }
}
```

## Coverage Goals

- **Unit tests**: >80% coverage
- **Integration tests**: Critical paths covered
- **Provider tests**: All supported providers
- **Guardrail tests**: All validation types

## Best Practices

1. **Test names describe behavior**: `test_token_validation_fails_when_exceeds_limit()`
2. **Use fixtures for complex data**: Don't inline large JSON/PDFs
3. **Mock external APIs**: Use `mockito` for HTTP, avoid real API calls
4. **Test error cases**: Don't just test happy paths
5. **Isolate tests**: No shared state between tests

## See Also

- [Contributing]({{ site.baseurl }}{% link contributing/ci-checklist.md %}) - Pre-push checklist
- [Layers]({{ site.baseurl }}{% link architecture/layers.md %}) - What each layer should test
