use crate::{
    error::CliError,
    models::{Message, OpenAIRequest, OpenAIResponse},
    provider::{InvokeParams, LlmProvider},
};
use async_trait::async_trait;
use reqwest::Client;

use super::logging::{log_request, log_response};

/// OpenAI-compatible provider implementation
pub struct OpenAIProvider {
    client: Client,
    api_url: String,
}

impl OpenAIProvider {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        let request = OpenAIRequest {
            model: params.model.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: params.system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: params.user_prompt.to_string(),
                },
            ],
            temperature: params.temperature,
            max_tokens: params.max_tokens,
            seed: params.seed,
            response_format: params.response_format.cloned(),
        };

        log_request(&request);

        let mut req = self
            .client
            .post(&self.api_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(params.timeout_secs));

        if let Some(key) = params.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
            log::debug!("Authorization header: Bearer [REDACTED]");
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();

            // Special case: 401 authentication error
            if status == 401 {
                return Err(CliError::AuthenticationFailed(
                    "Invalid or missing API key".to_string(),
                ));
            }

            let error_body = response.text().await.unwrap_or_default();

            // Let the API's error message speak for itself
            let error_msg = format!(
                "HTTP {} error: {}\nResponse from API: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown error"),
                if error_body.is_empty() {
                    "No details provided"
                } else {
                    &error_body
                }
            );

            return Err(CliError::InvalidResponse(error_msg));
        }

        // Get response body as text for logging and parsing
        let response_text = response.text().await?;
        log_response(&response_text);

        // Parse the response
        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)
            .map_err(|e| CliError::InvalidResponse(format!("Failed to parse response: {e}")))?;

        openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| CliError::InvalidResponse("No choices in response".to_string()))
    }

    fn name(&self) -> &str {
        "OpenAI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_new() {
        let provider =
            OpenAIProvider::new("https://api.openai.com/v1/chat/completions".to_string());
        assert_eq!(provider.name(), "OpenAI");
        assert_eq!(
            provider.api_url,
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_openai_provider_name() {
        let provider =
            OpenAIProvider::new("https://api.openai.com/v1/chat/completions".to_string());
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn test_openai_provider_supports_streaming() {
        let provider =
            OpenAIProvider::new("https://api.openai.com/v1/chat/completions".to_string());
        assert!(!provider.supports_streaming()); // Default implementation
    }
}
