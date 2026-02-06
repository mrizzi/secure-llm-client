---
layout: default
title: Regex Guardrails
parent: Guardrails
nav_order: 1
---

# Regex Guardrails

Fast pattern-based validation for both inputs and outputs.

## Overview

Regex guardrails provide:
- Custom pattern matching via regex
- Content length limits
- Configurable severity thresholds
- User-provided pattern files

**Speed**: <10ms (no LLM calls)
**Cost**: Free (local validation)
**Works for**: Both input and output validation

## Configuration

### Basic Configuration

```toml
[guardrails.input]
type = "regex"
max_length_bytes = 1048576  # 1MB
patterns_file = "patterns/input.txt"
severity_threshold = "medium"
```

### Unified Configuration (Same for Input and Output)

To apply the same regex guardrails to both input and output, use the flattened format:

```toml
[guardrails]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/common.txt"
severity_threshold = "medium"
```

This automatically applies to both input validation and output validation, reducing configuration duplication.

**When to use:**
- Same patterns apply to both input and output
- Consistent length limits for both directions
- Symmetric validation requirements

**Override example:**
```toml
# Base config applies to both
[guardrails]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/common.txt"

# Output allows longer content
[guardrails.output]
max_length_bytes = 2097152
```

### All Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_length_bytes` | `usize` | 1048576 (1MB) | Maximum content length in bytes |
| `patterns_file` | `Option<PathBuf>` | None | Path to custom patterns file |
| `severity_threshold` | `Severity` | Medium | Minimum severity to report (violations below this become warnings) |

### Severity Levels

- `Critical` - Severe violations (always reported)
- `High` - Important violations
- `Medium` - Moderate violations (default threshold)
- `Low` - Minor issues (often treated as warnings)

## Pattern File Format

Pattern files use a simple format:

```
SEVERITY | Description | Regex Pattern
```

**Example** (`patterns/input.txt`):

```
CRITICAL | Social Security Number | \b\d{3}-\d{2}-\d{4}\b
HIGH | Credit Card Number | \b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b
MEDIUM | Email Address | \b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b
LOW | Phone Number | \b\d{3}[-.]?\d{3}[-.]?\d{4}\b
```

## Usage Examples

### Input Validation

```toml
[guardrails.input]
type = "regex"
max_length_bytes = 1048576
patterns_file = "patterns/input.txt"
severity_threshold = "medium"
```

**Input patterns** might include:
- PII detection (SSN, credit cards, emails)
- Prompt injection patterns
- Sensitive data patterns

### Output Validation

```toml
[guardrails.output]
type = "regex"
max_length_bytes = 2097152  # 2MB (larger for responses)
patterns_file = "patterns/output.txt"
severity_threshold = "high"  # Stricter threshold
```

**Output patterns** might include:
- Toxic language detection
- Dangerous instructions
- Inappropriate content

### Library Usage

```rust
use fortified_llm_client::{
    evaluate, ConfigBuilder,
    guardrails::{GuardrailProviderConfig, RegexGuardrailConfig, Severity},
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_guardrails = GuardrailProviderConfig::Regex(
        RegexGuardrailConfig {
            max_length_bytes: 1048576,
            patterns_file: Some(PathBuf::from("patterns/input.txt")),
            severity_threshold: Severity::Medium,
        }
    );

    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .user_prompt("Your prompt here")
        .input_guardrails(input_guardrails)
        .build()?;

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);

    Ok(())
}
```

## Validation Behavior

### Length Validation

If content exceeds `max_length_bytes`:
- **Severity**: `High`
- **Rule**: `MAX_LENGTH`
- **Message**: "Content exceeds max length (X > Y bytes)"

### Pattern Matching

For each pattern in the patterns file:
1. Apply regex to content
2. If match found:
   - Compare pattern severity to `severity_threshold`
   - If >= threshold: Add to violations (validation fails)
   - If < threshold: Add to warnings (validation passes)

### Example: Severity Threshold

With `severity_threshold = "medium"`:

```
# These patterns trigger violations (>= medium)
CRITICAL | SSN | ...         → Violation
HIGH | Credit Card | ...      → Violation
MEDIUM | Email | ...          → Violation

# This pattern triggers warning (< medium)
LOW | Phone Number | ...      → Warning
```

## Best Practices

### Pattern Design

1. **Be specific**: Avoid overly broad patterns that cause false positives
2. **Test thoroughly**: Validate patterns against sample data
3. **Use anchors**: `\b` word boundaries prevent partial matches
4. **Escape special chars**: Use `\` to escape regex metacharacters

### Severity Assignment

- **Critical**: Data breaches, severe security violations
- **High**: Important violations, PII exposure
- **Medium**: General violations, potentially sensitive data
- **Low**: Minor issues, informational warnings

### Performance Tips

- Keep pattern files small (< 100 patterns)
- Use simple patterns when possible (avoid complex lookaheads)
- Consider pattern ordering (most common first)

## See Also

- [Composite Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) - Combine regex with LLM-based validation
- [Llama Guard]({{ site.baseurl }}{% link guardrails/llama-guard.md %}) - More comprehensive safety validation
- [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) - Full configuration reference
