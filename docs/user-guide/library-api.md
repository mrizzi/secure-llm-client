---
layout: default
title: Library API
parent: User Guide
nav_order: 2
---

# Library API

Use Fortified LLM Client as a Rust library in your applications.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fortified_llm_client = { git = "https://github.com/mrizzi/fortified-llm-client" }
tokio = { version = "1", features = ["full"] }
```

{: .note }
> The library is not yet published to crates.io. Use the git dependency until the first stable release.

## Core API

### evaluate()

LLM evaluation with optional security guardrails.

**Signature**:
```rust
pub async fn evaluate(config: EvaluationConfig) -> Result<EvaluationResult, FortifiedError>
```

**Basic Example** (no guardrails):
```rust
use fortified_llm_client::{evaluate, ConfigBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .user_prompt("Explain Rust ownership")
        .build()?;

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);
    println!("Tokens: {}", result.metadata.tokens_estimated);

    Ok(())
}
```

**With Input Guardrails**:
```rust
use fortified_llm_client::{
    evaluate, ConfigBuilder,
    guardrails::{GuardrailProviderConfig, RegexGuardrailConfig, Severity},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_guardrails = GuardrailProviderConfig::Regex(
        RegexGuardrailConfig {
            max_length_bytes: 1048576,  // 1MB limit
            patterns_file: Some("patterns/input.txt".into()),
            severity_threshold: Severity::Medium,
        }
    );

    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .user_prompt("Your prompt here")
        .input_guardrails(input_guardrails)
        .build()?;

    let result = evaluate(config).await?;
    println!("Safe response: {}", result.content);

    Ok(())
}
```

**With Both Input and Output Guardrails**:
```rust
use fortified_llm_client::{
    evaluate, ConfigBuilder,
    guardrails::{GuardrailProviderConfig, RegexGuardrailConfig, Severity},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_guardrails = GuardrailProviderConfig::Regex(
        RegexGuardrailConfig {
            max_length_bytes: 1048576,
            patterns_file: Some("patterns/input.txt".into()),
            severity_threshold: Severity::Medium,
        }
    );

    let output_guardrails = GuardrailProviderConfig::Regex(
        RegexGuardrailConfig {
            max_length_bytes: 2097152,  // 2MB limit for responses
            patterns_file: Some("patterns/output.txt".into()),
            severity_threshold: Severity::High,
        }
    );

    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .user_prompt("Your prompt here")
        .input_guardrails(input_guardrails)
        .output_guardrails(output_guardrails)
        .build()?;

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);

    Ok(())
}
```

## Data Structures

### EvaluationConfig

Configuration for LLM evaluation.

**Fields**:
```rust
pub struct EvaluationConfig {
    /// LLM API endpoint URL
    pub api_url: String,

    /// Model name/identifier
    pub model: String,

    /// Optional system prompt
    pub system_prompt: Option<String>,

    /// User prompt (main query)
    pub user_prompt: String,

    /// Optional PDF file path (extracted text replaces user_prompt)
    pub pdf_input: Option<String>,

    /// Sampling temperature (0.0-2.0)
    pub temperature: Option<f32>,

    /// Maximum response tokens
    pub max_tokens: Option<u32>,

    /// Random seed for reproducibility
    pub seed: Option<u64>,

    /// Enable token validation
    pub validate_tokens: bool,

    /// Override context window limit
    pub context_limit: Option<usize>,

    /// Response format (text, json-object, json-schema)
    pub response_format: Option<ResponseFormat>,

    /// JSON Schema for validation
    pub response_format_schema: Option<String>,

    /// Strict schema validation mode
    pub response_format_schema_strict: bool,

    /// API key (direct value)
    pub api_key: Option<String>,

    /// Environment variable name for API key
    pub api_key_name: Option<String>,

    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,

    /// Force specific provider format
    pub provider: Option<Provider>,
}
```

**Default**:
```rust
impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            model: String::new(),
            system_prompt: None,
            user_prompt: String::new(),
            pdf_input: None,
            temperature: None,
            max_tokens: None,
            seed: None,
            validate_tokens: false,
            context_limit: None,
            response_format: None,
            response_format_schema: None,
            response_format_schema_strict: true,
            api_key: None,
            api_key_name: None,
            timeout_secs: None,
            provider: None,
        }
    }
}
```

### EvaluationResult

Successful LLM response with metadata.

**Fields**:
```rust
pub struct EvaluationResult {
    /// LLM-generated content
    pub content: String,

    /// Execution metadata
    pub metadata: Metadata,
}
```

### Metadata

Execution details and statistics.

**Fields**:
```rust
pub struct Metadata {
    /// Model used for evaluation
    pub model: String,

    /// Estimated total tokens (input + output)
    pub tokens_estimated: usize,

    /// Request latency in milliseconds
    pub latency_ms: u64,

    /// Timestamp (ISO 8601)
    pub timestamp: String,

    /// Provider type (openai, ollama)
    pub provider: Option<Provider>,

