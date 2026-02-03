# secure-llm-client

A Rust library and CLI tool for secure interaction with Large Language Model (LLM) providers, featuring built-in security guardrails, PDF extraction, and multi-provider support.

> [!WARNING]
> **Active Development**: This project is currently under active development.  
> While we strive for stability, please note:
> - The library API may change between versions
> - Issues and bugs are expected - please report them!
> - Contributions and feedback are welcome and appreciated
> - Not recommended for production use without thorough testing

## Features

**Multi-Provider Support**
- OpenAI-compatible APIs
- Ollama local models
- Automatic provider detection from API URL
- Unified interface via `LlmProvider` trait

**Security Guardrails**
- Input validation (pattern matching, PII detection, prompt injection detection)
- Output validation with quality scoring
- LLM-based guardrails:
  - Llama Guard (MLCommons safety taxonomy, categories S1-S13)
  - Llama Prompt Guard (jailbreak detection)
  - GPT OSS Safeguard (GPT-4 based policy validation)
- Hybrid guardrails with configurable execution modes (sequential/parallel) and aggregation strategies (any/all/majority)
- Customizable pattern-based rules via config files

**PDF Processing**
- Extract text from PDFs using external Docling CLI
- File size validation for resource protection
- Direct integration into LLM prompts

**Token Management**
- Model-specific token estimation
- Context limit validation
- Per-request token budget control

**Response Formatting**
- Plain text output (default)
- JSON object generation
- JSON Schema validation for structured responses

**Configuration**
- CLI arguments with full control
- TOML/JSON configuration files
- Environment variable support for API keys
- Figment-based config merging (CLI args override file values)

**Safety & Reliability**
- Request timeout protection (configurable, default 300s)
- Input length limits
- Atomic file writes for output
- Comprehensive error handling

## Installation

### Prerequisites

- Rust 1.70 or later
- (Optional) Docling CLI for PDF extraction

### Build from Source

```bash
# Clone the repository
git clone https://github.com/mrizzi/secure-llm-client
cd secure-llm-client

# Build the CLI and library
cargo build --release

# Binary will be at target/release/secure-llm-client
```

## Usage

### CLI

```bash
# Basic usage
secure-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Explain Rust ownership"

# With guardrails config file
secure-llm-client --config config.toml --user-text "Your prompt here"

# PDF extraction with LLM analysis
secure-llm-client --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --pdf-input document.pdf \
  --system-text "Summarize the key points from this document"

# JSON Schema validation
secure-llm-client --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --user-text "Generate a product catalog" \
  --response-format json-schema \
  --response-format-schema-strict schema.json

# Output to file
secure-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello" \
  --output response.json
```

### Library API

```rust
use secure_llm_client::{evaluate, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: "Explain Rust ownership".to_string(),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);
    Ok(())
}
```

### Configuration File

Create a `config.toml` file:

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7
max_tokens = 2000
validate_tokens = true

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

## Architecture

The codebase follows a layered architecture:

1. **CLI Layer**: Argument parsing and config merging
2. **Library Layer**: Public API (`evaluate()`, `evaluate_with_guardrails()`)
3. **Client Layer**: Provider-agnostic LLM client abstraction
4. **Provider Layer**: Provider-specific implementations
5. **Guardrails Layer**: Security validation pipeline

Evaluation pipeline: PDF Extraction → Input Guardrails → Token Validation → LLM Invocation → Output Guardrails → Metadata Generation

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture
```

## License

Apache-2.0

## Contributing

Contributions are welcome! To ensure smooth collaboration, please follow these steps:

1. **File an Issue First**: Before starting work, create an issue describing:
   - The bug you want to fix, or
   - The feature you want to add, or
   - The improvement you want to make

2. **Wait for Discussion**: Allow maintainers and community members to discuss the proposal. This helps avoid duplicate work and ensures alignment with project goals.

3. **Open a Pull Request**: Once the issue is discussed and approved, you can:
   - Fork the repository
   - Create a feature branch
   - Make your changes
   - Ensure all tests pass: `cargo test`
   - Follow Rust conventions: `cargo +nightly fmt` and `cargo clippy`
   - Open a PR referencing the issue number

**Note**: Pull requests without a corresponding issue may be closed. This process helps maintain code quality and project direction.

**Why `+nightly`?** The `cargo +nightly fmt` command is required because this project uses the `imports_granularity` feature in `rustfmt.toml`, which is only available in Rust nightly (see [tracking issue #4991](https://github.com/rust-lang/rustfmt/issues/4991)). Make sure you have the nightly toolchain installed: `rustup toolchain install nightly`.
