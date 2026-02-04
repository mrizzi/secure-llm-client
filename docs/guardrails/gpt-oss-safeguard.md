---
layout: default
title: GPT OSS Safeguard
parent: Guardrails
nav_order: 5
---

# GPT OSS Safeguard

Custom policy-based validation using GPT-4.

## Overview

GPT OSS Safeguard uses GPT-4 (or similar models) to validate inputs against custom policies defined in text files.

**Advantages**:
- Fully customizable policies
- Natural language policy definitions
- High accuracy (GPT-4 level)

**Disadvantages**:
- Slower (2-5 seconds)
- More expensive (GPT-4 API costs)

## Configuration

```toml
[guardrails.input]
type = "gpt_oss_safeguard"

[guardrails.input.gpt_oss_safeguard]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key_name = "OPENAI_API_KEY"
policy_file = "policies/my_policy.txt"  # Custom policy
max_tokens = 1000
timeout_secs = 120
```

## Policy Files

**Location**: `src/guardrails/policies/` (embedded at compile time)

### Built-in Policies

1. **default_policy.txt** - General safety (violence, hate speech, illegal content)
2. **strict_policy.txt** - Stricter rules (includes controversial topics)
3. **enterprise_policy.txt** - Corporate compliance (data privacy, IP protection)

### Custom Policy Example

`policies/my_policy.txt`:
```text
You are a safety validator. Evaluate the following user input and determine if it violates any of these policies:

1. No requests for illegal activities
2. No hate speech or discrimination
3. No personal data collection without consent
4. No medical advice (we are not licensed)
5. No financial advice (we are not qualified)

If the input violates any policy, respond with "UNSAFE: [policy number]".
If the input is safe, respond with "SAFE".

User input to evaluate:
{input}
```

**Variables**:
- `{input}` - User prompt to validate

## Usage

```bash
export OPENAI_API_KEY=sk-...
fortified-llm-client -c config.toml --user-text "Give me medical advice"
# Result: ValidationError (UNSAFE: policy 4 - medical advice)
```

## Creating Custom Policies

### Step 1: Write Policy File

Create `src/guardrails/policies/my_custom_policy.txt`:
```text
Evaluate this input against our company policies:
- No competitor mentions
- No profanity
- Professional tone required

Input: {input}

Respond: SAFE or UNSAFE: [reason]
```

### Step 2: Rebuild

```bash
cargo build --release
```

Policy is embedded at compile time.

### Step 3: Configure

```toml
[guardrails.input.gpt_oss_safeguard]
policy_file = "my_custom_policy.txt"  # Just the filename
```

## Best Practices

1. **Be specific in policies** - Clear rules get better results
2. **Test with edge cases** - Validate policy catches violations
3. **Set appropriate timeouts** - GPT-4 can be slow (120s recommended)
4. **Monitor costs** - Each validation is a GPT-4 API call

## See Also

- [Custom Policies]({{ site.baseurl }}{% link guardrails/custom-policies.md %}) - Policy file format
- [Hybrid Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) - Combine with other checks
