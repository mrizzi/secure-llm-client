use crate::{
    error::CliError,
    guardrails::{
        config::{AggregationMode, ExecutionMode},
        provider::{GuardrailProvider, GuardrailResult},
    },
};
use async_trait::async_trait;

/// Composite guardrail combining multiple providers
pub struct HybridGuardrail {
    providers: Vec<Box<dyn GuardrailProvider>>,
    execution: ExecutionMode,
    aggregation: AggregationMode,
}

impl HybridGuardrail {
    pub fn new(
        providers: Vec<Box<dyn GuardrailProvider>>,
        execution: ExecutionMode,
        aggregation: AggregationMode,
    ) -> Self {
        Self {
            providers,
            execution,
            aggregation,
        }
    }

    /// Aggregate results from multiple providers based on aggregation mode
    fn aggregate_results(&self, results: Vec<GuardrailResult>) -> GuardrailResult {
        if results.is_empty() {
            return GuardrailResult {
                passed: true,
                violations: vec![],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            };
        }

        let passed = match self.aggregation {
            AggregationMode::AllMustPass => {
                // All must say "safe" for overall "safe" (conservative)
                results.iter().all(|r| r.passed)
            }
            AggregationMode::AnyCanPass => {
                // Any can say "safe" for overall "safe" (permissive)
                results.iter().any(|r| r.passed)
            }
        };

        // Merge all violations and warnings
        let mut violations = vec![];
        let mut warnings = vec![];

        for result in &results {
            violations.extend(result.violations.clone());
            warnings.extend(result.warnings.clone());
        }

        // Use first available quality_score
        let quality_score = results.iter().find_map(|r| r.quality_score);

        // Use first available provider_specific data
        let provider_specific = results.iter().find_map(|r| r.provider_specific.clone());

        GuardrailResult {
            passed,
            violations,
            warnings,
            quality_score,
            provider_specific,
        }
    }

    /// Validate content using the configured execution and aggregation strategy
    async fn validate_with_strategy(
        &self,
        content: &str,
        is_input: bool,
    ) -> Result<GuardrailResult, CliError> {
        match self.execution {
            ExecutionMode::Sequential => self.validate_sequential(content, is_input).await,
            ExecutionMode::Parallel => self.validate_parallel(content, is_input).await,
        }
    }

    /// Sequential execution (can short-circuit based on aggregation mode)
    async fn validate_sequential(
        &self,
        content: &str,
        is_input: bool,
    ) -> Result<GuardrailResult, CliError> {
        let mut results = Vec::new();

        for provider in &self.providers {
            let result = if is_input {
                provider.validate_input(content).await?
            } else {
                provider.validate_output(content).await?
            };

            let can_short_circuit = match self.aggregation {
                // AllMustPass: short-circuit on first failure
                AggregationMode::AllMustPass => !result.passed,
                // AnyCanPass: short-circuit on first success
                AggregationMode::AnyCanPass => result.passed,
            };

            results.push(result);

            if can_short_circuit {
                log::debug!(
                    "Short-circuiting sequential validation after {} providers (aggregation={:?})",
                    results.len(),
                    self.aggregation
                );
                break;
            }
        }

        Ok(self.aggregate_results(results))
    }

    /// Parallel execution (all providers run simultaneously)
    async fn validate_parallel(
        &self,
        content: &str,
        is_input: bool,
    ) -> Result<GuardrailResult, CliError> {
        // Handle empty providers gracefully
        if self.providers.is_empty() {
            return Ok(self.aggregate_results(vec![]));
        }

        // Execute all providers in parallel
        let futures: Vec<_> = self
            .providers
            .iter()
            .map(|provider| {
                if is_input {
                    provider.validate_input(content)
                } else {
                    provider.validate_output(content)
                }
            })
            .collect();

        // Wait for all to complete
        let results = futures::future::join_all(futures).await;

        // Collect successes, log failures
        let mut successes = Vec::new();
        for (idx, result) in results.into_iter().enumerate() {
            match result {
                Ok(r) => successes.push(r),
                Err(e) => {
                    log::warn!(
                        "Provider {} failed during parallel execution: {}",
                        self.providers[idx].name(),
                        e
                    );
                }
            }
        }

        if successes.is_empty() {
            return Err(CliError::InvalidResponse(
                "All providers failed during parallel execution".to_string(),
            ));
        }

        Ok(self.aggregate_results(successes))
    }
}

