# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`secure-llm-client` is a Rust library and CLI tool for interacting with LLM providers (OpenAI, Ollama) with built-in security guardrails, PDF extraction, and multi-provider support. It provides both a library API (`secure_llm_client`) and a CLI binary (`secure-llm-client`).

## Common Commands

### Building
```bash
# Build library and CLI
cargo build

# Build release version
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests in a specific file
cargo test --test <test_file_name>

# Run with verbose output
cargo test -- --nocapture
```

### Running the CLI
```bash
# Show help (required: at least one of api_url/model must be provided)
cargo run -- --help

# Example: simple LLM call
cargo run -- --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello"

# With config file (CLI args override config file values)
cargo run -- -c config.toml --user-text "Override prompt"
```

### Linting
```bash
# Check code without building
cargo check

# Run clippy for linting
cargo clippy

# Format code
cargo +nightly fmt
```

### Pre-Push Checklist

**Run these commands locally before pushing to ensure CI will pass:**

```bash
# 1. Check formatting (CI fails if not formatted)
cargo +nightly fmt --check

# 2. Verify code compiles (warnings treated as errors)
RUSTFLAGS="-D warnings" cargo check

# 3. Run clippy with warnings as errors
cargo clippy -- -D warnings

# 4. Run all tests with warnings as errors (requires docling: pip install docling)
RUSTFLAGS="-D warnings" cargo test
```

**Quick one-liner to run all CI checks:**
```bash
cargo +nightly fmt --check && RUSTFLAGS="-D warnings" cargo check && cargo clippy -- -D warnings && RUSTFLAGS="-D warnings" cargo test
```

## Architecture

### Layered Design

The codebase follows a **layered architecture** with separation of concerns:

1. **CLI Layer** (`main.rs`, `cli/*`): Handles argument parsing, config merging (Figment), and output formatting
2. **Library Layer** (`lib.rs`): Exposes public API via `evaluate()` and `evaluate_with_guardrails()`
3. **Client Layer** (`client.rs`): Provider-agnostic LLM client abstraction
4. **Provider Layer** (`providers/*`): Provider-specific implementations (OpenAI, Ollama)
5. **Guardrails Layer** (`guardrails/*`): Security validation pipeline

### Evaluation Pipeline

The core evaluation flow in `lib.rs::evaluate_internal()` follows this sequence:

1. **PDF Extraction** (if `pdf_input` provided) → extracts text from PDF using external Docling CLI
2. **Input Guardrails** → validates user input (NOT system prompt - system prompts are trusted)
3. **Token Validation** → estimates tokens and checks against context limits
4. **LLM Invocation** → calls provider API
5. **Output Guardrails** (if enabled) → validates LLM response
6. **Metadata Generation** → creates structured output with execution metadata

### Guardrails System

Guardrails are **composable** and support multiple execution strategies:

- **Input Guardrails** (`guardrails/input.rs`): Pattern-based validation (regex), PII detection, prompt injection detection
- **Output Guardrails** (`guardrails/output.rs`): Response validation, quality scoring
- **LLM-Based Guardrails**:
  - `llama_guard.rs`: MLCommons safety taxonomy (13 categories: S1-S13)
  - `llama_prompt_guard.rs`: Jailbreak detection
  - `gpt_oss_safeguard.rs`: GPT-4 based safety validation with custom policies
- **Hybrid Guardrails** (`guardrails/hybrid.rs`): Combines multiple guardrail providers with configurable execution modes:
  - `ExecutionMode::Sequential` - Run providers one by one, stop on first failure
  - `ExecutionMode::Parallel` - Run all providers concurrently
  - `AggregationMode::Any` - Pass if ANY provider passes
  - `AggregationMode::All` - Pass only if ALL providers pass
  - `AggregationMode::Majority` - Pass if majority passes

**Key Design Principle**: Input guardrails ONLY validate user-provided content (user_prompt), NOT system prompts, since system prompts are developer-controlled trusted content.

### Provider System

Provider detection and selection (`providers/detection.rs`):

1. **Auto-detection**: Analyzes API URL patterns to infer provider type
2. **Explicit Override**: `--provider` flag forces specific provider format
3. **Fallback**: Defaults to OpenAI-compatible format if detection fails

Both providers share the `LlmProvider` trait (`provider.rs`) with a unified `invoke()` method.

### Configuration System

**Dual configuration approach**:

1. **Figment Merging** (`main.rs::merge_config()`): Handles scalar fields (api_url, model, temperature, etc.) with priority: CLI args > Config file
2. **ConfigFileRequest** (`config.rs`): Parses complex nested structures like guardrails configuration from TOML/JSON files

