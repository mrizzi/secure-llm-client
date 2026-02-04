---
layout: default
title: Token Management
parent: User Guide
nav_order: 6
---

# Token Management

Estimate and validate token usage to avoid exceeding model limits.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Overview

Token management helps you:
- Estimate token counts before making API calls
- Validate prompts don't exceed model context limits
- Budget token usage for cost optimization
- Fail early to save API costs

## What Are Tokens?

Tokens are the basic units LLMs use to process text:
- 1 token ≈ 4 characters in English
- 1 token ≈ ¾ of a word
- Tokens include punctuation and spaces

**Example**:
- "Hello, world!" ≈ 4 tokens
- "Explain Rust ownership" ≈ 3 tokens
- A 1000-word essay ≈ 1333 tokens

## Token Estimation

Fortified LLM Client estimates tokens using model-specific tokenizers.

### Enable Token Validation

**CLI**:
```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Your prompt" \
  --validate-tokens
```

**Config file**:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
validate_tokens = true
```

**Library**:
```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    user_prompt: "Your prompt".to_string(),
    validate_tokens: true,
    ..Default::default()
};

let result = evaluate(config).await?;
println!("Tokens used: {}", result.metadata.tokens_estimated);
```

### What Gets Estimated

Token estimation includes:
1. **System prompt** (if provided)
2. **User prompt** (or PDF-extracted text)
3. **Response buffer** (reserved for LLM output based on `max_tokens`)

**Formula**:
```
total_tokens = system_tokens + user_tokens + response_buffer
```

## Context Limits

### Auto-Detection

Context limits are auto-detected from model names:

| Model Pattern | Context Limit |
|---------------|---------------|
| `gpt-4-turbo` | 128,000 tokens |
| `gpt-4-32k` | 32,768 tokens |
| `gpt-4` | 8,192 tokens |
| `gpt-3.5-turbo-16k` | 16,385 tokens |
| `gpt-3.5-turbo` | 4,096 tokens |
| `claude-3-opus` | 200,000 tokens |
| `claude-3-sonnet` | 200,000 tokens |
| `llama3` | 8,192 tokens |
| `llama3-70b` | 8,192 tokens |
| `mistral` | 8,192 tokens |

**Fallback**: 4096 tokens if model not recognized.

### Override Context Limit

Force a specific context limit:

**CLI**:
```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model custom-model \
  --user-text "Your prompt" \
  --validate-tokens \
  --context-limit 16384
```

**Config file**:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "custom-model"
validate_tokens = true
context_limit = 16384
```

**Library**:
```rust
let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "custom-model".to_string(),
    user_prompt: "Your prompt".to_string(),
    validate_tokens: true,
    context_limit: Some(16384),
    ..Default::default()
};
```

## Validation Behavior

### Success Case

If tokens ≤ context limit, request proceeds:

```json
{
  "status": "success",
  "content": "LLM response...",
  "metadata": {
    "tokens_estimated": 1234,
    ...
  }
}
```

### Failure Case

If tokens > context limit, request fails **before** calling the API:

```json
{
  "status": "error",
  "error": {
    "code": "ValidationError",
    "message": "Token count (10000) exceeds model context limit (8192)"
  }
}
```

**Benefits**:
- No API charges for invalid requests
- Immediate feedback
- Faster failure (no network round-trip)

## Use Cases

### Use Case 1: Validate Large PDFs

Prevent exceeding limits when processing PDFs:

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file large-document.pdf \
  --system-text "Summarize" \
  --validate-tokens \
  --context-limit 8192
```

If PDF text + system prompt + response buffer > 8192 tokens, fails before extraction.

### Use Case 2: Cost Optimization

Estimate cost before expensive API calls:

```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-file long-prompt.txt \
  --validate-tokens