#[async_trait]
impl GuardrailProvider for HybridGuardrail {
    async fn validate_input(&self, content: &str) -> Result<GuardrailResult, CliError> {
        self.validate_with_strategy(content, true).await
    }

    async fn validate_output(&self, content: &str) -> Result<GuardrailResult, CliError> {
        self.validate_with_strategy(content, false).await
    }

    fn name(&self) -> &str {
        "CompositeGuardrail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardrails::{
        input::{InputGuardrail, InputGuardrailConfig},
        provider::{Severity, Violation},
    };

    #[tokio::test]
    async fn test_sequential_all_must_pass() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
        ];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Sequential,
            AggregationMode::AllMustPass,
        );

        // Clean input should pass all providers
        let result = composite.validate_input("Clean input").await.unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_sequential_all_must_pass_short_circuit() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
        ];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Sequential,
            AggregationMode::AllMustPass,
        );

        // PII should fail first provider and short-circuit
        let result = composite.validate_input("SSN: 123-45-6789").await.unwrap();
        assert!(!result.passed);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_parallel_all_must_pass() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
        ];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        // Clean input should pass all providers
        let result = composite.validate_input("Clean input").await.unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_parallel_all_must_pass_failure() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
        ];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        // PII should fail both providers
        let result = composite.validate_input("SSN: 123-45-6789").await.unwrap();
        assert!(!result.passed);
        // Violations from both providers should be aggregated
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_aggregate_all_must_pass() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![Box::new(InputGuardrail::new(
            InputGuardrailConfig::default(),
        ))];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        let results = vec![
            GuardrailResult {
                passed: true,
                violations: vec![],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
            GuardrailResult {
                passed: true,
                violations: vec![],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
        ];

        let aggregated = composite.aggregate_results(results);
        assert!(aggregated.passed); // All passed → aggregate passes
    }

    #[tokio::test]
    async fn test_aggregate_all_must_pass_one_fails() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![Box::new(InputGuardrail::new(
            InputGuardrailConfig::default(),
        ))];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        let results = vec![
            GuardrailResult {
                passed: true,
                violations: vec![],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
            GuardrailResult {
                passed: false,
                violations: vec![Violation {
                    rule: "TEST".to_string(),
                    severity: Severity::Critical,
                    message: "Test violation".to_string(),
                    location: None,
                }],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
        ];

        let aggregated = composite.aggregate_results(results);
        assert!(!aggregated.passed); // One failed → aggregate fails
        assert_eq!(aggregated.violations.len(), 1);
    }

    #[tokio::test]
    async fn test_aggregate_any_can_pass() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![Box::new(InputGuardrail::new(
            InputGuardrailConfig::default(),
        ))];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AnyCanPass,
        );

        let results = vec![
            GuardrailResult {
                passed: false,
                violations: vec![Violation {
                    rule: "TEST".to_string(),
                    severity: Severity::Critical,
                    message: "Test violation".to_string(),
                    location: None,
                }],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
            GuardrailResult {
                passed: true,
                violations: vec![],
                warnings: vec![],
                quality_score: None,
                provider_specific: None,
            },
        ];

        let aggregated = composite.aggregate_results(results);
        assert!(aggregated.passed); // At least one passed → aggregate passes
        assert_eq!(aggregated.violations.len(), 1); // But violations still aggregated
    }

    #[tokio::test]
    async fn test_three_providers_composite() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
            Box::new(InputGuardrail::new(InputGuardrailConfig::default())),
        ];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        // Clean input should pass all three providers
        let result = composite.validate_input("Clean input").await.unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_empty_providers() {
        let providers: Vec<Box<dyn GuardrailProvider>> = vec![];

        let composite = HybridGuardrail::new(
            providers,
            ExecutionMode::Parallel,
            AggregationMode::AllMustPass,
        );

        // Empty providers should default to "safe"
        let result = composite.validate_input("Any input").await.unwrap();
        assert!(result.passed);
    }
}
