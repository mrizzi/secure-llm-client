use crate::{
    error::CliError,
    models::{OllamaOptions, OllamaRequest, OllamaResponse},
    provider::{InvokeParams, LlmProvider},
};
use async_trait::async_trait;
use reqwest::Client;

use super::logging::{log_request, log_response};

/// Provider for Ollama /api/generate format (local servers)
pub struct OllamaProvider {
    client: Client,
    api_url: String,
}

impl OllamaProvider {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        // Note: Ollama's /api/generate format doesn't use max_tokens, api_key, or response_format
        let request = OllamaRequest {
            model: params.model.to_string(),
            system: params.system_prompt.to_string(),
            prompt: params.user_prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: params.temperature,
                seed: params.seed,
            },
        };

        log_request(&request);

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(params.timeout_secs))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CliError::InvalidResponse(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Get response body as text for logging and parsing
        let response_text = response.text().await?;
        log_response(&response_text);

        // Parse the response
        let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
            .map_err(|e| CliError::InvalidResponse(format!("Failed to parse response: {e}")))?;
        Ok(ollama_response.response)
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_new() {
        let provider = OllamaProvider::new("http://localhost:11434/api/generate".to_string());
        assert_eq!(provider.name(), "Ollama");
        assert_eq!(provider.api_url, "http://localhost:11434/api/generate");
    }

    #[test]
    fn test_ollama_provider_name() {
        let provider = OllamaProvider::new("http://localhost:11434/api/generate".to_string());
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn test_ollama_provider_supports_streaming() {
        let provider = OllamaProvider::new("http://localhost:11434/api/generate".to_string());
        assert!(!provider.supports_streaming()); // Default implementation
    }
}
