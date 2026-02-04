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

1. **Input Patterns** - Fast regex-based validation (PII, prompt injection)
2. **Output Patterns** - Response quality and safety checks
3. **Llama Guard** - MLCommons safety taxonomy (13 categories S1-S13)
4. **Llama Prompt Guard** - Jailbreak detection
5. **GPT OSS Safeguard** - GPT-4 based policy validation
6. **Hybrid** - Composable multi-provider validation

## Key Concepts

### Defense in Depth

Layer multiple guardrails for comprehensive protection:

```toml
[guardrails.input]
type = "hybrid"

# Layer 1: Fast pattern checks
[[guardrails.input.hybrid.providers]]
type = "patterns"

# Layer 2: LLM-based validation
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
```

### System Prompts Are Trusted

**Important**: Guardrails ONLY validate user-provided inputs, NOT system prompts.

- **System prompts** - Developer-controlled, trusted content
- **User prompts** - User-provided, must be validated

### Input vs Output Guardrails

- **Input Guardrails** - Validate before sending to LLM (prevents harmful inputs)
- **Output Guardrails** - Validate LLM responses (ensures safe outputs)

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
type = "patterns"
max_length_bytes = 1048576

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = true

[guardrails.output]
type = "patterns"

[guardrails.output.patterns]
detect_toxic = true
```

## Guardrail Types

| Type | Speed | Accuracy | Use Case |
|------|-------|----------|----------|
| **Input Patterns** | Fast (<10ms) | Good | PII, basic prompt injection |
| **Output Patterns** | Fast (<10ms) | Good | Toxic content, quality checks |
| **Llama Guard** | Slow (1-3s) | Excellent | Comprehensive safety (S1-S13) |
| **Llama Prompt Guard** | Slow (1-3s) | Excellent | Advanced jailbreak detection |
| **GPT OSS Safeguard** | Slow (2-5s) | Excellent | Custom policy validation |
| **Hybrid** | Variable | Best | Combine multiple strategies |

## Section Contents

- **[Input Patterns]({{ site.baseurl }}{% link guardrails/input-patterns.md %})** - Regex-based input validation
- **[Output Patterns]({{ site.baseurl }}{% link guardrails/output-patterns.md %})** - Response quality checks
- **[Llama Guard]({{ site.baseurl }}{% link guardrails/llama-guard.md %})** - MLCommons safety taxonomy
- **[Llama Prompt Guard]({{ site.baseurl }}{% link guardrails/llama-prompt-guard.md %})** - Jailbreak detection
- **[GPT OSS Safeguard]({{ site.baseurl }}{% link guardrails/gpt-oss-safeguard.md %})** - Policy-based validation
- **[Hybrid Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %})** - Multi-provider strategies
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

Start with [Input Patterns]({{ site.baseurl }}{% link guardrails/input-patterns.md %}) for basic validation, then explore [Hybrid Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) for defense-in-depth strategies.
