---
layout: default
title: Configuration
parent: User Guide
nav_order: 3
---

# Configuration

Learn how to use configuration files to manage LLM settings and guardrails.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Overview

Fortified LLM Client supports two configuration file formats:

- **TOML** (`.toml`) - Recommended for readability
- **JSON** (`.json`) - For programmatic generation

Configuration files can specify:
- LLM connection settings (API URL, model, authentication)
- Sampling parameters (temperature, max tokens, seed)
- Token validation settings
- Response formatting options
- **Guardrails configuration** (input/output validation)

## Merge Behavior

**Priority**: CLI arguments > Config file

CLI arguments always override config file values. This allows you to:
- Define base settings in the config file
- Override specific values via CLI flags as needed

**Example**:
```bash
# config.toml has model="llama3"
fortified-llm-client -c config.toml --model gpt-4  # Uses gpt-4, not llama3
```

## TOML Format

### Basic Configuration

`config.toml`:
```toml
# Connection settings
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

# Sampling parameters
temperature = 0.7
max_tokens = 2000
seed = 42

# Token management
validate_tokens = true
context_limit = 8192

# Timeout
timeout_secs = 300
```

### With System Prompt

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
system_prompt = "You are a helpful Rust expert. Explain concepts clearly."
temperature = 0.7
```

### With API Authentication

```toml
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key_name = "OPENAI_API_KEY"  # Environment variable name
temperature = 0.7
max_tokens = 1000
```

{: .warning }
> Never commit API keys directly in config files! Use `api_key_name` to reference environment variables.

### With Response Formatting

```toml
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key_name = "OPENAI_API_KEY"

# JSON Schema validation
response_format = "json-schema"
response_format_schema = "schemas/product.json"
response_format_schema_strict = true
```

### With Input Guardrails (Pattern-Based)

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7

[guardrails.input]
type = "patterns"
max_length_bytes = 1048576  # 1MB

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = true
```

### With Output Guardrails

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.output]
type = "patterns"

[guardrails.output.patterns]
detect_toxic = true
min_quality_score = 0.7
```

### With LLM-Based Guardrails (Llama Guard)

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.input]
type = "llama_guard"

[guardrails.input.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
max_tokens = 512
timeout_secs = 60

# Enable specific safety categories (S1-S13)
enabled_categories = ["S1", "S2", "S3"]  # Violent Crimes, Non-Violent Crimes, Sex-Related Crimes
```

See [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) for all guardrail types and options.

### Hybrid Guardrails (Defense-in-Depth)

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.input]
type = "hybrid"

[guardrails.input.hybrid]
execution_mode = "sequential"  # or "parallel"
aggregation_mode = "all"       # or "any", "majority"

# Layer 1: Fast pattern checks
[[guardrails.input.hybrid.providers]]
type = "patterns"
[guardrails.input.hybrid.providers.patterns]
detect_pii = true
detect_prompt_injection = true

