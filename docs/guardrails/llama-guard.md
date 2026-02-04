---
layout: default
title: Llama Guard
parent: Guardrails
nav_order: 3
---

# Llama Guard

MLCommons safety taxonomy validation (13 categories S1-S13).

## Overview

Llama Guard uses a dedicated LLM to classify inputs against 13 safety categories defined by MLCommons.

## Safety Categories

| Code | Category | Example |
|------|----------|---------|
| S1 | Violent Crimes | Murder, assault instructions |
| S2 | Non-Violent Crimes | Fraud, theft instructions |
| S3 | Sex-Related Crimes | Human trafficking, sexual abuse |
| S4 | Child Sexual Exploitation | CSAM, grooming |
| S5 | Defamation | Libel, slander |
| S6 | Specialized Advice | Unqualified medical/legal advice |
| S7 | Privacy | Unauthorized PII requests |
| S8 | Intellectual Property | Copyright violation |
| S9 | Indiscriminate Weapons | Bioweapons, explosives |
| S10 | Hate | Discrimination, harassment |
| S11 | Suicide & Self-Harm | Encouragement, instructions |
| S12 | Sexual Content | Explicit adult content |
| S13 | Elections | Voter suppression, fraud |

## Configuration

### All Categories (Default)

```toml
[guardrails.input]
type = "llama_guard"

[guardrails.input.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
max_tokens = 512
timeout_secs = 60
```

### Specific Categories

```toml
[guardrails.input.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
enabled_categories = ["S1", "S2", "S3", "S4", "S10", "S11"]  # Focus on critical
```

## Prerequisites

Install Llama Guard model:

```bash
ollama pull llama-guard-3
```

## Usage

```bash
fortified-llm-client -c config.toml --user-text "How do I hack into a system?"
# Result: ValidationError (S2: Non-Violent Crimes detected)
```

## Performance

- **Latency**: 1-3 seconds per validation
- **Cost**: Free with Ollama, or API cost with hosted models
- **Accuracy**: High (trained specifically for safety classification)

## Best Practices

1. **Use in hybrid mode** with patterns first (sequential)
2. **Select relevant categories** - Not all apps need all 13
3. **Cache results** for repeated prompts
4. **Set timeouts** to prevent hanging (60s recommended)

## See Also

- [Llama Prompt Guard]({{ site.baseurl }}{% link guardrails/llama-prompt-guard.md %}) - Jailbreak detection
- [Hybrid Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) - Combine with patterns
