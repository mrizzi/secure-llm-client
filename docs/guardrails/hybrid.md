---
layout: default
title: Hybrid Guardrails
parent: Guardrails
nav_order: 6
---

# Hybrid Guardrails

Combine multiple guardrail providers with configurable execution strategies.

## Overview

Hybrid guardrails enable defense-in-depth by layering multiple validation strategies.

## Execution Modes

### Sequential (Recommended)

Run providers one by one, stop on first failure:

```toml
[guardrails.input]
type = "composite"
execution = "sequential"
aggregation = "all_must_pass"

# Fast check first
[[guardrails.input.providers]]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/input.txt"
severity_threshold = "medium"

# Expensive check only if regex passes
[[guardrails.input.providers]]
type = "llama_guard"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard3:8b"
timeout_secs = 30
enabled_categories = ["S1", "S2", "S3", "S4", "S5"]
```

**Benefits**: Faster (stops early), lower cost

### Parallel

Run all providers concurrently:

```toml
[guardrails.input]
type = "composite"
execution = "parallel"
aggregation = "all_must_pass"

[[guardrails.input.hybrid.providers]]
type = "patterns"

[[guardrails.input.hybrid.providers]]
type = "llama_guard"

[[guardrails.input.hybrid.providers]]
type = "gpt_oss_safeguard"
```

**Benefits**: Consistent latency, full coverage

## Aggregation Modes

### All (Strict)

Pass only if ALL providers pass:

```toml
aggregation = "all_must_pass"
```

**Use case**: High-security environments

### Any (Permissive)

Pass if ANY provider passes:

```toml
aggregation = "any_can_pass"
```

**Use case**: Development, avoid false positives

{: .note }
> **Majority mode** is not currently implemented. Use `all_must_pass` (conservative) or `any_can_pass` (permissive).

## Complete Example

Defense-in-depth with three layers:

```toml
[guardrails.input]
type = "composite"

[guardrails.input]
type = "composite"
execution = "sequential"
aggregation = "all_must_pass"

# Layer 1: Fast regex checks (< 10ms)
[[guardrails.input.providers]]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/input.txt"
severity_threshold = "medium"

# Layer 2: Prompt injection detection (1-3s)
[[guardrails.input.providers]]
type = "llama_prompt_guard"
api_url = "http://localhost:11434/v1/chat/completions"
model = "prompt-guard-86m"
timeout_secs = 30

# Layer 3: Safety taxonomy (1-3s)
[[guardrails.input.providers]]
type = "llama_guard"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard3:8b"
timeout_secs = 30
enabled_categories = ["S1", "S2", "S3", "S4", "S10"]
```

## Performance Considerations

| Mode | Latency | Cost | Coverage |
|------|---------|------|----------|
| Sequential + all_must_pass | Fast on failure | Low | Complete |
| Sequential + any_can_pass | Fast on success | Low | Partial |
| Parallel + all_must_pass | Consistent | High | Complete |
| Parallel + any_can_pass | Consistent | High | Partial |

## Best Practices

1. **Put fast checks first** in sequential mode
2. **Use all_must_pass for production** - conservative, safer for security
3. **Test all providers** independently before combining
4. **Monitor latency** - parallel can hide slow providers

## See Also

- [Regex Guardrails]({{ site.baseurl }}{% link guardrails/regex.md %})
- [Llama Guard]({{ site.baseurl }}{% link guardrails/llama-guard.md %})
- [GPT OSS Safeguard]({{ site.baseurl }}{% link guardrails/gpt-oss-safeguard.md %})
