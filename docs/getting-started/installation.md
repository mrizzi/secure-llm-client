---
layout: default
title: Installation
parent: Getting Started
nav_order: 2
---

# Installation

Learn how to build and install Fortified LLM Client from source.

## Build from Source

### 1. Clone the Repository

```bash
git clone https://github.com/mrizzi/fortified-llm-client
cd fortified-llm-client
```

### 2. Build the Project

**Release build (recommended)**:
```bash
cargo build --release
```

The compiled binary will be at: `target/release/fortified-llm-client`

**Debug build (faster compilation, slower runtime)**:
```bash
cargo build
```

Debug binary location: `target/debug/fortified-llm-client`

### 3. (Optional) Install to PATH

Add the binary to your system PATH for easy access:

```bash
# Copy to a directory in your PATH
sudo cp target/release/fortified-llm-client /usr/local/bin/

# Or create a symlink
sudo ln -s $(pwd)/target/release/fortified-llm-client /usr/local/bin/fortified-llm-client

# Verify installation
fortified-llm-client --version
```

Alternatively, run from the project directory:
```bash
./target/release/fortified-llm-client --help
```

## Verify Installation

Test the CLI is working:

```bash
# Show help (requires at least one of api_url/model)
fortified-llm-client --help

# Test with Ollama (if running locally)
fortified-llm-client --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello, world!"
```

Expected output: JSON response with LLM content and metadata.

## Using as a Library

To use Fortified LLM Client as a Rust library in your project:

### Add to `Cargo.toml`

```toml
[dependencies]
fortified_llm_client = { git = "https://github.com/mrizzi/fortified-llm-client" }
tokio = { version = "1", features = ["full"] }
```

{: .note }
> The library is not yet published to crates.io. Use the git dependency until the first stable release.

### Import in Your Code

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};
```

See [Quick Start]({{ site.baseurl }}{% link getting-started/quick-start.md %}) for complete examples.

## Development Setup

If you plan to contribute or modify the code:

### 1. Install Nightly Toolchain

Required for code formatting:
```bash
rustup toolchain install nightly
```

### 2. Install Development Tools

```bash
# Clippy (linting)
rustup component add clippy

# Rustfmt (formatting)
rustup component add rustfmt --toolchain nightly
```

### 3. Run Pre-Commit Checks

Before committing changes:
```bash
# Format code
cargo +nightly fmt

# Check for errors
cargo check

# Run linter
cargo clippy

# Run tests (requires docling: pip install docling)
cargo test
```

**One-liner for all CI checks**:
```bash
cargo +nightly fmt --check && RUSTFLAGS="-D warnings" cargo check && cargo clippy -- -D warnings && RUSTFLAGS="-D warnings" cargo test
```

See [Contributing]({{ site.baseurl }}{% link contributing/ci-checklist.md %}) for detailed development guidelines.

## Troubleshooting

### Build Fails with Linker Errors

**Symptom**: `error: linking with 'cc' failed`

**Solution**: Install build tools:
- **Linux**: `sudo apt-get install build-essential`
- **macOS**: `xcode-select --install`
- **Windows**: Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

### Tests Fail with "docling not found"

**Symptom**: PDF extraction tests fail

**Solution**: Install Docling CLI:
```bash
pip install docling
```

Or skip PDF tests:
```bash
cargo test --lib
```

### Rust Version Too Old

**Symptom**: `error: package requires rustc 1.70 or newer`

**Solution**: Update Rust:
```bash
rustup update stable
```

## Next Steps

Now that you've installed Fortified LLM Client, try the [Quick Start]({{ site.baseurl }}{% link getting-started/quick-start.md %}) tutorial to run your first examples.
