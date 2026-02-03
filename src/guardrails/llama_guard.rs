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

/// MLCommons AI Risk and Reliability Benchmark v1.0 categories
/// Plus Meta additions (S13, S14)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlamaGuardCategory {
    S1,  // Violent Crimes
    S2,  // Non-Violent Crimes
    S3,  // Sex-Related Crimes
    S4,  // Child Sexual Exploitation
    S5,  // Defamation
    S6,  // Specialized Advice
    S7,  // Privacy
    S8,  // Intellectual Property
    S9,  // Indiscriminate Weapons
    S10, // Hate
    S11, // Suicide & Self-Harm
    S12, // Sexual Content
    S13, // Elections
    S14, // Code Interpreter Abuse
}

impl LlamaGuardCategory {
    /// Return all 14 safety categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::S1,
            Self::S2,
            Self::S3,
            Self::S4,
            Self::S5,
            Self::S6,
            Self::S7,
            Self::S8,
            Self::S9,
            Self::S10,
            Self::S11,
            Self::S12,
            Self::S13,
            Self::S14,
        ]
    }

    /// Get human-readable description of the category
    pub fn description(&self) -> &'static str {
        match self {
            Self::S1 => "Violent Crimes",
            Self::S2 => "Non-Violent Crimes",
            Self::S3 => "Sex-Related Crimes",
            Self::S4 => "Child Sexual Exploitation",
            Self::S5 => "Defamation",
            Self::S6 => "Specialized Advice (financial, medical, legal)",
            Self::S7 => "Privacy Violations",
            Self::S8 => "Intellectual Property Violations",
            Self::S9 => "Indiscriminate Weapons (CBRNE)",
            Self::S10 => "Hate Speech",
            Self::S11 => "Suicide & Self-Harm",
            Self::S12 => "Sexual Content",
            Self::S13 => "Elections (misinformation)",
            Self::S14 => "Code Interpreter Abuse",
        }
    }

    /// Parse category from string (e.g., "S1", "S10")
    pub fn parse(s: &str) -> Result<Self, CliError> {
        match s.to_uppercase().as_str() {
            "S1" => Ok(Self::S1),
            "S2" => Ok(Self::S2),
            "S3" => Ok(Self::S3),
            "S4" => Ok(Self::S4),
            "S5" => Ok(Self::S5),
            "S6" => Ok(Self::S6),
            "S7" => Ok(Self::S7),
            "S8" => Ok(Self::S8),
            "S9" => Ok(Self::S9),
            "S10" => Ok(Self::S10),
            "S11" => Ok(Self::S11),
            "S12" => Ok(Self::S12),
            "S13" => Ok(Self::S13),
            "S14" => Ok(Self::S14),
            _ => Err(CliError::InvalidResponse(format!(
                "Unknown Llama Guard category: {s}"
            ))),
        }
    }

    /// Format category as string (e.g., "S1", "S10")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S1 => "S1",
            Self::S2 => "S2",
            Self::S3 => "S3",
            Self::S4 => "S4",
            Self::S5 => "S5",
            Self::S6 => "S6",
            Self::S7 => "S7",
            Self::S8 => "S8",
            Self::S9 => "S9",
            Self::S10 => "S10",
            Self::S11 => "S11",
            Self::S12 => "S12",
            Self::S13 => "S13",
            Self::S14 => "S14",
        }
    }
}

/// Configuration for Llama Guard 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaGuardConfig {
    pub api_url: String,
    pub model: String, // e.g., "llama-guard-3:8b", "llama-guard-3:1b"
    pub enabled_categories: Vec<LlamaGuardCategory>,
    pub timeout_secs: u64,
    pub api_key: Option<String>,
}

impl Default for LlamaGuardConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:11434/api/generate".to_string(),
            model: "llama-guard-3:8b".to_string(),
            enabled_categories: LlamaGuardCategory::all(),
            timeout_secs: 30,
            api_key: None,
        }
    }
}

/// Llama Guard 3 provider
pub struct LlamaGuardProvider {
    client: LlmClient,
    config: LlamaGuardConfig,
}

impl LlamaGuardProvider {
    pub fn new(config: LlamaGuardConfig) -> Self {
        let client = LlmClient::new(config.api_url.clone(), None);
        Self { client, config }
    }

