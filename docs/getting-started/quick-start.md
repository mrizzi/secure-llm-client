---
layout: default
title: Quick Start
parent: Getting Started
nav_order: 3
---

# Quick Start

This tutorial walks you through your first LLM interactions using both the CLI and library API.

## Prerequisites

- Fortified LLM Client [installed]({{ site.baseurl }}{% link getting-started/installation.md %})
- Access to an LLM provider (Ollama or OpenAI)

## CLI Quick Start

### Example 1: Basic LLM Call

Using Ollama (local):
```bash
fortified-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Explain Rust ownership in one sentence"
```

Using OpenAI:
```bash
export OPENAI_API_KEY=sk-...
fortified-llm-client --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Explain Rust ownership in one sentence"
```

**Expected output**:
```json
{
  "status": "success",
  "content": "Rust ownership ensures memory safety by enforcing that each value has a single owner, automatically deallocating when the owner goes out of scope.",
  "metadata": {
    "model": "llama3",
    "tokens_estimated": 45,
    "latency_ms": 1234,
    "timestamp": "2025-01-30T12:00:00Z"
  }
}
```

### Example 2: With System Prompt

Add context to guide the LLM:
```bash
fortified-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --system-text "You are a Rust expert. Explain concepts clearly and concisely." \
  --user-text "What is the borrow checker?"
```

### Example 3: Save Output to File

```bash
fortified-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Write a haiku about Rust" \
  --output response.json
```

Check the file:
```bash
cat response.json | jq '.content'
```

### Example 4: Using a Config File

Create `config.toml`:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7
max_tokens = 500
```

Run with config:
```bash
fortified-llm-client --config config.toml \
  --user-text "What are the benefits of Rust?"
```

{: .note }
> CLI arguments override config file values. This allows you to reuse configs while customizing specific parameters.

## Library Quick Start

### Example 1: Basic Usage

Create `examples/basic.rs`:

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: "Explain Rust ownership".to_string(),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);
    println!("Tokens: {}", result.metadata.tokens_estimated);

    Ok(())
}
```

Run it:
```bash
cargo run --example basic
```

### Example 2: With System Prompt and Temperature

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        system_prompt: Some("You are a Rust expert.".to_string()),
        user_prompt: "What is the borrow checker?".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(500),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    println!("{}", result.content);

    Ok(())
}
```

### Example 3: With Error Handling

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
        Err(FortifiedError::ApiError { message, .. }) => {
            eprintln!("API error: {}", message);
        }
        Err(FortifiedError::ValidationError { message, .. }) => {
            eprintln!("Validation error: {}", message);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
```

## Testing Your Setup

### Verify Ollama is Running

```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

If this fails, start Ollama: `ollama serve`

### Verify OpenAI API Key

```bash
echo $OPENAI_API_KEY
# Should print: sk-...
```

If empty, set it:
```bash
export OPENAI_API_KEY=sk-your-key-here
```

## Common Issues

### "Error: Model not found"

**Ollama**: Pull the model first:
```bash
ollama pull llama3
```

**OpenAI**: Check model name spelling (e.g., `gpt-4`, not `gpt4`).

### "Connection refused"

**Ollama not running**: Start it with `ollama serve`

**Wrong API URL**: Verify the URL matches your provider's endpoint.

### "API key not found"

Set the environment variable:
```bash
export OPENAI_API_KEY=sk-...
```

Or use `--api-key-name` flag to specify a different variable name.

## Next Steps

Now that you've completed the quick start:

- **[User Guide]({{ site.baseurl }}{% link user-guide/index.md %})** - Explore all CLI flags and library options
- **[Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %})** - Learn about config file formats
- **[Guardrails]({{ site.baseurl }}{% link guardrails/index.md %})** - Add security validation to your prompts
- **[Examples]({{ site.baseurl }}{% link examples/index.md %})** - See more advanced use cases

## Learn More

- [CLI Usage]({{ site.baseurl }}{% link user-guide/cli-usage.md %}) - Full CLI reference
- [Library API]({{ site.baseurl }}{% link user-guide/library-api.md %}) - Complete API documentation
- [PDF Extraction]({{ site.baseurl }}{% link user-guide/pdf-extraction.md %}) - Work with PDF files
- [Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %}) - JSON Schema validation
