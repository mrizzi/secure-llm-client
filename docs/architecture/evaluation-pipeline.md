---
layout: default
title: Evaluation Pipeline
parent: Architecture
nav_order: 2
---

# Evaluation Pipeline

Step-by-step execution flow for LLM evaluation.

## Overview

The evaluation pipeline in `lib.rs::evaluate_internal()` follows a strict sequence to ensure security, efficiency, and correctness.

## Pipeline Steps

### Step 1: PDF Extraction (Optional)

**When**: `pdf_input` is provided

**Process**:
1. Validate PDF file exists
2. Check file size ≤ MAX_PDF_SIZE_BYTES (50MB)
3. Create temporary directory
4. Execute Docling CLI: `docling <pdf> --output <temp> --format markdown`
5. Read extracted Markdown text
6. Replace `user_prompt` with extracted text
7. Clean up temporary files

**Code**: `src/pdf.rs::extract_pdf_text()`

**Error Handling**: Fails if Docling not installed or extraction fails

### Step 2: Input Guardrails (Optional)

**When**: Input guardrails configured in config file

**Process**:
1. Load guardrail configuration from config file
2. Create appropriate `GuardrailProvider` (patterns, llama_guard, hybrid, etc.)
3. Validate `user_prompt` (NOT system_prompt - system prompts are trusted)
4. If validation fails, return `ValidationError` immediately

**Code**: `src/guardrails/config.rs::create_guardrail_provider()`

**Important**: Only user-provided content is validated. System prompts are developer-controlled and trusted.

**Example**:
```rust
if let Some(input_guardrail) = &guardrails.input {
    input_guardrail.validate(&config.user_prompt).await?;
}
```

### Step 3: Token Validation (Optional)

**When**: `validate_tokens` is `true`

**Process**:
1. Determine context limit (auto-detect or use `context_limit` override)
2. Estimate system prompt tokens
3. Estimate user prompt tokens
4. Calculate response buffer (from `max_tokens` or default)
5. Total = system + user + response_buffer
6. If total > context_limit, return `ValidationError`

**Code**: `src/token_estimator.rs::estimate_tokens()`

**Benefit**: Fails early before API call, saving cost and latency

### Step 4: LLM Invocation

**Process**:
1. Detect provider from API URL (or use explicit `provider` override)
2. Create provider instance (`OpenAIProvider` or `OllamaProvider`)
3. Build `LlmRequest` with all parameters
4. Call `provider.invoke(request)`
5. Parse response and extract content

**Code**:
- `src/client.rs::LlmClient::call()`
- `src/providers/openai.rs::OpenAIProvider::invoke()`
- `src/providers/ollama.rs::OllamaProvider::invoke()`

**Error Handling**: Returns `ApiError` for network/auth failures

### Step 5: Output Guardrails (Optional)

**When**: Output guardrails configured in config file

**Process**:
1. Load output guardrail configuration
2. Create `GuardrailProvider`
3. Validate LLM response content
4. If validation fails, return `ValidationError`

**Code**: `src/guardrails/output.rs`

**Use Case**: Detect toxic content, low quality responses, policy violations

### Step 6: Metadata Generation

**Process**:
1. Calculate total latency (pipeline start to end)
2. Collect metadata:
   - Model name
   - Estimated tokens (from step 3 or post-hoc)
   - Latency in milliseconds
   - Timestamp (ISO 8601)
   - Provider type
   - Request parameters (temperature, max_tokens, etc.)
3. Create `EvaluationResult` with content + metadata

**Code**: `src/lib.rs::evaluate_internal()`

**Output**:
```rust
EvaluationResult {
    content: String,  // LLM response
    metadata: Metadata {
        model, tokens_estimated, latency_ms, timestamp, ...
    }
}
```

## Complete Flow Diagram

```
Input (EvaluationConfig)
    │
    ▼
┌─────────────────────────┐
│ PDF Extraction?         │  ← Step 1 (optional)
│ Extract text → replace  │
│ user_prompt             │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Input Guardrails?       │  ← Step 2 (optional)
│ Validate user_prompt    │
│ (NOT system_prompt)     │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Token Validation?       │  ← Step 3 (optional)
│ Estimate & check limit  │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ LLM Invocation          │  ← Step 4 (required)
│ Detect provider         │
│ Call API                │
│ Parse response          │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Output Guardrails?      │  ← Step 5 (optional)
│ Validate LLM response   │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Metadata Generation     │  ← Step 6 (required)
│ Collect stats, format   │
└───────────┬─────────────┘
            │
            ▼
Output (EvaluationResult)
```

## Error Handling

Each step can fail with specific error types:

| Step | Error Type | Example |
|------|------------|---------|
| PDF Extraction | `PdfError` | Docling not installed, file too large |
| Input Guardrails | `ValidationError` | PII detected, prompt injection |
| Token Validation | `ValidationError` | Token count exceeds limit |
| LLM Invocation | `ApiError` | Network failure, invalid API key |
| Output Guardrails | `ValidationError` | Toxic content detected |
| Metadata | `InternalError` | Timestamp formatting error |

**Pipeline behavior**: First error stops execution, returns immediately

## Performance Characteristics

**Typical latency breakdown**:
- PDF Extraction: 500ms - 5s (depends on PDF size/complexity)
- Input Guardrails: 10ms (patterns) to 2s (LLM-based)
- Token Validation: <10ms
- LLM Invocation: 1s - 30s (depends on model, response length)
- Output Guardrails: 10ms (patterns) to 2s (LLM-based)
- Metadata Generation: <1ms

**Optimization tips**:
- Use pattern-based guardrails before LLM-based (sequential hybrid)
- Enable token validation to fail fast for oversized prompts
- Cache PDF extractions if processing same file multiple times

## See Also

- [Layers]({{ site.baseurl }}{% link architecture/layers.md %}) - Architecture overview
- [Providers]({{ site.baseurl }}{% link architecture/providers.md %}) - LLM provider details
- [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %}) - Validation strategies
