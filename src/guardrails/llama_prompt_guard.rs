use crate::{
    client::LlmClient,
    error::CliError,
    guardrails::provider::{GuardrailProvider, GuardrailResult, Severity, Violation},
    provider::InvokeParams,
};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Configuration for Llama Prompt Guard 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaPromptGuardConfig {
    /// API endpoint (Ollama or OpenAI-compatible)
    pub api_url: String,

    /// Model identifier
    /// - Ollama: "llama-prompt-guard-2-22m" or "llama-prompt-guard-2-86m"
    /// - Groq: "meta-llama/llama-prompt-guard-2-22m"
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Confidence threshold for MALICIOUS classification (0.0-1.0)
    /// Higher = fewer false positives, may miss subtle attacks
    /// Lower = catches more attacks, more false positives
    /// Default: 0.5 (balanced)
    #[serde(default = "default_threshold")]
    pub threshold: f32,

    /// Optional API key for authenticated endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Optional API key environment variable name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_name: Option<String>,
}

fn default_threshold() -> f32 {
    0.5
}

impl Default for LlamaPromptGuardConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:11434/api/generate".to_string(),
            model: "llama-prompt-guard-2-22m".to_string(),
            timeout_secs: 10,
            threshold: 0.5,
            api_key: None,
            api_key_name: None,
        }
    }
}

/// Llama Prompt Guard 2 specific result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaPromptGuardResult {
    /// True if classified as malicious
    pub malicious: bool,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// "BENIGN" or "MALICIOUS"
    pub label: String,
    /// Raw response from model
    pub raw_response: String,
}

/// Llama Prompt Guard 2 provider for prompt injection detection
pub struct LlamaPromptGuardProvider {
    client: LlmClient,
    config: LlamaPromptGuardConfig,
}

impl LlamaPromptGuardProvider {
    pub fn new(config: LlamaPromptGuardConfig) -> Self {
        let client = LlmClient::new(config.api_url.clone(), None);
        Self { client, config }
    }

    /// Parse Prompt Guard response (handles multiple formats)
    fn parse_response(&self, response: &str) -> Result<GuardrailResult, CliError> {
        let normalized = response.trim().to_uppercase();

        // Parse label (handle variations: "BENIGN", "MALICIOUS", "LABEL_0", "LABEL_1")
        let (is_malicious, confidence) =
            if normalized.contains("MALICIOUS") || normalized.contains("LABEL_1") {
                (true, self.extract_confidence(&normalized).unwrap_or(1.0))
            } else if normalized.contains("BENIGN") || normalized.contains("LABEL_0") {
                (false, self.extract_confidence(&normalized).unwrap_or(0.0))
            } else {
                return Err(CliError::InvalidResponse(format!(
                    "Unexpected Prompt Guard response: {response}"
                )));
            };

        // Apply threshold
        let passed = if is_malicious {
            confidence < self.config.threshold
        } else {
            true
        };

        // Create violations
        let violations = if !passed {
            vec![Violation {
                rule: "PROMPT_INJECTION".to_string(),
                severity: Severity::Critical,
                message: format!(
                    "Prompt injection or jailbreak attempt detected (confidence: {:.2}, threshold: {:.2})",
                    confidence,
                    self.config.threshold
                ),
                location: None,
            }]
        } else {
            vec![]
        };

        // Provider-specific result
        let prompt_guard_result = LlamaPromptGuardResult {
            malicious: is_malicious,
            confidence,
            label: if is_malicious { "MALICIOUS" } else { "BENIGN" }.to_string(),
            raw_response: response.to_string(),
        };

        Ok(GuardrailResult {
            passed,
            violations,
            warnings: vec![],
            quality_score: None,
            provider_specific: Some(
                crate::guardrails::provider::ProviderSpecificResult::LlamaPromptGuard(
                    prompt_guard_result,
                ),
            ),
        })
    }

