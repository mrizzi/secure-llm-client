use crate::error::CliError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Generic guardrail provider trait for extensibility
#[async_trait]
pub trait GuardrailProvider: Send + Sync {
    /// Validate content (works for both input and output)
    async fn validate(&self, content: &str) -> Result<GuardrailResult, CliError>;

    /// Provider name for logging and debugging
    fn name(&self) -> &str;
}

/// Generic validation result (unified for all providers)
#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<Violation>,

    /// Quality score (0.0-10.0) if provider supports it
    /// - Input/regex providers: None
    /// - Output guardrails: Some(score)
    pub quality_score: Option<f32>,

    /// Provider-specific metadata
    pub provider_specific: Option<ProviderSpecificResult>,
}

impl GuardrailResult {
    /// Helper: Create result without quality score (for input/regex providers)
    pub fn without_quality_score(
        passed: bool,
        violations: Vec<Violation>,
        warnings: Vec<Violation>,
    ) -> Self {
        Self {
            passed,
            violations,
            warnings,
            quality_score: None,
            provider_specific: None,
        }
    }

    /// Helper: Create result with quality score (for output guardrails)
    pub fn with_quality_score(
        passed: bool,
        violations: Vec<Violation>,
        warnings: Vec<Violation>,
        quality_score: f32,
    ) -> Self {
        Self {
            passed,
            violations,
            warnings,
            quality_score: Some(quality_score),
            provider_specific: None,
        }
    }

    /// Helper: Create result with provider-specific data (for LLM providers)
    pub fn with_provider_specific(
        passed: bool,
        violations: Vec<Violation>,
        warnings: Vec<Violation>,
        provider_specific: ProviderSpecificResult,
    ) -> Self {
        Self {
            passed,
            violations,
            warnings,
            quality_score: None,
            provider_specific: Some(provider_specific),
        }
    }
}

/// Provider-specific result data
#[derive(Debug, Clone)]
pub enum ProviderSpecificResult {
    LlamaGuard(LlamaGuardResult),
    GptOssSafeguard(GptOssSafeguardResult),
    LlamaPromptGuard(crate::guardrails::llama_prompt_guard::LlamaPromptGuardResult),
    // Future: OpenAI(OpenAIModerationResult),
    // Future: Azure(AzureContentSafetyResult),
}

/// Llama Guard 3 specific result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaGuardResult {
    pub safe: bool,
    pub violated_categories: Vec<String>,
    pub raw_response: String,
}

/// GPT-OSS-Safeguard specific result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GptOssSafeguardResult {
    pub violation: bool,
    pub category: Option<String>,
    pub rationale: Option<String>,
    pub raw_response: String,
}

/// Violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
}

/// Violation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}
