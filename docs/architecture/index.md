---
layout: default
title: Architecture
nav_order: 4
has_children: true
permalink: /architecture/
---

# Architecture

Understanding the design and structure of Fortified LLM Client.

## Overview

Fortified LLM Client follows a **layered architecture** with clear separation of concerns:

1. **CLI Layer** - Argument parsing, config merging, output formatting
2. **Library Layer** - Public API (`evaluate()`, `evaluate_with_guardrails()`)
3. **Client Layer** - Provider-agnostic LLM client abstraction
4. **Provider Layer** - Provider-specific implementations (OpenAI, Ollama)
5. **Guardrails Layer** - Security validation pipeline

## Core Concepts

### Evaluation Pipeline

The evaluation flow follows this sequence:

1. **PDF Extraction** (optional) - Extract text from PDF using Docling CLI
2. **Input Guardrails** - Validate user input (NOT system prompts - those are trusted)
3. **Token Validation** - Estimate tokens and check against context limits
4. **LLM Invocation** - Call provider API
5. **Output Guardrails** (optional) - Validate LLM response
6. **Metadata Generation** - Create structured output with execution metadata

### Configuration System

**Dual configuration approach**:
- **Figment Merging** - Handles scalar fields with priority: CLI args > Config file
- **ConfigFileRequest** - Parses complex nested structures (guardrails) from TOML/JSON

### Provider System

- **Auto-detection** - Analyzes API URL patterns to infer provider type
- **Explicit Override** - `--provider` flag forces specific provider format
- **Unified Interface** - `LlmProvider` trait with common `invoke()` method

## Key Design Principles

1. **Defense in Depth** - Multiple security layers (pattern-based + LLM-based guardrails)
2. **Provider Agnostic** - Unified interface for all LLM providers
3. **Fail Fast** - Validate early to save API costs
4. **Composable Guardrails** - Mix and match validation strategies
5. **System Prompts Are Trusted** - Only user inputs are validated by guardrails

## Section Contents

- **[Layers]({{ site.baseurl }}{% link architecture/layers.md %})** - 5-layer design details
- **[Evaluation Pipeline]({{ site.baseurl }}{% link architecture/evaluation-pipeline.md %})** - Step-by-step execution flow
- **[Providers]({{ site.baseurl }}{% link architecture/providers.md %})** - Provider detection and implementation
- **[Testing]({{ site.baseurl }}{% link architecture/testing.md %})** - Test organization and strategy
