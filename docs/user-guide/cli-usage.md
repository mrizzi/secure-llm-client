---
layout: default
title: CLI Usage
parent: User Guide
nav_order: 1
---

# CLI Usage

Complete reference for all command-line flags and options.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Synopsis

```bash
fortified-llm-client [OPTIONS] --api-url <API_URL> --model <MODEL>
```

At minimum, you must provide:
- `--api-url` (or via config file)
- `--model` (or via config file)
- One of: `--user-text`, `--user-file`, or `--pdf-file`

## Core Options

### --api-url, -a

**Description**: LLM API endpoint URL

**Required**: Yes (unless in config file)

**Examples**:
```bash
# Ollama (local)
--api-url http://localhost:11434/v1/chat/completions

# OpenAI
--api-url https://api.openai.com/v1/chat/completions

# Azure OpenAI
--api-url https://your-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2024-02-15-preview
```

### --model, -m

**Description**: Model name/identifier

**Required**: Yes (unless in config file)

**Examples**:
```bash
# Ollama
--model llama3
--model mistral
--model codellama

# OpenAI
--model gpt-4
--model gpt-3.5-turbo
```

### --provider

**Description**: Force specific provider format (overrides auto-detection)

**Values**: `openai`, `ollama`

**Default**: Auto-detected from API URL

**Example**:
```bash
--provider openai  # Force OpenAI format even for Ollama-compatible URLs
```

{: .note }
> Auto-detection analyzes the API URL to infer the provider. Explicitly set this only if auto-detection fails or you need to override it.

## Prompts

### System Prompts

Provide context or instructions to guide the LLM's behavior.

**--system-text**

Direct text input:
```bash
--system-text "You are a Rust expert. Explain concepts clearly."
```

**--system-file, -s**

Read from file:
```bash
--system-file prompts/expert.txt
```

{: .warning }
> `--system-text` and `--system-file` are mutually exclusive. Use one or the other.

### User Prompts

The main query or input to the LLM.

**--user-text**

Direct text input:
```bash
--user-text "Explain Rust ownership"
```

**--user-file, -u**

Read from file:
```bash
--user-file prompts/question.txt
```

**--pdf-file, -p**

Extract text from PDF (replaces user prompt):
```bash
--pdf-file document.pdf
```

{: .warning }
> `--user-text`, `--user-file`, and `--pdf-file` are mutually exclusive. Use only one.

## Configuration

### --config-file, -c

**Description**: Load default values from JSON or TOML file

**Merge Priority**: CLI arguments override config file values

**Examples**:
```bash
# TOML config
--config-file config.toml
-c config.toml

# JSON config
--config-file config.json
```

**Config file example** (`config.toml`):
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7
max_tokens = 2000
```

See [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) for full config file documentation.

## Sampling Parameters

### --temperature, -t

**Description**: Sampling temperature (controls randomness)

**Range**: `0.0` to `2.0`

**Default**: Provider default (usually `1.0`)

**Examples**:
```bash
--temperature 0.0   # Deterministic (greedy sampling)
--temperature 0.7   # Balanced creativity
--temperature 1.5   # More creative/random
```

{: .note }
> Lower = more deterministic, higher = more creative/random

### --max-tokens

**Description**: Maximum response tokens to generate

**Range**: `1` to model's max

**Default**: Provider default

**Example**:
```bash
--max-tokens 500   # Limit response to ~375 words
--max-tokens 2000  # Longer responses
```

### --seed

**Description**: Random seed for reproducible sampling

**Type**: Unsigned 64-bit integer

**Example**:
```bash
--seed 42  # Same seed + temperature 0 = reproducible outputs
```

## Token Management

### --validate-tokens

**Description**: Enable token count validation before API call

**Default**: `false`

**Example**:
```bash
--validate-tokens
```

When enabled:
1. Estimates tokens for system + user prompts
2. Validates against context limit
3. Fails early if exceeds limit (saves API cost)

See [Token Management]({{ site.baseurl }}{% link user-guide/token-management.md %}) for details.

### --context-limit

**Description**: Override model's context window limit

**Range**: `1` to `9999999999`

**Default**: Auto-detected from model name

**Example**:
```bash
--context-limit 8192   # Force 8K context limit
--context-limit 128000 # GPT-4 Turbo 128K context
```

{: .note }
> Only needed if auto-detection fails or you want to enforce a lower limit.

## Response Formatting

### --response-format

**Description**: Control output format (OpenAI-compatible models only)

**Values**: `text`, `json-object`, `json-schema`

**Default**: `text`

**Examples**:
```bash
# Plain text (default)
--response-format text

# Unstructured JSON object
--response-format json-object

