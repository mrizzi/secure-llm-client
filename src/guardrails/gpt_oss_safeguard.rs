use crate::{
    client::LlmClient,
    error::CliError,
    guardrails::provider::{
        GuardrailProvider, GuardrailResult, ProviderSpecificResult, Severity, Violation,
    },
    provider::InvokeParams,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// GPT-OSS-Safeguard response format
#[derive(Debug, Clone, Deserialize, Serialize)]
struct GptOssSafeguardResponse {
    violation: u8, // 0 = safe, 1 = unsafe
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rationale: Option<String>,
}

/// Configuration for GPT-OSS-Safeguard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GptOssSafeguardConfig {
    pub api_url: String,
    pub model: String,  // e.g., "gpt-oss-safeguard-20b", "gpt-oss-safeguard-120b"
    pub policy: String, // Full policy text as system prompt
    pub timeout_secs: u64,
    pub api_key: Option<String>,
}

impl Default for GptOssSafeguardConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:1234/v1/chat/completions".to_string(),
            model: "gpt-oss-safeguard-20b".to_string(),
            policy: Self::mlcommons_taxonomy_policy(),
            timeout_secs: 30,
            api_key: None,
        }
    }
}

impl GptOssSafeguardConfig {
    /// MLCommons taxonomy policy (S1-S13) - mirrors Llama Guard 3 categories
    /// This makes GPT-OSS-Safeguard behave like Llama Guard 3
    ///
    /// Policy is loaded from external file at compile time for easier maintenance.
    pub fn mlcommons_taxonomy_policy() -> String {
        const POLICY: &str = include_str!("policies/mlcommons_taxonomy_policy.txt");
        POLICY.to_string()
    }
}

/// GPT-OSS-Safeguard provider
pub struct GptOssSafeguardProvider {
    client: LlmClient,
    config: GptOssSafeguardConfig,
}

impl GptOssSafeguardProvider {
    pub fn new(config: GptOssSafeguardConfig) -> Self {
        let client = LlmClient::new(config.api_url.clone(), None);
        Self { client, config }
    }

    /// Parse GPT-OSS-Safeguard JSON response
    /// Format: {"violation": 0 or 1, "category": "S1" or null, "rationale": "..."}
    fn parse_json_response(&self, response: &str) -> Result<GuardrailResult, CliError> {
        // Try to parse as JSON
        let json: GptOssSafeguardResponse = serde_json::from_str(response.trim()).map_err(|e| {
            CliError::InvalidResponse(format!(
                "Failed to parse GPT-OSS-Safeguard response as JSON: {e}. Response: {response}"
            ))
        })?;

        let passed = json.violation == 0;

        let violations = if !passed {
            vec![Violation {
                rule: json
                    .category
                    .clone()
                    .unwrap_or_else(|| "POLICY_VIOLATION".to_string()),
                severity: Severity::Critical,
                message: json
                    .rationale
                    .clone()
                    .unwrap_or_else(|| "Policy violation detected".to_string()),
                location: None,
            }]
        } else {
            vec![]
        };

        // Create provider-specific result
        let gpt_oss_result = crate::guardrails::provider::GptOssSafeguardResult {
            violation: json.violation == 1,
            category: json.category.clone(),
            rationale: json.rationale.clone(),
            raw_response: response.to_string(),
        };

        Ok(GuardrailResult {
            passed,
            violations,
            warnings: vec![],
            quality_score: None, // GPT-OSS-Safeguard is binary (no confidence scores)
            provider_specific: Some(ProviderSpecificResult::GptOssSafeguard(gpt_oss_result)),
        })
    }
}

#[async_trait]
impl GuardrailProvider for GptOssSafeguardProvider {
    async fn validate_input(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // GPT-OSS-Safeguard requires policy as system prompt and content as user prompt
        let response = self
            .client
            .invoke(InvokeParams {
                model: &self.config.model,
                system_prompt: &self.config.policy, // Policy in system prompt
                user_prompt: content,               // Content to evaluate in user prompt
                temperature: 0.0, // Temperature 0 for deterministic classification
                max_tokens: Some(300), // Longer than Llama Guard (JSON output needs more tokens)
                seed: None,       // No seed needed for guardrails
                api_key: self.config.api_key.as_deref(),
                timeout_secs: self.config.timeout_secs,
                response_format: None, // No response_format needed for guardrails
            })
            .await?;

        self.parse_json_response(&response)
    }

    async fn validate_output(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Same as validate_input - GPT-OSS-Safeguard doesn't distinguish between
        // input and output in its classification, just evaluates content against policy
        self.validate_input(content).await
    }

    fn name(&self) -> &str {
        "GptOssSafeguard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_safe() {
        let config = GptOssSafeguardConfig::default();
        let provider = GptOssSafeguardProvider::new(config);

        let response = r#"{"violation": 0, "category": null, "rationale": "Legitimate question"}"#;
        let result = provider.parse_json_response(response).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
        assert!(result.quality_score.is_none());
    }

    #[test]
    fn test_parse_response_unsafe_with_category() {
        let config = GptOssSafeguardConfig::default();
        let provider = GptOssSafeguardProvider::new(config);

        let response =
            r#"{"violation": 1, "category": "S1", "rationale": "Violent crime instruction"}"#;
        let result = provider.parse_json_response(response).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "S1");
        assert_eq!(result.violations[0].severity, Severity::Critical);
        assert!(result.violations[0].message.contains("Violent crime"));
    }

    #[test]
    fn test_parse_response_unsafe_no_category() {
        let config = GptOssSafeguardConfig::default();
        let provider = GptOssSafeguardProvider::new(config);

        let response = r#"{"violation": 1, "rationale": "Policy violation"}"#;
        let result = provider.parse_json_response(response).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "POLICY_VIOLATION");
    }

    #[test]
    fn test_parse_response_minimal_safe() {
        let config = GptOssSafeguardConfig::default();
        let provider = GptOssSafeguardProvider::new(config);

        let response = r#"{"violation": 0}"#;
        let result = provider.parse_json_response(response).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let config = GptOssSafeguardConfig::default();
        let provider = GptOssSafeguardProvider::new(config);

        let response = "not json";
        let result = provider.parse_json_response(response);

        assert!(result.is_err());
    }

    #[test]
    fn test_mlcommons_policy_contains_all_categories() {
        let policy = GptOssSafeguardConfig::mlcommons_taxonomy_policy();

        // Verify all 13 categories are present
        assert!(policy.contains("S1"));
        assert!(policy.contains("S2"));
        assert!(policy.contains("S3"));
        assert!(policy.contains("S4"));
        assert!(policy.contains("S5"));
        assert!(policy.contains("S6"));
        assert!(policy.contains("S7"));
        assert!(policy.contains("S8"));
        assert!(policy.contains("S9"));
        assert!(policy.contains("S10"));
        assert!(policy.contains("S11"));
        assert!(policy.contains("S12"));
        assert!(policy.contains("S13"));

        // Verify category descriptions
        assert!(policy.contains("Violent Crimes"));
        assert!(policy.contains("Non-Violent Crimes"));
        assert!(policy.contains("Child Sexual Exploitation"));
        assert!(policy.contains("Hate Speech"));
        assert!(policy.contains("Elections"));
    }
}
