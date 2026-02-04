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
type = "hybrid"

[guardrails.input.hybrid]
execution_mode = "sequential"
aggregation_mode = "all"

# Fast check first
[[guardrails.input.hybrid.providers]]
type = "patterns"
[guardrails.input.hybrid.providers.patterns]
detect_pii = true
detect_prompt_injection = true

# Expensive check only if patterns pass
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
[guardrails.input.hybrid.providers.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
```

**Benefits**: Faster (stops early), lower cost

### Parallel

Run all providers concurrently:

```toml
[guardrails.input.hybrid]
execution_mode = "parallel"
aggregation_mode = "majority"

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
aggregation_mode = "all"
```

**Use case**: High-security environments

### Any (Permissive)

Pass if ANY provider passes:

```toml
aggregation_mode = "any"
```

**Use case**: Development, avoid false positives

### Majority (Balanced)

Pass if majority passes:

```toml
aggregation_mode = "majority"
```

**Use case**: Production balance between security and usability

## Complete Example

Defense-in-depth with three layers:

```toml
[guardrails.input]
type = "hybrid"

[guardrails.input.hybrid]
execution_mode = "sequential"
aggregation_mode = "all"

# Layer 1: Fast patterns (< 10ms)
[[guardrails.input.hybrid.providers]]
type = "patterns"
[guardrails.input.hybrid.providers.patterns]
detect_pii = true
detect_prompt_injection = true
max_length_bytes = 1048576

# Layer 2: Prompt injection (1-3s)
[[guardrails.input.hybrid.providers]]
type = "llama_prompt_guard"
[guardrails.input.hybrid.providers.llama_prompt_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"

# Layer 3: Safety taxonomy (1-3s)
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
[guardrails.input.hybrid.providers.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
enabled_categories = ["S1", "S2", "S3", "S4", "S10"]
```

## Performance Considerations

| Mode | Latency | Cost | Coverage |
|------|---------|------|----------|
| Sequential + all | Fast on failure | Low | Complete |
| Sequential + majority | Medium | Medium | Complete |
| Parallel + all | Consistent | High | Complete |
| Parallel + any | Consistent | High | Partial |

## Best Practices

1. **Put fast checks first** in sequential mode
2. **Use majority for production** - balances false positives
3. **Test all providers** independently before combining
4. **Monitor latency** - parallel can hide slow providers

## See Also

- [Input Patterns]({{ site.baseurl }}{% link guardrails/input-patterns.md %})
- [Llama Guard]({{ site.baseurl }}{% link guardrails/llama-guard.md %})
- [GPT OSS Safeguard]({{ site.baseurl }}{% link guardrails/gpt-oss-safeguard.md %})
