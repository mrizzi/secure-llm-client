---
layout: default
title: Providers
parent: Architecture
nav_order: 3
---

# Provider System

Multi-provider support with automatic detection and unified interface.

## Supported Providers

1. **OpenAI** - Official OpenAI API
2. **Ollama** - Local models with OpenAI-compatible API
3. **OpenAI-compatible** - Any service using `/v1/chat/completions` endpoint

## Provider Detection

**Location**: `src/providers/detection.rs`

### Auto-Detection Logic

Analyzes API URL to infer provider:

```rust
pub fn detect_provider(api_url: &str) -> Provider {
    if api_url.contains("openai.com") || api_url.contains("azure.com") {
        Provider::OpenAI
    } else if api_url.contains("ollama") || api_url.contains("localhost:11434") {
        Provider::Ollama
    } else {
        Provider::OpenAI  // Default fallback (most compatible)
    }
}
```

**Patterns matched**:
- `openai.com` → OpenAI
- `azure.com` → OpenAI (Azure uses same format)
- `ollama` or `localhost:11434` → Ollama
- Everything else → OpenAI (fallback)

### Explicit Override

Force provider via CLI or config:

**CLI**:
```bash
--provider openai
--provider ollama
```

**Config**:
```toml
provider = "openai"
```

**Library**:
```rust
use fortified_llm_client::Provider;

let config = EvaluationConfig {
    provider: Some(Provider::OpenAI),
    // ...
};
```

## LlmProvider Trait

**Location**: `src/providers/provider.rs`

Unified interface for all providers:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError>;
}
```

### LlmRequest

```rust
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u64>,
    pub response_format: Option<ResponseFormat>,
    // ...
}
```

### LlmResponse

```rust
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: Option<String>,
}
```

## OpenAI Provider

**Location**: `src/providers/openai.rs`

### Implementation

```rust
pub struct OpenAIProvider {
    api_url: String,
    api_key: Option<String>,
    timeout: Duration,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Build request body
        // Make HTTP POST to api_url
        // Parse JSON response
        // Extract content from choices[0].message.content
    }
}
```

### Request Format

```json
{
  "model": "gpt-4",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Explain Rust ownership"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "seed": 42
}
```

### Response Format

```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "Rust ownership ensures..."
      },
      "finish_reason": "stop"
    }
  ],
  "model": "gpt-4",
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 100,
    "total_tokens": 120
  }
}
```

## Ollama Provider

**Location**: `src/providers/ollama.rs`

### Implementation

```rust
pub struct OllamaProvider {
    api_url: String,
    timeout: Duration,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Same format as OpenAI (Ollama is compatible)
        // No API key required
    }
}
```

### Differences from OpenAI

1. **No API key required** - Ollama runs locally
2. **Same request/response format** - OpenAI-compatible
3. **Local models** - Models must be pulled first (`ollama pull llama3`)

## Error Handling

### Common Errors

| Error | Provider | Cause |
|-------|----------|-------|
| 401 Unauthorized | OpenAI | Invalid/missing API key |
| 404 Not Found | Ollama | Model not pulled |
| 429 Rate Limit | OpenAI | Too many requests |
| Connection Refused | Ollama | Ollama not running |
| Timeout | Both | Request took too long |

### Error Mapping

```rust
match status {
    401 => FortifiedError::ApiError {
        message: "Authentication failed".to_string(),
        status_code: Some(401),
    },
    404 => FortifiedError::ApiError {
        message: "Model not found".to_string(),
        status_code: Some(404),
    },
    // ...
}
```

## Adding New Providers

### Step 1: Implement LlmProvider Trait

Create `src/providers/my_provider.rs`:

```rust
use async_trait::async_trait;
use crate::providers::{LlmProvider, LlmRequest, LlmResponse};
use crate::FortifiedError;

pub struct MyProvider {
    api_url: String,
    api_key: Option<String>,
}

#[async_trait]
impl LlmProvider for MyProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Your implementation
    }
}
```

### Step 2: Update Provider Enum

In `src/lib.rs`:

```rust
pub enum Provider {
    OpenAI,
    Ollama,
    MyProvider,  // Add new variant
}
```

### Step 3: Update Detection Logic

In `src/providers/detection.rs`:

```rust
pub fn detect_provider(api_url: &str) -> Provider {
    if api_url.contains("myprovider.com") {
        Provider::MyProvider
    } else if api_url.contains("openai.com") {
        Provider::OpenAI
    }
    // ...
}
```

### Step 4: Update Client Factory

In `src/client.rs`:

```rust
match provider {
    Provider::OpenAI => Box::new(OpenAIProvider::new(/* ... */)),
    Provider::Ollama => Box::new(OllamaProvider::new(/* ... */)),
    Provider::MyProvider => Box::new(MyProvider::new(/* ... */)),
}
```

## Testing

### Mock Providers for Testing

Use `mockito` for HTTP mocking:

```rust
use mockito::{mock, server_url};

#[tokio::test]
async fn test_openai_provider() {
    let _m = mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_body(r#"{"choices":[{"message":{"content":"Test"}}]}"#)
        .create();

    let provider = OpenAIProvider::new(server_url(), None, Duration::from_secs(30));
    let request = LlmRequest { /* ... */ };
    let response = provider.invoke(request).await.unwrap();

    assert_eq!(response.content, "Test");
}
```

## See Also

- [Layers]({{ site.baseurl }}{% link architecture/layers.md %}) - Architecture overview
- [Evaluation Pipeline]({{ site.baseurl }}{% link architecture/evaluation-pipeline.md %}) - Where providers fit
- [User Guide]({{ site.baseurl }}{% link user-guide/cli-usage.md %}) - Using providers in practice