**Why dual loading?** Figment elegantly handles flat fields, but guardrail configuration requires nested structures not in the CLI `Args` struct. The `ConfigBuilder` (`config_builder.rs`) unifies both approaches.

### Policy Files

Guardrail policies are **embedded at compile time** using `include_str!()`:

- Location: `src/guardrails/policies/*.txt`
- Rebuild required after editing: `cargo build --release`
- See `src/guardrails/policies/README.md` for policy structure requirements

### Token Estimation

Model-specific token estimation (`token_estimator.rs`, `model_registry.rs`):

- Uses model name to select appropriate tokenizer estimation
- Calculates: system tokens + user tokens + response buffer
- Validates against context limits if `--validate-tokens` enabled

### PDF Processing

External tool integration (`pdf.rs`):

- **Primary**: Docling CLI (`docling` command) - converts PDF to Markdown/text
- **Fallback**: Not implemented (Docling required)
- **Security**: File size validation before extraction (see `constants::pdf_limits::MAX_PDF_SIZE_BYTES`)

## Configuration Files

Config files (`.toml` or `.json`) support all CLI arguments plus guardrails:

```toml
# Example: config.toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7

[guardrails.input]
type = "patterns"
max_length_bytes = 1048576

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = true
```

**Merge behavior**: CLI arguments override config file values. See `main.rs::merge_config()` and `config_builder.rs::ConfigBuilder::merge_file_config()`.

## Testing Strategy

Tests are organized by purpose:

- **Unit tests**: `tests/unit_tests.rs` - Core functionality tests
- **Integration tests**: `tests/*_test.rs` - End-to-end workflows
- **Provider tests**: `tests/provider_tests.rs` - Provider-specific behavior
- **Config tests**: `tests/config_*.rs` - Configuration merging and validation
- **Guardrail tests**: `tests/guardrail*.rs` - Guardrail validation logic

**Test fixtures**: Located in `tests/fixtures/` (PDF samples, JSON schemas, markdown files)

## Key Security Features

1. **Input validation limits**: Max input length, max input tokens (configurable)
2. **PDF size validation**: Prevents resource exhaustion (see `constants::pdf_limits`)
3. **Timeout protection**: Request timeouts prevent hanging (default: 300s)
4. **API key handling**: Supports environment variables via `--api-key-name` or config file `api_key_name`
5. **Atomic file writes**: Output file writes use temp file + rename for atomicity

## Important Implementation Notes

### Adding New CLI Arguments

When adding `#[serde(skip)]` fields to `Args` struct in `main.rs`:

1. Add the field to the `Args` struct with `#[serde(skip)]` or `#[serde(skip, default)]`
2. Update `main.rs::merge_config()` restoration block (see CRITICAL CHECKLIST comment)
3. Update `Args::default()` implementation
4. CLI-only fields should NOT appear in config files

### Modifying Guardrails

1. Implement `GuardrailProvider` trait (`guardrails/provider.rs`)
2. Add provider to `guardrails/config.rs::create_guardrail_provider()`
3. Add configuration struct (derive `Serialize`, `Deserialize`)
4. For LLM-based guardrails, ensure proper error handling for API failures

### Provider Response Parsing

Both providers parse JSON responses from LLM APIs:

- **OpenAI**: Expects `{"choices": [{"message": {"content": "..."}}]}`
- **Ollama**: Compatible with OpenAI format
- Error handling in `providers/openai.rs` and `providers/ollama.rs`

### Response Format Feature

The `--response-format` flag (OpenAI-compatible only):

- `text`: Plain text (default)
- `json-object`: Structured JSON output
- `json-schema`: Validates against provided JSON Schema file
- Schema validation uses `jsonschema` crate (`schema_validator.rs`)

## Dependencies

Key external dependencies:

- **HTTP**: `reqwest` (with `rustls-tls` feature)
- **Async**: `tokio`, `async-trait`
- **CLI**: `clap` (derive API)
- **Config**: `figment` (TOML/JSON merging)
- **Serialization**: `serde`, `serde_json`, `toml`
- **Validation**: `regex` (pattern matching), `jsonschema` (schema validation)
- **Testing**: `mockito` (HTTP mocking), `assert_cmd` (CLI testing), `proptest` (property-based testing)

## Output Format

CLI output is always JSON (`CliOutput` in `output.rs`):

```json
{
  "status": "success" | "error",
  "content": "...",
  "metadata": {
    "model": "...",
    "tokens_estimated": 123,
    "latency_ms": 456,
    "timestamp": "2025-01-30T12:00:00Z",
    ...
  },
  "error": { "code": "...", "message": "..." }  // Only if status=error
}
```

Use `--output` flag to write to file instead of stdout.
