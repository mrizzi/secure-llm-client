---
layout: default
title: Response Formats
parent: User Guide
nav_order: 5
---

# Response Formats

Control how LLMs structure their output.

## Table of Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

## Overview

Fortified LLM Client supports three response formats for OpenAI-compatible models:

1. **Text** (default) - Plain text responses
2. **JSON Object** - Unstructured JSON output
3. **JSON Schema** - Structured JSON validated against a schema

{: .note }
> Response formatting requires OpenAI-compatible models. Not all providers support these features.

## Text Format (Default)

Plain text responses without structure enforcement.

**CLI**:
```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Explain Rust ownership"
# No --response-format needed (text is default)
```

**Library**:
```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    user_prompt: "Explain Rust ownership".to_string(),
    ..Default::default()
};

let result = evaluate(config).await?;
```

**Output**:
```json
{
  "status": "success",
  "content": "Rust ownership ensures memory safety...",
  "metadata": { ... }
}
```

## JSON Object Format

Instructs the LLM to output valid JSON (unstructured).

**CLI**:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Generate a product: name, price, category" \
  --response-format json-object
```

**Library**:
```rust
use fortified_llm_client::{evaluate, EvaluationConfig, ResponseFormat};

let config = EvaluationConfig {
    api_url: "https://api.openai.com/v1/chat/completions".to_string(),
    model: "gpt-4".to_string(),
    user_prompt: "Generate a product: name, price, category".to_string(),
    response_format: Some(ResponseFormat::JsonObject),
    api_key_name: Some("OPENAI_API_KEY".to_string()),
    ..Default::default()
};

let result = evaluate(config).await?;
```

**Output** (example):
```json
{
  "status": "success",
  "content": "{\"name\": \"Laptop\", \"price\": 999.99, \"category\": \"Electronics\"}",
  "metadata": { ... }
}
```

{: .warning }
> The LLM generates JSON, but structure is not enforced. Use JSON Schema for strict validation.

## JSON Schema Format

Validates LLM output against a JSON Schema, ensuring strict structure.

### Basic Usage

**CLI**:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Generate a product catalog with 2 items" \
  --response-format json-schema \
  --response-format-schema schemas/product.json
```

**Library**:
```rust
use fortified_llm_client::{evaluate, EvaluationConfig, ResponseFormat};

let config = EvaluationConfig {
    api_url: "https://api.openai.com/v1/chat/completions".to_string(),
    model: "gpt-4".to_string(),
    user_prompt: "Generate a product catalog with 2 items".to_string(),
    response_format: Some(ResponseFormat::JsonSchema),
    response_format_schema: Some("schemas/product.json".to_string()),
    response_format_schema_strict: true,
    api_key_name: Some("OPENAI_API_KEY".to_string()),
    ..Default::default()
};

let result = evaluate(config).await?;
```

### Schema Example

`schemas/product.json`:
```json
{
  "type": "object",
  "properties": {
    "products": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "price": {"type": "number"},
          "category": {"type": "string"}
        },
        "required": ["name", "price", "category"]
      }
    }
  },
  "required": ["products"]
}
```

**Output**:
```json
{
  "status": "success",
  "content": "{\"products\": [{\"name\": \"Laptop\", \"price\": 999.99, \"category\": \"Electronics\"}, {\"name\": \"Mouse\", \"price\": 29.99, \"category\": \"Accessories\"}]}",
  "metadata": { ... }
}
```

### Strict Mode

**Default**: `true` (strict validation enabled)

**Strict mode** enforces:
- All fields specified in schema
- No additional properties
- Exact type matching

**Disable strict mode**:
```bash
--response-format-schema-strict false
```

```rust
response_format_schema_strict: false,
```

## Use Cases

### Use Case 1: Structured Data Extraction

Extract invoice details with guaranteed structure:

**Schema** (`schemas/invoice.json`):
```json
{
  "type": "object",
  "properties": {
    "invoice_number": {"type": "string"},
    "date": {"type": "string", "format": "date"},
    "total": {"type": "number"},
    "vendor": {"type": "string"},
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "description": {"type": "string"},
          "quantity": {"type": "integer"},
          "unit_price": {"type": "number"}
        },
        "required": ["description", "quantity", "unit_price"]
      }
    }
  },
  "required": ["invoice_number", "date", "total", "vendor", "items"]
}
```

