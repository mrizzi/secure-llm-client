use crate::{
    error::CliError,
    guardrails::{
        config::RegexGuardrailConfig,
        patterns::{load_patterns_from_file, PatternDefinition},
        provider::{GuardrailProvider, GuardrailResult, Severity, Violation},
    },
};
use async_trait::async_trait;

/// Unified regex-based guardrail for both input and output validation
pub struct RegexGuardrail {
    config: RegexGuardrailConfig,
    patterns: Vec<PatternDefinition>,
}

impl RegexGuardrail {
    /// Create a new regex guardrail
    pub fn new(config: RegexGuardrailConfig) -> Self {
        // Load user-provided patterns if file is provided
        let patterns = if let Some(ref path) = config.patterns_file {
            match load_patterns_from_file(path) {
                Ok(patterns) => {
                    log::info!("Loaded {} patterns from {:?}", patterns.len(), path);
                    patterns
                }
                Err(e) => {
                    log::warn!("Failed to load patterns from {path:?}: {e}. No pattern validation will be performed.");
                    Vec::new()
                }
            }
        } else {
            log::debug!("No patterns file provided, only length validation will be performed");
            Vec::new()
        };

        Self { config, patterns }
    }

    /// Internal validation logic
    async fn validate_internal(&self, content: &str) -> Result<GuardrailResult, CliError> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // 1. Length validation
        if content.len() > self.config.max_length_bytes {
            violations.push(Violation {
                rule: "MAX_LENGTH".to_string(),
                severity: Severity::High,
                message: format!(
                    "Content exceeds max length ({} > {} bytes)",
                    content.len(),
                    self.config.max_length_bytes
                ),
                location: None,
            });
        }

        // 2. Pattern validation (simple pattern matching)
        for pattern_def in &self.patterns {
            if let Some(mat) = pattern_def.regex.find(content) {
                let violation = Violation {
                    rule: pattern_def.description.to_uppercase().replace(' ', "_"),
                    severity: pattern_def.severity,
                    message: format!("Matched: {}", pattern_def.description),
                    location: Some(format!("Position {}", mat.start())),
                };

                // Respect user-configured severity threshold
                if pattern_def.severity >= self.config.severity_threshold {
                    violations.push(violation);
                } else {
                    warnings.push(violation);
                }
            }
        }

        let passed = violations.is_empty();
        Ok(GuardrailResult::without_quality_score(
            passed, violations, warnings,
        ))
    }
}

#[async_trait]
impl GuardrailProvider for RegexGuardrail {
    async fn validate(&self, content: &str) -> Result<GuardrailResult, CliError> {
        self.validate_internal(content).await
    }

    fn name(&self) -> &str {
        "RegexGuardrail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_input_clean() {
        let config = RegexGuardrailConfig::default();
        let guardrail = RegexGuardrail::new(config);

        let result = guardrail.validate("Clean input").await.unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[tokio::test]
    async fn test_output_clean() {
        let config = RegexGuardrailConfig::default();
        let guardrail = RegexGuardrail::new(config);

        let result = guardrail
            .validate("This is a clean, factual response.")
            .await
            .unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[tokio::test]
    async fn test_input_max_length() {
        let config = RegexGuardrailConfig {
            max_length_bytes: 10,
            patterns_file: None,
            severity_threshold: Severity::Medium,
        };
        let guardrail = RegexGuardrail::new(config);

        let result = guardrail
            .validate("This is a long input that exceeds 10 bytes")
            .await
            .unwrap();
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "MAX_LENGTH"));
        assert!(result
            .violations
            .iter()
            .any(|v| v.severity == Severity::High));
        assert!(result
            .violations
            .iter()
            .any(|v| v.message.contains("Content exceeds")));
    }

    #[tokio::test]
    async fn test_output_max_length() {
        let config = RegexGuardrailConfig {
            max_length_bytes: 10,
            patterns_file: None,
            severity_threshold: Severity::Medium,
        };
        let guardrail = RegexGuardrail::new(config);

        let long_output = "This is a very long output that exceeds the limit";
        let result = guardrail.validate(long_output).await.unwrap();
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "MAX_LENGTH"));
        assert!(result
            .violations
            .iter()
            .any(|v| v.severity == Severity::High));
        assert!(result
            .violations
            .iter()
            .any(|v| v.message.contains("Content exceeds")));
    }

    #[tokio::test]
    async fn test_severity_threshold() {
        let config = RegexGuardrailConfig {
            max_length_bytes: 100000,
            patterns_file: None,
            severity_threshold: Severity::High,
        };
        let guardrail = RegexGuardrail::new(config);

        let result = guardrail.validate("clean input").await.unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_name() {
        let config = RegexGuardrailConfig::default();
        let guardrail = RegexGuardrail::new(config);
        assert_eq!(guardrail.name(), "RegexGuardrail");
    }
}