# Layer 2: LLM-based validation
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
[guardrails.input.hybrid.providers.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
```

See [Hybrid Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) for detailed configuration.

## JSON Format

### Basic Configuration

`config.json`:
```json
{
  "api_url": "http://localhost:11434/v1/chat/completions",
  "model": "llama3",
  "temperature": 0.7,
  "max_tokens": 2000,
  "validate_tokens": true
}
```

### With Guardrails

`config.json`:
```json
{
  "api_url": "http://localhost:11434/v1/chat/completions",
  "model": "llama3",
  "guardrails": {
    "input": {
      "type": "patterns",
      "max_length_bytes": 1048576,
      "patterns": {
        "detect_pii": true,
        "detect_prompt_injection": true
      }
    },
    "output": {
      "type": "patterns",
      "patterns": {
        "detect_toxic": true
      }
    }
  }
}
```

## All Configuration Fields

### Top-Level Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `api_url` | String | LLM API endpoint URL | None (required) |
| `model` | String | Model name/identifier | None (required) |
| `provider` | String | Force provider: `"openai"` or `"ollama"` | Auto-detect |
| `system_prompt` | String | System prompt text | None |
| `temperature` | Float | Sampling temperature (0.0-2.0) | Provider default |
| `max_tokens` | Integer | Maximum response tokens | Provider default |
| `seed` | Integer | Random seed for reproducibility | None |
| `validate_tokens` | Boolean | Enable token validation | `false` |
| `context_limit` | Integer | Override context window limit | Auto-detect |
| `response_format` | String | `"text"`, `"json-object"`, or `"json-schema"` | `"text"` |
| `response_format_schema` | String | Path to JSON Schema file | None |
| `response_format_schema_strict` | Boolean | Strict schema validation | `true` |
| `api_key` | String | API key (direct value) | None |
| `api_key_name` | String | Environment variable for API key | None |
| `timeout_secs` | Integer | Request timeout in seconds | `300` |

### Guardrails Section

See [Guardrails Configuration]({{ site.baseurl }}{% link guardrails/index.md %}) for complete details.

## CLI-Only Fields

These fields **cannot** be set in config files and must be provided via CLI:

- `config_file` - Path to config file itself
- `verbose` - Enable verbose logging
- `quiet` - Suppress all logging
- `output` - Output file path
- `enable_input_validation` - Simple CLI-based input validation
- `max_input_length` - Max input bytes (CLI validation)
- `max_input_tokens` - Max input tokens (CLI validation)

{: .note }
> For guardrails, use the `[guardrails]` section in config files instead of CLI flags.

## Usage Examples

### Example 1: CLI with Config

```bash
# Use config as base
fortified-llm-client -c config.toml --user-text "Hello"

# Override specific values
fortified-llm-client -c config.toml --temperature 1.0 --user-text "Be creative"
```

### Example 2: Library with Config

```rust
use fortified_llm_client::{evaluate_with_guardrails, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: "Your prompt".to_string(),
        ..Default::default()
    };

    // Load guardrails from config file
    let result = evaluate_with_guardrails(config, "config.toml").await?;
    println!("{}", result.content);

    Ok(())
}
```

### Example 3: Multiple Configs

Create environment-specific configs:

`dev.toml`:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 1.0  # More creative for testing
```

`prod.toml`:
```toml
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key_name = "OPENAI_API_KEY"
temperature = 0.7
validate_tokens = true

[guardrails.input]
type = "hybrid"
# ... strict validation for production
```

Usage:
```bash
# Development
fortified-llm-client -c dev.toml --user-text "test"

# Production
fortified-llm-client -c prod.toml --user-text "production query"
```

## Validation

Config files are validated on load. Common errors:

### Missing Required Fields

**Error**: `Failed to merge config: missing field 'api_url'`

**Fix**: Add required fields (`api_url` and `model`)

### Invalid File Extension

**Error**: `Config file must have .json or .toml extension`

**Fix**: Rename file to `.toml` or `.json`

### TOML Syntax Errors

**Error**: `Failed to merge config: expected an equals, found ...`

**Fix**: Check TOML syntax (use TOML validator)

### JSON Syntax Errors

**Error**: `Failed to merge config: expected value at line X column Y`

**Fix**: Check JSON syntax (use JSON validator)

## Best Practices

### 1. Use Environment Variables for Secrets

**Don't**:
```toml
api_key = "sk-real-key-here"  # NEVER commit this!
```

**Do**:
```toml
api_key_name = "OPENAI_API_KEY"  # Reference env var
```

### 2. Environment-Specific Configs

Keep separate configs for different environments:
- `dev.toml` - Local development
- `staging.toml` - Staging environment
- `prod.toml` - Production

### 3. Override for Testing

Use config for stable settings, CLI for experiments:
```bash
# Base: config.toml (temperature=0.7)
fortified-llm-client -c config.toml --temperature 0.0  # Test deterministic
fortified-llm-client -c config.toml --temperature 1.5  # Test creative
```

### 4. Document Custom Configs

Add comments to explain non-obvious settings:
```toml
# GPT-4 Turbo with 128K context window
model = "gpt-4-turbo-preview"
context_limit = 128000

# Lower temperature for factual extraction tasks
temperature = 0.3
```

## Next Steps

- [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) - Configure security validation
- [Examples]({{ site.baseurl }}{% link examples/guardrails-config.md %}) - Complete config examples
- [CLI Usage]({{ site.baseurl }}{% link user-guide/cli-usage.md %}) - Override config with CLI flags
