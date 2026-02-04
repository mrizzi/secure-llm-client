---
layout: default
title: User Guide
nav_order: 3
has_children: true
permalink: /user-guide/
---

# User Guide

Comprehensive guides for using Fortified LLM Client in your projects.

## Contents

This section covers:

- **[CLI Usage]({{ site.baseurl }}{% link user-guide/cli-usage.md %})** - Complete CLI reference with all flags and options
- **[Library API]({{ site.baseurl }}{% link user-guide/library-api.md %})** - Rust library API documentation and examples
- **[Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %})** - Config file formats and merging behavior
- **[PDF Extraction]({{ site.baseurl }}{% link user-guide/pdf-extraction.md %})** - Extract and analyze PDF documents
- **[Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %})** - Control output format (text, JSON, JSON Schema)
- **[Token Management]({{ site.baseurl }}{% link user-guide/token-management.md %})** - Estimate and validate token usage

## Quick Reference

### CLI Basics

```bash
# Minimal invocation
fortified-llm-client --api-url URL --model MODEL --user-text "prompt"

# With config file
fortified-llm-client -c config.toml --user-text "prompt"

# Save output to file
fortified-llm-client -c config.toml --user-text "prompt" -o output.json
```

### Library Basics

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    user_prompt: "Your prompt".to_string(),
    ..Default::default()
};

let result = evaluate(config).await?;
```

## Common Workflows

### 1. Basic LLM Interaction

CLI:
```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Explain Rust ownership"
```

Library:
```rust
let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    user_prompt: "Explain Rust ownership".to_string(),
    ..Default::default()
};
let result = evaluate(config).await?;
```

### 2. PDF Analysis

CLI:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --pdf-file document.pdf \
  --system-text "Summarize the key points"
```

See [PDF Extraction]({{ site.baseurl }}{% link user-guide/pdf-extraction.md %}) for details.

### 3. Structured JSON Output

CLI:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --user-text "Generate a product catalog" \
  --response-format json-schema \
  --response-format-schema schema.json
```

See [Response Formats]({{ site.baseurl }}{% link user-guide/response-formats.md %}) for details.

### 4. With Input Validation

CLI:
```bash
fortified-llm-client -c config.toml \
  --enable-input-validation \
  --max-input-length 1MB \
  --user-text "Your prompt"
```

Config file:
```toml
[guardrails.input]
type = "patterns"
max_length_bytes = 1048576

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = true
```

See [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) for comprehensive security options.

## Next Steps

- New users: Start with [CLI Usage]({{ site.baseurl }}{% link user-guide/cli-usage.md %}) for a complete flag reference
- Library users: See [Library API]({{ site.baseurl }}{% link user-guide/library-api.md %}) for Rust integration
- Advanced users: Explore [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) for complex setups
