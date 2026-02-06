---
layout: default
title: Guardrails
nav_order: 5
has_children: true
permalink: /guardrails/
---

# Guardrails

Multi-layered security validation for LLM inputs and outputs.

## Overview

Fortified LLM Client provides five types of guardrails to protect against unsafe or malicious LLM interactions:

1. **Regex** - Fast pattern-based validation (custom patterns, length limits)
2. **Llama Guard** - MLCommons safety taxonomy (13 categories S1-S13)
3. **Llama Prompt Guard** - Jailbreak detection
4. **GPT OSS Safeguard** - GPT-4 based policy validation
5. **Composite** - Composable multi-provider validation

## Key Concepts

### Defense in Depth

Layer multiple guardrails for comprehensive protection:

```toml
[guardrails.input]
type = "composite"
execution = "parallel"
aggregation = "all_must_pass"

# Layer 1: Fast regex checks
[[guardrails.input.providers]]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/input.txt"

# Layer 2: LLM-based validation
[[guardrails.input.providers]]
type = "llama_guard"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard3:8b"
```

### System Prompts Are Trusted

**Important**: Guardrails ONLY validate user-provided inputs, NOT system prompts.

- **System prompts** - Developer-controlled, trusted content
- **User prompts** - User-provided, must be validated

### Input vs Output Guardrails

- **Input Guardrails** - Validate before sending to LLM (prevents harmful inputs)
- **Output Guardrails** - Validate LLM responses (ensures safe outputs)

## Configuration Formats

Guardrails can be configured in two ways:

1. **Separate Input/Output**: Use `[guardrails.input]` and `[guardrails.output]` for different configurations
2. **Unified**: Use `[guardrails]` to apply the same configuration to both input and output

See the [Configuration Guide]({{ site.baseurl }}{% link user-guide/configuration.md %}) for details.

## Quick Start

### CLI (Simple Validation)

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Your prompt" \
  --enable-input-validation \
  --max-input-length 1MB
```

### Config File (Advanced Validation)

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.input]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/input.txt"
severity_threshold = "medium"

[guardrails.output]
type = "regex"
max_length_bytes = 2097152
patterns_file = "patterns/output.txt"
severity_threshold = "high"
```

Pattern file format (`patterns/input.txt`):
```
CRITICAL | SSN | \b\d{3}-\d{2}-\d{4}\b
HIGH | Credit Card | \b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b
MEDIUM | Email | [a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[A-Z|a-z]{2,}
```

## Guardrail Types

| Type | Speed | Accuracy | Use Case |
|------|-------|----------|----------|
| **Regex** | Fast (<10ms) | Good | Custom patterns, length limits (input & output) |
| **Llama Guard** | Slow (1-3s) | Excellent | Comprehensive safety (S1-S13) |
| **Llama Prompt Guard** | Slow (1-3s) | Excellent | Advanced jailbreak detection |
| **GPT OSS Safeguard** | Slow (2-5s) | Excellent | Custom policy validation |
| **Composite** | Variable | Best | Combine multiple strategies |

## Section Contents

- **[Regex Guardrails]({{ site.baseurl }}{% link guardrails/regex.md %})** - Fast pattern-based validation (input & output)
- **[Llama Guard]({{ site.baseurl }}{% link guardrails/llama-guard.md %})** - MLCommons safety taxonomy
- **[Llama Prompt Guard]({{ site.baseurl }}{% link guardrails/llama-prompt-guard.md %})** - Jailbreak detection
- **[GPT OSS Safeguard]({{ site.baseurl }}{% link guardrails/gpt-oss-safeguard.md %})** - Policy-based validation
- **[Composite Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %})** - Multi-provider strategies
- **[Custom Policies]({{ site.baseurl }}{% link guardrails/custom-policies.md %})** - Creating custom policy files

## Choosing the Right Guardrail

### For Development/Testing

```toml
[guardrails.input]
type = "patterns"  # Fast, low cost

[guardrails.input.patterns]
detect_prompt_injection = true
```

### For Production (Balanced)

```toml
[guardrails.input]
type = "hybrid"
execution_mode = "sequential"

# Fast check first
[[guardrails.input.hybrid.providers]]
type = "patterns"

# LLM check only if patterns pass
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
```

### For High-Security Environments

```toml
[guardrails.input]
type = "hybrid"
execution_mode = "parallel"
aggregation_mode = "all"  # All must pass

[[guardrails.input.hybrid.providers]]
type = "patterns"

[[guardrails.input.hybrid.providers]]
type = "llama_guard"

[[guardrails.input.hybrid.providers]]
type = "gpt_oss_safeguard"
```

## Next Steps

Start with [Regex Guardrails]({{ site.baseurl }}{% link guardrails/regex.md %}) for basic validation, then explore [Composite Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) for defense-in-depth strategies.
