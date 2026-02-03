use fortified_llm_client::{
    detect_provider_type, CliError, InvokeParams, LlmProvider, ProviderType,
};

// Mock provider for testing
struct MockProvider {
    response: String,
}

impl MockProvider {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockProvider {
    async fn invoke(&self, _params: InvokeParams<'_>) -> Result<String, CliError> {
        Ok(self.response.clone())
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

#[tokio::test]
async fn test_mock_provider() {
    let provider = MockProvider::new("Test response from mock provider");

    let result = provider
        .invoke(InvokeParams {
            model: "test-model",
            system_prompt: "system",
            user_prompt: "user",
            temperature: 0.1,
            max_tokens: Some(1000),
            seed: None,
            api_key: None,
            timeout_secs: 300,
            response_format: None,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Test response from mock provider");
}

#[tokio::test]
async fn test_mock_provider_name() {
    let provider = MockProvider::new("test");
    assert_eq!(provider.name(), "Mock");
}

#[test]
fn test_provider_detection_ollama_localhost() {
    let url = "http://localhost:11434/api/generate";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::Ollama));
}

#[test]
fn test_provider_detection_ollama_ip() {
    let url = "http://127.0.0.1:11434/api/generate";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::Ollama));
}

#[test]
fn test_provider_detection_openai() {
    let url = "https://api.openai.com/v1/chat/completions";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::OpenAI));
}

#[test]
fn test_provider_detection_groq() {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::OpenAI));
}

#[test]
fn test_provider_detection_azure() {
    let url = "https://myresource.openai.azure.com/openai/deployments/gpt-4/chat/completions";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::OpenAI));
}

#[test]
fn test_provider_detection_ollama_openai_compat_endpoint() {
    // Path-based detection should respect explicit /v1/chat/completions endpoint
    // even on localhost:11434
    let url = "http://localhost:11434/v1/chat/completions";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::OpenAI));
}

#[test]
fn test_provider_detection_ollama_native_endpoint() {
    // Explicit /api/generate path should always use Ollama format
    let url = "http://localhost:11434/api/generate";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::Ollama));
}

#[test]
fn test_provider_detection_ollama_port_fallback() {
    // Without explicit path, localhost:11434 falls back to Ollama
    let url = "http://localhost:11434";
    let provider_type = detect_provider_type(url);
    assert!(matches!(provider_type, ProviderType::Ollama));
}

#[test]
fn test_provider_detection_path_takes_precedence() {
    // Path-based detection should override port-based detection
    let url = "http://localhost:11434/v1/chat/completions";
    let provider_type = detect_provider_type(url);
    assert!(
        matches!(provider_type, ProviderType::OpenAI),
        "Path /v1/chat/completions should override port 11434 detection"
    );
}
