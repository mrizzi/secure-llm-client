---
layout: default
title: Home
nav_order: 1
description: "Rust library and CLI tool for LLM interactions fortified by multi-layered security guardrails"
permalink: /
---

# Fortified LLM Client

A Rust library and CLI tool for interacting with Large Language Model (LLM) providers, fortified by multi-layered security guardrails, PDF extraction, and multi-provider support.

{: .warning }
> **Active Development**: This project is currently under active development.
> The library API may change between versions. Not recommended for production use without thorough testing.

## Quick Links

- [Getting Started]({{ site.baseurl }}{% link getting-started/index.md %})
- [User Guide]({{ site.baseurl }}{% link user-guide/index.md %})
- [Architecture]({{ site.baseurl }}{% link architecture/index.md %})
- [Guardrails]({{ site.baseurl }}{% link guardrails/index.md %})

## Key Features

### Multi-Provider Support
- OpenAI-compatible APIs
- Ollama local models
- Automatic provider detection from API URL
- Unified interface via `LlmProvider` trait

### Security Guardrails
- Input validation (pattern matching, PII detection, prompt injection)
- Output validation with quality scoring
- LLM-based guardrails (Llama Guard, Llama Prompt Guard, GPT OSS Safeguard)
- Hybrid guardrails with configurable execution modes

### PDF Processing
- Extract text from PDFs using Docling CLI
- File size validation for resource protection
- Direct integration into LLM prompts

### Token Management
- Model-specific token estimation
- Context limit validation
- Per-request token budget control

[Get Started]({{ site.baseurl }}{% link getting-started/installation.md %}){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/mrizzi/fortified-llm-client){: .btn .fs-5 .mb-4 .mb-md-0 }