    /// Parse Llama Guard 3 response
    /// Format: "safe" or "unsafe\nS1,S3,S7"
    fn parse_response(&self, response: &str) -> Result<GuardrailResult, CliError> {
        let response = response.trim();
        let lines: Vec<&str> = response.lines().collect();

        if lines.is_empty() {
            return Err(CliError::InvalidResponse(
                "Empty Llama Guard response".to_string(),
            ));
        }

        let safe = lines[0].trim().eq_ignore_ascii_case("safe");

        let violated_categories = if !safe && lines.len() > 1 {
            self.parse_categories(lines[1])?
        } else {
            vec![]
        };

        // Filter violations to only enabled categories
        let filtered_violations: Vec<LlamaGuardCategory> = violated_categories
            .into_iter()
            .filter(|cat| self.config.enabled_categories.contains(cat))
            .collect();

        // Convert to generic GuardrailResult
        let violations = filtered_violations
            .iter()
            .map(|cat| Violation {
                rule: cat.as_str().to_string(),
                severity: Severity::Critical,
                message: format!("Llama Guard violation: {}", cat.description()),
                location: None,
            })
            .collect();

        // Create provider-specific result (shows all violations, not just enabled ones)
        let llama_result = crate::guardrails::provider::LlamaGuardResult {
            safe: safe || filtered_violations.is_empty(), // Safe if no enabled categories violated
            violated_categories: filtered_violations
                .iter()
                .map(|c| c.as_str().to_string())
                .collect(),
            raw_response: response.to_string(),
        };

        Ok(GuardrailResult {
            passed: safe || filtered_violations.is_empty(),
            violations,
            warnings: vec![],
            quality_score: None, // Llama Guard is binary (no confidence scores)
            provider_specific: Some(ProviderSpecificResult::LlamaGuard(llama_result)),
        })
    }

    /// Parse comma-separated categories from response line
    fn parse_categories(&self, line: &str) -> Result<Vec<LlamaGuardCategory>, CliError> {
        line.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(LlamaGuardCategory::parse)
            .collect()
    }
}

#[async_trait]
impl GuardrailProvider for LlamaGuardProvider {
    async fn validate_input(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Llama Guard 3 is fine-tuned for safety classification and doesn't need
        // elaborate prompting. Just pass the raw content with an empty system prompt.
        let response = self
            .client
            .invoke(InvokeParams {
                model: &self.config.model,
                system_prompt: "", // Empty system prompt - model has built-in safety policy
                user_prompt: content, // Raw content to evaluate
                temperature: 0.0,  // Temperature 0 for deterministic safety checks
                max_tokens: Some(100), // Short response: "safe" or "unsafe\nS1,S3"
                seed: None,        // No seed needed for guardrails
                api_key: self.config.api_key.as_deref(),
                timeout_secs: self.config.timeout_secs,
                response_format: None, // No response_format needed for guardrails
            })
            .await?;

        self.parse_response(&response)
    }

    async fn validate_output(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Same as validate_input - Llama Guard 3 doesn't distinguish between
        // input and output in its classification, just evaluates content for safety.
        let response = self
            .client
            .invoke(InvokeParams {
                model: &self.config.model,
                system_prompt: "", // Empty system prompt - model has built-in safety policy
                user_prompt: content, // Raw content to evaluate
                temperature: 0.0,
                max_tokens: Some(100),
                seed: None, // No seed needed for guardrails
                api_key: self.config.api_key.as_deref(),
                timeout_secs: self.config.timeout_secs,
                response_format: None, // No response_format needed for guardrails
            })
            .await?;

        self.parse_response(&response)
    }

