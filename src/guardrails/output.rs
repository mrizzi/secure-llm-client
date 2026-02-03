use crate::{
    constants::{guardrails, output_limits},
    error::CliError,
    guardrails::{
        patterns::{load_patterns_from_file, PatternDefinition},
        provider::{GuardrailProvider, GuardrailResult, Severity, Violation},
    },
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputGuardrailConfig {
    pub max_length_bytes: usize,
    pub check_safety: bool,
    pub check_hallucination: bool,
    pub check_format: bool,
    pub min_quality_score: f32,
    /// Optional path to external pattern file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_patterns_file: Option<PathBuf>,
}

impl Default for OutputGuardrailConfig {
    fn default() -> Self {
        Self {
            max_length_bytes: output_limits::MAX_OUTPUT_BYTES,
            check_safety: true,
            check_hallucination: true,
            check_format: true,
            min_quality_score: guardrails::DEFAULT_MIN_QUALITY_SCORE,
            custom_patterns_file: None,
        }
    }
}

pub struct OutputGuardrail {
    config: OutputGuardrailConfig,
    default_patterns: Vec<PatternDefinition>,
    custom_patterns: Vec<PatternDefinition>,
}

impl OutputGuardrail {
    pub fn new(config: OutputGuardrailConfig) -> Self {
        // Load default patterns from embedded file
        const DEFAULT_PATTERNS_FILE: &str =
            include_str!("default_patterns/default_output_patterns.txt");
        let default_patterns =
            match crate::guardrails::patterns::parse_patterns(DEFAULT_PATTERNS_FILE) {
                Ok(mut patterns) => {
                    patterns.retain(|p| p.applies_to_output());
                    log::debug!("Loaded {} default output patterns", patterns.len());
                    patterns
                }
                Err(e) => {
                    log::error!(
                        "Failed to parse default output patterns: {e}. This should never happen!"
                    );
                    Vec::new()
                }
            };

        // Load custom patterns if file is provided
        let custom_patterns = if let Some(ref path) = config.custom_patterns_file {
            match load_patterns_from_file(path) {
                Ok(mut patterns) => {
                    // Filter to only patterns applicable to output
                    patterns.retain(|p| p.applies_to_output());
                    log::info!(
                        "Loaded {} custom output patterns from {:?}",
                        patterns.len(),
                        path
                    );
                    patterns
                }
                Err(e) => {
                    log::warn!("Failed to load custom patterns from {path:?}: {e}. Using default patterns only.");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Self {
            config,
            default_patterns,
            custom_patterns,
        }
    }

    fn calculate_quality_score(response: &str, hallucination_count: usize) -> f32 {
        let base_score = 10.0;
        let hallucination_penalty = (hallucination_count as f32) * 0.2;
        let length_bonus = if response.len() > 100 { 0.5 } else { 0.0 };

        (base_score - hallucination_penalty + length_bonus).clamp(0.0, 10.0)
    }
}

#[async_trait]
impl GuardrailProvider for OutputGuardrail {
    async fn validate_input(&self, content: &str) -> Result<GuardrailResult, CliError> {
        // Output guardrail can also validate input with same logic
        self.validate_output(content).await
    }

    async fn validate_output(&self, content: &str) -> Result<GuardrailResult, CliError> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // 1. Length check
        if content.len() > self.config.max_length_bytes {
            violations.push(Violation {
                rule: "MAX_LENGTH".to_string(),
                severity: Severity::High,
                message: format!("Response exceeds max length ({} bytes)", content.len()),
                location: None,
            });
        }

        // 2. Default pattern validation (from embedded pattern file)
        let mut hallucination_count = 0;
        for pattern_def in &self.default_patterns {
            let desc_lower = pattern_def.description.to_lowercase();

            // Determine if this is a safety pattern
            let is_safety = desc_lower.contains("bomb")
                || desc_lower.contains("illegal")
                || desc_lower.contains("hack")
                || desc_lower.contains("dangerous");

            // Determine if this is a hallucination marker
            let is_hallucination = desc_lower.contains("uncertainty")
                || desc_lower.contains("citation")
                || desc_lower.contains("speculative")
                || desc_lower.contains("vague");

            // Skip safety checks if disabled
            if !self.config.check_safety && is_safety {
                continue;
            }

            // Handle hallucination patterns differently - only count, don't create individual violations
            if is_hallucination {
                // Skip hallucination checks if disabled
                if !self.config.check_hallucination {
                    continue;
                }
                // Count hallucination matches for quality scoring
                hallucination_count += pattern_def.regex.find_iter(content).count();
                continue; // Don't create individual violations for each match
            }

            // For safety patterns, create violations
            if let Some(mat) = pattern_def.regex.find(content) {
                let violation = Violation {
                    rule: "SAFETY".to_string(),
                    severity: pattern_def.severity,
                    message: format!("Detected: {}", pattern_def.description),
                    location: Some(format!("Position {}", mat.start())),
                };

                if pattern_def.severity >= Severity::Critical {
                    violations.push(violation);
                } else {
                    warnings.push(violation);
                }
            }
        }

        // 3. Hallucination warning if high count
        if self.config.check_hallucination && hallucination_count > 5 {
            warnings.push(Violation {
                rule: "HALLUCINATION".to_string(),
                severity: Severity::Medium,
                message: format!("High uncertainty markers: {hallucination_count}"),
                location: None,
            });
        }

        // 4. Custom patterns (from external file)
        for pattern_def in &self.custom_patterns {
            if let Some(mat) = pattern_def.regex.find(content) {
                let violation = Violation {
                    rule: format!(
                        "CUSTOM_{}",
                        pattern_def.description.to_uppercase().replace(' ', "_")
                    ),
                    severity: pattern_def.severity,
                    message: format!("Custom pattern matched: {}", pattern_def.description),
                    location: Some(format!("Position {}", mat.start())),
                };

                if pattern_def.severity >= Severity::Critical {
                    violations.push(violation);
                } else {
                    warnings.push(violation);
                }
            }
        }

        // 5. Quality scoring (0-10)
        let quality_score = Self::calculate_quality_score(content, hallucination_count);
        if quality_score < self.config.min_quality_score {
            warnings.push(Violation {
                rule: "QUALITY".to_string(),
                severity: Severity::Low,
                message: format!("Quality score below threshold: {quality_score:.1}/10"),
                location: None,
            });
        }

        let passed = violations.is_empty();

        // CRITICAL: Use with_quality_score() to preserve quality score
        Ok(GuardrailResult::with_quality_score(
            passed,
            violations,
            warnings,
            quality_score,
        ))
    }

    fn name(&self) -> &str {
        "RegexOutputGuardrail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_output() {
        let config = OutputGuardrailConfig::default();
        let guardrail = OutputGuardrail::new(config);

        let result = guardrail
            .validate_output("This is a clean, factual response.")
            .await
            .unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
        assert!(result.quality_score.is_some());
        assert!(result.quality_score.unwrap() > 9.0);
    }

    #[tokio::test]
    async fn test_safety_violation() {
        let config = OutputGuardrailConfig::default();
        let guardrail = OutputGuardrail::new(config);

        let result = guardrail
            .validate_output("Here's how to build a bomb...")
            .await
            .unwrap();
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "SAFETY"));
    }

    #[tokio::test]
    async fn test_hallucination_warning() {
        let config = OutputGuardrailConfig::default();
        let guardrail = OutputGuardrail::new(config);

        let result = guardrail
            .validate_output(
                "I think maybe possibly this might be correct, I believe, probably true...",
            )
            .await
            .unwrap();
        assert!(result.passed); // Hallucinations are warnings, not violations
        assert!(result.warnings.iter().any(|v| v.rule == "HALLUCINATION"));
    }

    #[tokio::test]
    async fn test_quality_score() {
        let config = OutputGuardrailConfig::default();
        let guardrail = OutputGuardrail::new(config);

        let result = guardrail
            .validate_output("High quality response with no issues.")
            .await
            .unwrap();
        assert!(result.quality_score.is_some());
        assert!(result.quality_score.unwrap() > 9.0);
    }

    #[tokio::test]
    async fn test_quality_score_preservation() {
        // Verify quality_score is always present and calculated correctly
        let config = OutputGuardrailConfig {
            min_quality_score: 7.0,
            ..Default::default()
        };
        let guardrail = OutputGuardrail::new(config);

        // Text with many hallucination markers to ensure score < 7.0
        // Need 16+ markers (penalty of 3.2+) to get below 7.0
        let low_quality = "I think maybe possibly this might be correct, I believe probably maybe possibly this might be true, I think probably maybe this could be right, maybe possibly I think this might be accurate, probably maybe I think";
        let result = guardrail.validate_output(low_quality).await.unwrap();

        // Should have quality score
        assert!(result.quality_score.is_some());
        let score = result.quality_score.unwrap();
        assert!(score < 7.0); // Below threshold

        // Should have warning about low quality
        assert!(result.warnings.iter().any(|v| v.rule == "QUALITY"));
    }
}
