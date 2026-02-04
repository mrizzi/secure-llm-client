---
layout: default
title: Prerequisites
parent: Getting Started
nav_order: 1
---

# Prerequisites

Before installing Fortified LLM Client, ensure your system meets these requirements.

## Required

### Rust 1.70 or Later

**Check if installed**:
```bash
rustc --version
# Should show 1.70.0 or higher
```

**Install Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Visit [rustup.rs](https://rustup.rs/) for alternative installation methods.

### Rust Nightly (for development/formatting)

The project uses nightly rustfmt for code formatting:

```bash
rustup toolchain install nightly
```

This is **only required** if you plan to contribute code or run `cargo +nightly fmt`.

## Optional

### Docling CLI (for PDF extraction)

Required **only if** you plan to use PDF extraction features (`--pdf-input` flag).

**Install via pip**:
```bash
pip install docling
```

**Verify installation**:
```bash
docling --version
```

**Without Docling**: You can still use all other features (LLM invocation, guardrails, JSON schema validation, etc.). PDF extraction will fail with an error if Docling is not installed.

## LLM Provider Access

You need access to at least one LLM provider:

### Option 1: OpenAI API

- Sign up at [platform.openai.com](https://platform.openai.com/)
- Create an API key
- Set environment variable: `export OPENAI_API_KEY=sk-...`

### Option 2: Ollama (Local Models)

- Install Ollama from [ollama.com](https://ollama.com/)
- Pull a model: `ollama pull llama3`
- Start Ollama server: `ollama serve` (or run as background service)
- Default API URL: `http://localhost:11434/v1/chat/completions`

### Option 3: Other OpenAI-Compatible APIs

Any API compatible with OpenAI's `/v1/chat/completions` endpoint works:
- Azure OpenAI
- Together AI
- Anyscale
- Custom hosted models

## System Requirements

- **OS**: Linux, macOS, or Windows (WSL recommended for Windows)
- **Memory**: 512MB minimum (depends on LLM provider)
- **Disk**: ~200MB for compiled binary
- **Network**: Internet access for API-based providers (not needed for local Ollama)

## Next Steps

Once your system meets these requirements, proceed to [Installation]({{ site.baseurl }}{% link getting-started/installation.md %}).