```

Check `metadata.tokens_estimated` to calculate cost:
- GPT-4: $0.03 per 1K input tokens
- Estimated tokens: 5000
- Cost: `5 * $0.03 = $0.15`

### Use Case 3: Batch Processing with Token Budgets

Process multiple prompts with total token budget:

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn batch_with_budget(prompts: Vec<String>, max_total_tokens: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut total_tokens = 0;

    for prompt in prompts {
        let config = EvaluationConfig {
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            model: "llama3".to_string(),
            user_prompt: prompt.clone(),
            validate_tokens: true,
            ..Default::default()
        };

        match evaluate(config).await {
            Ok(result) => {
                total_tokens += result.metadata.tokens_estimated;
                if total_tokens > max_total_tokens {
                    println!("Budget exceeded at {} tokens", total_tokens);
                    break;
                }
                println!("Processed: {}", result.content);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
```

## Token Estimation Accuracy

### Estimation Methods

Fortified LLM Client uses:
1. **Model-specific tokenizers** - When model is recognized (GPT-4, Llama 3, etc.)
2. **Character-based estimation** - Fallback: `char_count / 4`

**Accuracy**: ±5% for recognized models, ±15% for fallback estimation.

### Factors Affecting Accuracy

- **Language**: Non-English text may use more tokens
- **Special characters**: Code, emojis, symbols increase token count
- **Tokenizer version**: Provider may update tokenizers

{: .note }
> Estimation is conservative (tends to overestimate) to avoid unexpected limit exceedances.

## Advanced Configuration

### Custom Response Buffer

By default, response buffer = `max_tokens` (if set) or provider default.

Override:

```rust
// Not directly exposed, but max_tokens controls it
let config = EvaluationConfig {
    // ...
    max_tokens: Some(1000),  // Reserves 1000 tokens for response
    validate_tokens: true,
    ..Default::default()
};
```

**Calculation**:
- If `max_tokens = 1000`: total = system + user + 1000
- If `max_tokens = None`: total = system + user + default (often 2048)

### Disable Validation for Specific Requests

```rust
let config = EvaluationConfig {
    // ...
    validate_tokens: false,  // Skip validation
    ..Default::default()
};
```

Use when:
- You know the prompt is safe
- You want to let the API handle limits
- Debugging estimation issues

## Troubleshooting

### Error: "Token count exceeds limit"

**Cause**: Prompt too long for model.

**Fixes**:
1. **Shorten prompt**: Reduce system/user prompt length
2. **Use larger model**: Switch to model with bigger context (e.g., gpt-4-turbo 128K)
3. **Reduce max_tokens**: Lower response buffer
4. **Split prompts**: Break into multiple smaller requests

### Estimation Seems Wrong

**Cause**: Model not recognized or tokenizer mismatch.

**Debug**:
```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model unknown-model \
  --user-text "test" \
  --validate-tokens \
  --verbose
```

Check logs for tokenizer being used.

**Fix**: Override context limit if estimation is incorrect:
```bash
--context-limit 32768
```

### Validation Passes but API Rejects

**Cause**: API's actual limit differs from estimation.

**Fix**: Set a lower context limit to add safety margin:
```bash
--context-limit 7000  # Instead of 8192
```

## Best Practices

### 1. Always Validate for Unknown Input

When processing user input or PDFs:

```bash
--validate-tokens --context-limit 8192
```

### 2. Add Safety Margin

Reserve 10-20% headroom:

```bash
# Model limit: 8192
--context-limit 7000  # ~15% margin
```

### 3. Monitor Token Usage

Track tokens in responses:

```rust
let result = evaluate(config).await?;
println!("Tokens used: {}", result.metadata.tokens_estimated);

// Log to metrics system
metrics::gauge!("llm.tokens.used", result.metadata.tokens_estimated as f64);
```

### 4. Use Appropriate Models

Match model to task:

- **Short prompts**: gpt-3.5-turbo (4K context)
- **Long prompts**: gpt-4-turbo (128K context)
- **Huge documents**: claude-3-opus (200K context)

## Next Steps

- [PDF Extraction]({{ site.baseurl }}{% link user-guide/pdf-extraction.md %}) - Handle large PDFs
- [Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %}) - Control output structure
- [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) - Set defaults