    /// Extract confidence score from response text (if present)
    fn extract_confidence(&self, response: &str) -> Option<f32> {
        // Try patterns like:
        // "MALICIOUS (confidence: 0.95)"
        // "score: 0.87"
        // "(0.95)"
        // (?i) makes patterns case-insensitive
        let patterns = [
            r"(?i)confidence[:\s]+([0-9.]+)",
            r"(?i)score[:\s]+([0-9.]+)",
            r"\(([0-9.]+)\)",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(response) {
                    if let Some(score_str) = caps.get(1) {
                        if let Ok(score) = score_str.as_str().parse::<f32>() {
                            return Some(score);
                        }
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl GuardrailProvider for LlamaPromptGuardProvider {
    async fn validate_input(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Truncate if exceeds 512 tokens (~2048 chars)
        let truncated = if content.len() > 2048 {
            log::warn!("Input exceeds 512 tokens, truncating for Prompt Guard validation");
            &content[..2048]
        } else {
            content
        };

        let response = self
            .client
            .invoke(InvokeParams {
                model: &self.config.model,
                system_prompt: "", // Empty - classifier doesn't need system prompt
                user_prompt: truncated,
                temperature: 0.0,     // Deterministic classification
                max_tokens: Some(50), // Short response
                seed: None,
                api_key: self.config.api_key.as_deref(),
                timeout_secs: self.config.timeout_secs,
                response_format: None,
            })
            .await?;

        self.parse_response(&response)
    }

    async fn validate_output(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Same logic as validate_input (can detect injection in outputs too)
        self.validate_input(content).await
    }

    fn name(&self) -> &str {
        "LlamaPromptGuard2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_benign() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("BENIGN").unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_parse_response_malicious() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("MALICIOUS").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "PROMPT_INJECTION");
    }

    #[test]
    fn test_parse_response_label_0() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("LABEL_0").unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_parse_response_label_1() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("LABEL_1").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "PROMPT_INJECTION");
    }

    #[test]
    fn test_parse_response_case_insensitive() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("benign").unwrap();
        assert!(result.passed);

        let result = provider.parse_response("malicious").unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_threshold_filtering() {
        let config = LlamaPromptGuardConfig {
            threshold: 0.8, // High threshold
            ..Default::default()
        };
        let provider = LlamaPromptGuardProvider::new(config);

        // Low confidence malicious should pass (below threshold)
        let result = provider
            .parse_response("MALICIOUS (confidence: 0.6)")
            .unwrap();
        assert!(
            result.passed,
            "Should pass with confidence 0.6 < threshold 0.8"
        );
    }

    #[test]
    fn test_threshold_filtering_high_confidence() {
        let config = LlamaPromptGuardConfig {
            threshold: 0.5,
            ..Default::default()
        };
        let provider = LlamaPromptGuardProvider::new(config);

        // High confidence malicious should fail (above threshold)
        let result = provider
            .parse_response("MALICIOUS (confidence: 0.95)")
            .unwrap();
        assert!(
            !result.passed,
            "Should fail with confidence 0.95 >= threshold 0.5"
        );
    }

    #[test]
    fn test_extract_confidence_patterns() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        // Pattern: "confidence: 0.95"
        assert_eq!(
            provider.extract_confidence("MALICIOUS confidence: 0.95"),
            Some(0.95)
        );

        // Pattern: "score: 0.87"
        assert_eq!(
            provider.extract_confidence("MALICIOUS score: 0.87"),
            Some(0.87)
        );

        // Pattern: "(0.93)"
        assert_eq!(provider.extract_confidence("MALICIOUS (0.93)"), Some(0.93));

        // No confidence found
        assert_eq!(provider.extract_confidence("MALICIOUS"), None);
    }

    #[test]
    fn test_invalid_response() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider.parse_response("INVALID_RESPONSE");
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_specific_result() {
        let config = LlamaPromptGuardConfig::default();
        let provider = LlamaPromptGuardProvider::new(config);

        let result = provider
            .parse_response("MALICIOUS (confidence: 0.95)")
            .unwrap();

        // Check provider-specific result is populated
        assert!(result.provider_specific.is_some());

        if let Some(crate::guardrails::provider::ProviderSpecificResult::LlamaPromptGuard(
            pg_result,
        )) = result.provider_specific
        {
            assert!(pg_result.malicious);
            assert_eq!(pg_result.confidence, 0.95);
            assert_eq!(pg_result.label, "MALICIOUS");
            assert_eq!(pg_result.raw_response, "MALICIOUS (confidence: 0.95)");
        } else {
            panic!("Expected LlamaPromptGuard provider-specific result");
        }
    }
}