    fn name(&self) -> &str {
        "LlamaGuard3"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_all() {
        let categories = LlamaGuardCategory::all();
        assert_eq!(categories.len(), 14);
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(
            LlamaGuardCategory::parse("S1").unwrap(),
            LlamaGuardCategory::S1
        );
        assert_eq!(
            LlamaGuardCategory::parse("S10").unwrap(),
            LlamaGuardCategory::S10
        );
        assert_eq!(
            LlamaGuardCategory::parse("s13").unwrap(),
            LlamaGuardCategory::S13
        );
        assert_eq!(
            LlamaGuardCategory::parse("S14").unwrap(),
            LlamaGuardCategory::S14
        );
        assert_eq!(
            LlamaGuardCategory::parse("s14").unwrap(),
            LlamaGuardCategory::S14
        );
        assert!(LlamaGuardCategory::parse("S15").is_err());
        assert!(LlamaGuardCategory::parse("invalid").is_err());
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(LlamaGuardCategory::S1.as_str(), "S1");
        assert_eq!(LlamaGuardCategory::S10.as_str(), "S10");
        assert_eq!(LlamaGuardCategory::S13.as_str(), "S13");
        assert_eq!(LlamaGuardCategory::S14.as_str(), "S14");
    }

    #[test]
    fn test_category_description() {
        assert_eq!(LlamaGuardCategory::S1.description(), "Violent Crimes");
        assert_eq!(LlamaGuardCategory::S10.description(), "Hate Speech");
        assert_eq!(
            LlamaGuardCategory::S14.description(),
            "Code Interpreter Abuse"
        );
    }

    #[test]
    fn test_parse_response_safe() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("safe").unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
        assert!(result.quality_score.is_none());
    }

    #[test]
    fn test_parse_response_unsafe_single_category() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("unsafe\nS1").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "S1");
        assert_eq!(result.violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_parse_response_unsafe_multiple_categories() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("unsafe\nS1,S9,S10").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 3);
        assert_eq!(result.violations[0].rule, "S1");
        assert_eq!(result.violations[1].rule, "S9");
        assert_eq!(result.violations[2].rule, "S10");
    }

    #[test]
    fn test_parse_response_unsafe_with_spaces() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("unsafe\nS1, S9, S10").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 3);
    }

    #[test]
    fn test_parse_response_with_s14() {
        // Test S14 (Code Interpreter Abuse) category parsing
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("unsafe\nS14").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "S14");
        assert_eq!(result.violations[0].severity, Severity::Critical);
        assert!(result.violations[0]
            .message
            .contains("Code Interpreter Abuse"));
    }

    #[test]
    fn test_parse_response_empty() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let result = provider.parse_response("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_categories() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        let categories = provider.parse_categories("S1,S9").unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], LlamaGuardCategory::S1);
        assert_eq!(categories[1], LlamaGuardCategory::S9);
    }

    #[test]
    fn test_enabled_categories_filtering() {
        // Test that only enabled categories are reported as violations
        let config = LlamaGuardConfig {
            enabled_categories: vec![LlamaGuardCategory::S1, LlamaGuardCategory::S9],
            ..Default::default()
        };
        let provider = LlamaGuardProvider::new(config);

        // Response includes S1, S2, S9 - but only S1 and S9 are enabled
        let result = provider.parse_response("unsafe\nS1,S2,S9").unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2); // Only S1 and S9
        assert_eq!(result.violations[0].rule, "S1");
        assert_eq!(result.violations[1].rule, "S9");
    }

    #[test]
    fn test_all_violations_filtered_out() {
        // Test that if all violations are for disabled categories, result is safe
        let config = LlamaGuardConfig {
            enabled_categories: vec![LlamaGuardCategory::S1],
            ..Default::default()
        };
        let provider = LlamaGuardProvider::new(config);

        // Response includes S2, S3 - but only S1 is enabled
        let result = provider.parse_response("unsafe\nS2,S3").unwrap();
        assert!(result.passed); // Passed because no enabled categories violated
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_llama_guard_category_s14_parse() {
        assert_eq!(
            LlamaGuardCategory::parse("S14").unwrap(),
            LlamaGuardCategory::S14
        );
        assert_eq!(LlamaGuardCategory::S14.as_str(), "S14");
        assert_eq!(
            LlamaGuardCategory::S14.description(),
            "Code Interpreter Abuse"
        );
    }

    #[test]
    fn test_llama_guard_parse_response_with_s14() {
        let config = LlamaGuardConfig::default();
        let provider = LlamaGuardProvider::new(config);

        // Test unsafe response with S14
        let response = "unsafe\nS14";
        let result = provider.parse_response(response).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "S14");
        assert!(result.violations[0]
            .message
            .contains("Code Interpreter Abuse"));
    }
}