# JSON validated against schema
--response-format json-schema --response-format-schema schema.json
```

See [Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %}) for complete guide.

### --response-format-schema

**Description**: JSON Schema file path (required when `--response-format=json-schema`)

**Example**:
```bash
--response-format json-schema \
--response-format-schema schemas/product.json
```

**Sample schema** (`schemas/product.json`):
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "price": {"type": "number"}
  },
  "required": ["name", "price"]
}
```

### --response-format-schema-strict

**Description**: Use strict mode for JSON schema validation

**Default**: `true`

**Example**:
```bash
--response-format-schema-strict false  # Disable strict mode
```

## Authentication

### --api-key

**Description**: API key for authentication (direct value)

**Security Warning**: Avoid using in shell history. Prefer `--api-key-name`.

**Example**:
```bash
--api-key sk-...
```

### --api-key-name

**Description**: Environment variable name containing the API key

**Recommended**: Use this instead of `--api-key`

**Examples**:
```bash
# Set env var
export OPENAI_API_KEY=sk-...

# Use in CLI
--api-key-name OPENAI_API_KEY
```

{: .note }
> `--api-key` and `--api-key-name` are mutually exclusive.

## Network & Performance

### --timeout

**Description**: Request timeout in seconds

**Range**: `> 0`

**Default**: `300` (5 minutes)

**Examples**:
```bash
--timeout 60    # 1 minute timeout
--timeout 600   # 10 minute timeout for large models
```

## Output Options

### --output, -o

**Description**: Write output to file instead of stdout

**Features**:
- Atomic writes (temp file + rename)
- Auto-creates parent directories
- Overwrites existing files

**Examples**:
```bash
--output response.json
-o results/output.json
```

## Logging

### --verbose, -v

**Description**: Enable verbose logging (DEBUG level)

**Conflicts with**: `--quiet`

**Example**:
```bash
--verbose   # Show debug logs
-v          # Short form
```

### --quiet, -q

**Description**: Suppress all logging output

**Conflicts with**: `--verbose`

**Example**:
```bash
--quiet  # No logs, only JSON output
-q       # Short form
```

## Input Validation (CLI-only)

{: .note }
> For LLM-based guardrails (Llama Guard, GPT OSS Safeguard, hybrid), use config files. These flags enable simple pattern-based validation only.

### --enable-input-validation

**Description**: Enable regex-based input validation (PII, prompt injection, etc.)

**Default**: `false`

**Example**:
```bash
--enable-input-validation
```

When enabled, applies default pattern checks:
- PII detection (SSN, credit cards, email addresses)
- Prompt injection patterns
- Suspicious characters/sequences

### --max-input-length

**Description**: Maximum input length in bytes (requires `--enable-input-validation`)

**Accepts**: Human-readable sizes (`100MB`, `1.5GB`, `500KB`) or plain bytes

**Default**: `1048576` (1MB) when validation enabled

**Examples**:
```bash
--enable-input-validation --max-input-length 2MB
--enable-input-validation --max-input-length 500KB
--enable-input-validation --max-input-length 1048576  # Bytes
```

### --max-input-tokens

**Description**: Maximum estimated input tokens (requires `--enable-input-validation`)

**Default**: `200000` (200K) when validation enabled

**Example**:
```bash
--enable-input-validation --max-input-tokens 100000
```

## Complete Examples

### Example 1: Minimal Invocation

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello, world!"
```

### Example 2: With All Common Options

```bash
fortified-llm-client \
  --config-file config.toml \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --system-text "You are a helpful assistant." \
  --user-text "Explain Rust ownership" \
  --temperature 0.7 \
  --max-tokens 500 \
  --validate-tokens \
  --output response.json \
  --verbose
```

### Example 3: OpenAI with JSON Schema

```bash
export OPENAI_API_KEY=sk-...

fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Generate a product catalog" \
  --response-format json-schema \
  --response-format-schema schemas/product.json \
  --output catalog.json
```

### Example 4: PDF Analysis with Guardrails

```bash
fortified-llm-client \
  --config-file guardrails-config.toml \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --pdf-file research-paper.pdf \
  --system-text "Summarize key findings" \
  --enable-input-validation \
  --max-input-length 5MB \
  --output summary.json
```

## Getting Help

```bash
# Show all options
fortified-llm-client --help

# Show version
fortified-llm-client --version
```

## Next Steps

- [Library API]({{ site.baseurl }}{% link user-guide/library-api.md %}) - Use from Rust code
- [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) - Config file formats
- [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) - Advanced security validation
- [Examples]({{ site.baseurl }}{% link examples/index.md %}) - More use cases
