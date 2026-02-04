---
layout: default
title: Custom Policies
parent: Guardrails
nav_order: 7
---

# Custom Policies

Create custom policy files for GPT OSS Safeguard.

## Policy File Format

**Location**: `src/guardrails/policies/`

**Format**: Plain text with `{input}` placeholder

## Example Policy

`src/guardrails/policies/my_policy.txt`:
```text
You are a content safety validator for a family-friendly app.

Evaluate this user input against our policies:
1. No violence or graphic content
2. No profanity or offensive language
3. No adult content
4. No religious or political controversy
5. Child-appropriate topics only

User input:
{input}

Response format:
- If SAFE: respond with "SAFE"
- If UNSAFE: respond with "UNSAFE: [specific policy violated]"
```

## Variables

- `{input}` - Replaced with user prompt at runtime

## Policy Guidelines

### 1. Be Explicit

**Good**:
```text
No requests for:
- Illegal activities (hacking, drugs, weapons)
- Violence instructions
- Hate speech targeting race, religion, gender
```

**Bad**:
```text
Nothing bad or harmful
```

### 2. Define Response Format

```text
Response must be EXACTLY one of:
- "SAFE"
- "UNSAFE: [reason]"

Do not explain or elaborate.
```

### 3. Provide Examples (Optional)

```text
Examples:
- "How do I bake a cake?" → SAFE
- "How do I pick a lock?" → UNSAFE: illegal activity
```

## Using Custom Policies

### Step 1: Create Policy File

Add to `src/guardrails/policies/company_policy.txt`

### Step 2: Rebuild

```bash
cargo build --release
```

Policies are embedded at compile time via `include_str!()`.

### Step 3: Configure

```toml
[guardrails.input.gpt_oss_safeguard]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
policy_file = "company_policy.txt"
```

## Built-in Policies

See `src/guardrails/policies/README.md` for:
- `default_policy.txt` - General safety
- `strict_policy.txt` - More restrictive
- `enterprise_policy.txt` - Corporate compliance

## Testing Policies

Test with known safe/unsafe inputs:

```bash
# Should PASS
fortified-llm-client -c config.toml --user-text "What's the weather?"

# Should FAIL
fortified-llm-client -c config.toml --user-text "How do I hack a website?"
```

## See Also

- [GPT OSS Safeguard]({{ site.baseurl }}{% link guardrails/gpt-oss-safeguard.md %})
- Policy README: `src/guardrails/policies/README.md`