    /// System prompt (if provided)
    pub system_prompt: Option<String>,

    /// User prompt
    pub user_prompt: String,

    /// Temperature used
    pub temperature: Option<f32>,

    /// Max tokens requested
    pub max_tokens: Option<u32>,
}
```

### ResponseFormat

Output format options (OpenAI-compatible only).

**Enum**:
```rust
pub enum ResponseFormat {
    /// Plain text (default)
    Text,

    /// Unstructured JSON object
    JsonObject,

    /// JSON validated against schema
    JsonSchema,
}
```

### Provider

LLM provider type.

**Enum**:
```rust
pub enum Provider {
    OpenAI,
    Ollama,
}
```

## Error Handling

### FortifiedError

Main error type for all operations.

**Variants**:
```rust
pub enum FortifiedError {
    /// API-related errors (network, authentication, rate limits)
    ApiError {
        message: String,
        status_code: Option<u16>,
    },

    /// Validation errors (guardrails, schema, tokens)
    ValidationError {
        message: String,
        validation_type: Option<String>,
    },

    /// Configuration errors (invalid config, missing fields)
    ConfigError {
        message: String,
    },

    /// PDF extraction errors
    PdfError {
        message: String,
    },

    /// Generic internal errors
    InternalError {
        message: String,
    },
}
```

**Example handling**:
```rust
use fortified_llm_client::{evaluate, EvaluationConfig, FortifiedError};

#[tokio::main]
async fn main() {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: "Hello!".to_string(),
        ..Default::default()
    };

    match evaluate(config).await {
        Ok(result) => {
            println!("Success: {}", result.content);
        }
        Err(FortifiedError::ApiError { message, status_code }) => {
            eprintln!("API error ({}): {}", status_code.unwrap_or(0), message);
        }
        Err(FortifiedError::ValidationError { message, .. }) => {
            eprintln!("Validation failed: {}", message);
        }
        Err(FortifiedError::ConfigError { message }) => {
            eprintln!("Config error: {}", message);
        }
        Err(e) => {
            eprintln!("Unexpected error: {:?}", e);
        }
    }
}
```

## Common Patterns

### Pattern 1: Basic LLM Call

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn ask_llm(question: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: question.to_string(),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```

### Pattern 2: With Custom Parameters

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn creative_completion(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        system_prompt: Some("You are a creative writer.".to_string()),
        user_prompt: prompt.to_string(),
        temperature: Some(1.2),  // More creative
        max_tokens: Some(1000),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```

### Pattern 3: Deterministic Output

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn deterministic_completion(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: prompt.to_string(),
        temperature: Some(0.0),  // Deterministic
        seed: Some(42),          // Reproducible
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```

### Pattern 4: PDF Analysis

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn analyze_pdf(pdf_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        system_prompt: Some("Summarize the key points from this document.".to_string()),
        user_prompt: String::new(),  // Replaced by PDF content
        pdf_input: Some(pdf_path.to_string()),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```

### Pattern 5: Structured JSON Output

```rust
use fortified_llm_client::{evaluate, EvaluationConfig, ResponseFormat};

async fn generate_json(prompt: &str, schema_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "https://api.openai.com/v1/chat/completions".to_string(),
        model: "gpt-4".to_string(),
        user_prompt: prompt.to_string(),
        response_format: Some(ResponseFormat::JsonSchema),
        response_format_schema: Some(schema_path.to_string()),
        response_format_schema_strict: true,
        api_key_name: Some("OPENAI_API_KEY".to_string()),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```

### Pattern 6: With Guardrails

```rust
use fortified_llm_client::{evaluate_with_guardrails, EvaluationConfig};

async fn safe_completion(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: prompt.to_string(),
        ..Default::default()
    };

    let result = evaluate_with_guardrails(config, "guardrails.toml").await?;
    Ok(result.content)
}
```

## Advanced Usage

### Custom Timeout

```rust
let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3",
    user_prompt: "Long computation task".to_string(),
    timeout_secs: Some(600),  // 10 minutes
    ..Default::default()
};
```

### Provider Override

Force specific provider format:

```rust
use fortified_llm_client::Provider;

let config = EvaluationConfig {
    api_url: "http://custom-api.com/v1/chat/completions".to_string(),
    model: "custom-model".to_string(),
    user_prompt: "Hello".to_string(),
    provider: Some(Provider::OpenAI),  // Force OpenAI format
    ..Default::default()
};
```

### Token Validation

Fail early if prompt exceeds model limits:

```rust
let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    user_prompt: very_long_prompt,
    validate_tokens: true,
    context_limit: Some(8192),
    ..Default::default()
};
```

## Next Steps

- [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) - Config file formats
- [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) - Security validation
- [Examples]({{ site.baseurl }}{% link examples/index.md %}) - More code samples
- [Error Handling]({{ site.baseurl }}{% link advanced/error-handling.md %}) - Detailed error handling guide
