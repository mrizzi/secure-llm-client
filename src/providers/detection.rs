use crate::provider::{LlmProvider, ProviderType};

use super::{ollama::OllamaProvider, openai::OpenAIProvider};

/// Detect API format from URL
///
/// # Detection Strategy
///
/// 1. **Path-based detection** (highest priority):
///    - `/api/generate` → Ollama
///    - `/v1/chat/completions` → OpenAI
///
/// 2. **Port-based detection** (fallback):
///    - Port 11434 → Ollama (common local server port)
///
/// 3. **Default**: OpenAI (industry standard for cloud APIs)
///
/// # Examples
///
/// ```
/// use secure_llm_client::{detect_provider_type, ProviderType};
///
/// // Path-based detection
/// assert!(matches!(
///     detect_provider_type("http://localhost:11434/api/generate"),
///     ProviderType::Ollama
/// ));
///
/// assert!(matches!(
///     detect_provider_type("https://api.openai.com/v1/chat/completions"),
///     ProviderType::OpenAI
/// ));
///
/// // Port-based fallback
/// assert!(matches!(
///     detect_provider_type("http://localhost:11434"),
///     ProviderType::Ollama
/// ));
///
/// // Default to OpenAI
/// assert!(matches!(
///     detect_provider_type("https://api.example.com"),
///     ProviderType::OpenAI
/// ));
/// ```
pub fn detect_provider_type(url: &str) -> ProviderType {
    // Path-based detection (most explicit, highest priority)
    // Respect the user's explicit endpoint path choice
    if url.contains("/api/generate") {
        return ProviderType::Ollama;
    }
    if url.contains("/v1/chat/completions") {
        return ProviderType::OpenAI;
    }

    // Port-based detection (fallback for ambiguous URLs)
    // Port 11434: Common for local servers, typically Ollama format
    if url.contains("localhost:11434") || url.contains("127.0.0.1:11434") {
        return ProviderType::Ollama;
    }

    // Default to OpenAI format (industry standard for cloud APIs)
    ProviderType::OpenAI
}

/// Create provider instance based on URL and optional explicit type
///
/// If `provider_type` is `Some(type)`, uses that type explicitly.
/// Otherwise, auto-detects from the URL using `detect_provider_type()`.
///
/// # Examples
///
/// ```ignore
/// use secure_llm_client::providers::detection::create_provider;
/// use secure_llm_client::provider::ProviderType;
///
/// // Auto-detection
/// let provider = create_provider("http://localhost:11434/api/generate".to_string(), None);
/// assert_eq!(provider.name(), "Ollama");
///
/// // Explicit type
/// let provider = create_provider(
///     "http://localhost:8080/custom".to_string(),
///     Some(ProviderType::Ollama)
/// );
/// assert_eq!(provider.name(), "Ollama");
/// ```
pub fn create_provider(
    api_url: String,
    provider_type: Option<ProviderType>,
) -> Box<dyn LlmProvider> {
    let provider = provider_type.unwrap_or_else(|| detect_provider_type(&api_url));

    match provider {
        ProviderType::Ollama => Box::new(OllamaProvider::new(api_url)),
        ProviderType::OpenAI => Box::new(OpenAIProvider::new(api_url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ollama_by_path() {
        let url = "http://localhost:11434/api/generate";
        assert!(matches!(detect_provider_type(url), ProviderType::Ollama));
    }

    #[test]
    fn test_detect_openai_by_path() {
        let url = "https://api.openai.com/v1/chat/completions";
        assert!(matches!(detect_provider_type(url), ProviderType::OpenAI));
    }

    #[test]
    fn test_detect_ollama_by_port() {
        let url = "http://localhost:11434";
        assert!(matches!(detect_provider_type(url), ProviderType::Ollama));

        let url = "http://127.0.0.1:11434";
        assert!(matches!(detect_provider_type(url), ProviderType::Ollama));
    }

    #[test]
    fn test_detect_default_openai() {
        let url = "https://api.example.com";
        assert!(matches!(detect_provider_type(url), ProviderType::OpenAI));
    }

    #[test]
    fn test_path_overrides_port() {
        // Port says Ollama, but path says OpenAI → path wins
        let url = "http://localhost:11434/v1/chat/completions";
        assert!(matches!(detect_provider_type(url), ProviderType::OpenAI));
    }

    #[test]
    fn test_create_provider_auto_detect_ollama() {
        let provider = create_provider("http://localhost:11434/api/generate".to_string(), None);
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn test_create_provider_auto_detect_openai() {
        let provider = create_provider(
            "https://api.openai.com/v1/chat/completions".to_string(),
            None,
        );
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn test_create_provider_explicit_type() {
        // Explicit type overrides auto-detection
        let provider = create_provider(
            "http://localhost:8080/custom".to_string(),
            Some(ProviderType::Ollama),
        );
        assert_eq!(provider.name(), "Ollama");
    }
}