**CLI**:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --pdf-file invoice.pdf \
  --response-format json-schema \
  --response-format-schema schemas/invoice.json
```

### Use Case 2: API Response Generation

Generate API-compatible responses:

**Schema** (`schemas/api-response.json`):
```json
{
  "type": "object",
  "properties": {
    "success": {"type": "boolean"},
    "data": {"type": "object"},
    "error": {
      "type": "object",
      "properties": {
        "code": {"type": "string"},
        "message": {"type": "string"}
      }
    }
  },
  "required": ["success"]
}
```

### Use Case 3: Configuration Generation

Generate config files with specific structure:

**Schema** (`schemas/config.json`):
```json
{
  "type": "object",
  "properties": {
    "database": {
      "type": "object",
      "properties": {
        "host": {"type": "string"},
        "port": {"type": "integer"},
        "username": {"type": "string"}
      },
      "required": ["host", "port"]
    },
    "logging": {
      "type": "object",
      "properties": {
        "level": {"type": "string", "enum": ["debug", "info", "warn", "error"]},
        "format": {"type": "string"}
      }
    }
  },
  "required": ["database"]
}
```

**CLI**:
```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Generate a database config for PostgreSQL on localhost:5432 with debug logging" \
  --response-format json-schema \
  --response-format-schema schemas/config.json \
  --output generated-config.json
```

## Schema Design Tips

### 1. Be Specific with Types

```json
{
  "properties": {
    "age": {"type": "integer", "minimum": 0, "maximum": 150},
    "email": {"type": "string", "format": "email"},
    "website": {"type": "string", "format": "uri"}
  }
}
```

### 2. Use Enums for Fixed Values

```json
{
  "properties": {
    "status": {"type": "string", "enum": ["pending", "approved", "rejected"]},
    "priority": {"type": "string", "enum": ["low", "medium", "high", "critical"]}
  }
}
```

### 3. Provide Descriptions

Helps the LLM understand intent:

```json
{
  "properties": {
    "summary": {
      "type": "string",
      "description": "A concise summary in 1-2 sentences"
    },
    "keywords": {
      "type": "array",
      "description": "3-5 most relevant keywords",
      "items": {"type": "string"},
      "minItems": 3,
      "maxItems": 5
    }
  }
}
```

### 4. Require Critical Fields

```json
{
  "required": ["id", "timestamp", "data"],
  "properties": {
    "id": {"type": "string"},
    "timestamp": {"type": "string", "format": "date-time"},
    "data": {"type": "object"},
    "metadata": {"type": "object"}  // Optional
  }
}
```

## Validation Errors

### Schema Not Found

**Error**: `Failed to read schema file: No such file or directory`

**Fix**: Ensure schema file path is correct.

### Invalid JSON Schema

**Error**: `Invalid JSON schema: ...`

**Fix**: Validate schema at [jsonschema.net](https://www.jsonschema.net/).

### LLM Output Doesn't Match Schema

**Error**: `Validation error: ... does not match schema`

**Possible causes**:
- Prompt not clear enough
- Schema too strict/complex
- Model doesn't support schema validation well

**Fix**:
1. Simplify schema
2. Make prompt more explicit about structure
3. Disable strict mode temporarily to debug

## Best Practices

### 1. Start Simple

Test with basic schema, then add complexity:

```json
// Start here
{
  "type": "object",
  "properties": {
    "result": {"type": "string"}
  }
}

// Then expand
{
  "type": "object",
  "properties": {
    "result": {"type": "string"},
    "confidence": {"type": "number"},
    "metadata": {"type": "object"}
  }
}
```

### 2. Include Schema Description in Prompt

```bash
--user-text "Generate a product in JSON format: {name: string, price: number, category: string}"
```

### 3. Test Schema Separately

Validate schema before using:
```bash
# Use a JSON schema validator
npx ajv validate -s schemas/product.json -d test-data.json
```

### 4. Use additionalProperties: false

Prevent unexpected fields in strict mode:

```json
{
  "type": "object",
  "properties": { ... },
  "additionalProperties": false
}
```

## Next Steps

- [Token Management]({{ site.baseurl }}{% link user-guide/token-management.md %}) - Estimate tokens
- [PDF Extraction]({{ site.baseurl }}{% link user-guide/pdf-extraction.md %}) - Extract data from PDFs
- [Examples]({{ site.baseurl }}{% link examples/basic-usage.md %}) - More examples
