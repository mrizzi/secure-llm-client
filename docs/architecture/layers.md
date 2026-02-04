---
layout: default
title: Layers
parent: Architecture
nav_order: 1
---

# 5-Layer Architecture

Fortified LLM Client is organized into five distinct layers, each with specific responsibilities.

## Layer 1: CLI Layer

**Location**: `src/main.rs`, `src/cli/`

**Responsibilities**:
- Parse command-line arguments (clap)
- Load and merge configuration files (Figment)
- Validate CLI inputs
- Format and write output
- Handle logging configuration

**Key Files**:
- `main.rs:20-162` - `Args` struct with all CLI flags
- `cli/mod.rs` - CLI utilities (validation, output formatting)

**Data Flow**:
```
CLI Args → Figment merge → EvaluationConfig → Library Layer
```

## Layer 2: Library Layer

**Location**: `src/lib.rs`

**Responsibilities**:
- Expose public API (`evaluate()`, `evaluate_with_guardrails()`)
- Coordinate evaluation pipeline
- Manage error handling and result formatting
- Orchestrate PDF extraction, guardrails, and LLM invocation

**Public API**:
```rust
pub async fn evaluate(config: EvaluationConfig) -> Result<EvaluationResult, FortifiedError>
pub async fn evaluate_with_guardrails(config: EvaluationConfig, config_file_path: &str) -> Result<EvaluationResult, FortifiedError>
```

**Key Function**: `evaluate_internal()` - Core evaluation pipeline

## Layer 3: Client Layer

**Location**: `src/client.rs`

**Responsibilities**:
- Provider-agnostic LLM client abstraction
- Provider detection from API URL
- Request preparation and response parsing
- Token estimation coordination

**Key Abstractions**:
```rust
pub struct LlmClient {
    provider: Box<dyn LlmProvider>,
    // ...
}
```

## Layer 4: Provider Layer

**Location**: `src/providers/`

**Responsibilities**:
- Provider-specific API implementations
- Request/response formatting
- Authentication handling
- API-specific error handling

**Files**:
- `providers/provider.rs` - `LlmProvider` trait
- `providers/openai.rs` - OpenAI implementation
- `providers/ollama.rs` - Ollama implementation
- `providers/detection.rs` - Auto-detection logic

**Trait**:
```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError>;
}
```

## Layer 5: Guardrails Layer

**Location**: `src/guardrails/`

**Responsibilities**:
- Input/output validation
- Pattern-based checks (PII, prompt injection)
- LLM-based validation (Llama Guard, GPT OSS Safeguard)
- Hybrid execution strategies

**Files**:
- `guardrails/provider.rs` - `GuardrailProvider` trait
- `guardrails/input.rs` - Input pattern validation
- `guardrails/output.rs` - Output pattern validation
- `guardrails/llama_guard.rs` - Llama Guard (S1-S13)
- `guardrails/llama_prompt_guard.rs` - Prompt injection detection
- `guardrails/gpt_oss_safeguard.rs` - GPT-4 policy validation
- `guardrails/hybrid.rs` - Composable guardrails

**Trait**:
```rust
#[async_trait]
pub trait GuardrailProvider: Send + Sync {
    async fn validate(&self, input: &str) -> Result<(), FortifiedError>;
}
```

## Layer Interactions

```
┌─────────────────────────────────────────────────────────────┐
│ CLI Layer (main.rs)                                         │
│ - Parse args, merge config, format output                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ Library Layer (lib.rs)                                      │
│ - evaluate(), evaluate_with_guardrails()                    │
│ - Orchestrate pipeline                                      │
└───┬───────────────────────┬───────────────────┬─────────────┘
    │                       │                   │
    ▼                       ▼                   ▼
┌─────────────┐    ┌────────────────┐    ┌──────────────────┐
│ PDF         │    │ Guardrails     │    │ Client Layer     │
│ Extraction  │    │ Layer          │    │ (client.rs)      │
│ (pdf.rs)    │    │ (guardrails/*) │    │ - Provider mgmt  │
└─────────────┘    └────────────────┘    └────────┬─────────┘
                                                   │
                                                   ▼
                                         ┌──────────────────┐
                                         │ Provider Layer   │
                                         │ (providers/*)    │
                                         │ - OpenAI/Ollama  │
                                         └──────────────────┘
```

## Dependency Rules

- **Top-down dependencies only** - Higher layers depend on lower layers
- **No circular dependencies** - Clear separation prevents cycles
- **Interface abstraction** - Layers interact via traits/interfaces
- **Minimal coupling** - Each layer is independently testable

## See Also

- [Evaluation Pipeline]({{ site.baseurl }}{% link architecture/evaluation-pipeline.md %}) - Detailed execution flow
- [Providers]({{ site.baseurl }}{% link architecture/providers.md %}) - Provider implementation details
- [Testing]({{ site.baseurl }}{% link architecture/testing.md %}) - Testing strategy per layer
