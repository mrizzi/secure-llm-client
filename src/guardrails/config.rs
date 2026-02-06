use crate::{
    error::CliError,
    guardrails::{
        gpt_oss_safeguard::GptOssSafeguardConfig,
        llama_guard::{LlamaGuardCategory, LlamaGuardConfig},
        provider::Severity,
    },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Regex guardrail configuration (unified for both input and output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexGuardrailConfig {
    /// Maximum content length in bytes
    pub max_length_bytes: usize,

    /// Optional user-provided patterns file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns_file: Option<PathBuf>,

    /// Minimum severity to report (violations below this become warnings)
    #[serde(default = "default_severity_threshold")]
    pub severity_threshold: Severity,
}

fn default_severity_threshold() -> Severity {
    Severity::Medium
}

impl Default for RegexGuardrailConfig {
    fn default() -> Self {
        Self {
            max_length_bytes: 1048576, // 1MB
            patterns_file: None,
            severity_threshold: Severity::Medium,
        }
    }
}

/// Default function for LlamaGuard enabled_categories (all categories enabled)
fn default_llama_guard_categories() -> Vec<LlamaGuardCategory> {
    LlamaGuardCategory::all()
}

/// Default threshold for Llama Prompt Guard 2 (balanced)
fn default_prompt_guard_threshold() -> f32 {
    0.5
}

/// Helper function to resolve API key from either direct value or environment variable
/// Returns the resolved API key value, or None if neither is specified
fn resolve_api_key(
    api_key: &Option<String>,
    api_key_name: &Option<String>,
    provider_name: &str,
) -> Result<Option<String>, CliError> {
    match (api_key, api_key_name) {
        (Some(_), Some(_)) => Err(CliError::InvalidArguments(format!(
            "Guardrail provider '{provider_name}' cannot specify both 'api_key' and 'api_key_name'"
        ))),
        (Some(key), None) => {
            log::debug!("Guardrail provider '{provider_name}': Using direct API key value");
            Ok(Some(key.clone()))
        }
        (None, Some(env_var_name)) => {
            log::debug!(
                "Guardrail provider '{provider_name}': Loading API key from environment variable '{env_var_name}'"
            );
            match std::env::var(env_var_name) {
                Ok(key) => Ok(Some(key)),
                Err(_) => Err(CliError::InvalidArguments(format!(
                    "Guardrail provider '{provider_name}': Environment variable '{env_var_name}' specified by 'api_key_name' does not exist"
                ))),
            }
        }
        (None, None) => {
            log::debug!(
                "Guardrail provider '{provider_name}': No API key configured (using unauthenticated endpoint)"
            );
            Ok(None)
        }
    }
}

/// Execution mode for composite guardrails
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Run providers one at a time (can short-circuit)
    Sequential,

    /// Run all providers simultaneously
    #[default]
    Parallel, // Faster, maximizes throughput
}

/// Aggregation mode for composite guardrails
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AggregationMode {
    /// All providers must say "safe" for overall "safe" (conservative)
    #[default]
    AllMustPass, // Conservative, safer for guardrails

    /// Any provider can say "safe" for overall "safe" (permissive)
    AnyCanPass,
}

/// Top-level guardrail configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    /// Input guardrails configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<GuardrailProviderConfig>,

    /// Output guardrails configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<GuardrailProviderConfig>,

    /// Flattened provider field for unified guardrail configuration
    /// When explicit input/output fields are not specified, this applies to BOTH
    /// input and output guardrails. Explicit fields take precedence.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<GuardrailProviderConfig>,
}

/// Unified provider-specific configuration (works for both input and output)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardrailProviderConfig {
    /// Regex-based guardrail (unified for input and output)
    Regex(RegexGuardrailConfig),

    /// Llama Guard 3 (MLCommons taxonomy, fixed categories)
    LlamaGuard {
        api_url: String,
        model: String,
        timeout_secs: u64,
        #[serde(default = "default_llama_guard_categories")]
        enabled_categories: Vec<LlamaGuardCategory>,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key_name: Option<String>,
    },

    /// GPT-OSS-Safeguard (policy-driven reasoning model)
    GptOssSafeguard {
        api_url: String,
        model: String,
        policy: String,
        timeout_secs: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key_name: Option<String>,
    },

    /// Llama Prompt Guard 2 (prompt injection detection, input-only)
    /// When used for output guardrails, this is gracefully ignored with a warning
    LlamaPromptGuard {
        api_url: String,
        model: String,
        timeout_secs: u64,
        #[serde(default = "default_prompt_guard_threshold")]
        threshold: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key_name: Option<String>,
    },

    /// Composite guardrail (combines multiple providers)
    Composite {
        providers: Vec<GuardrailProviderConfig>,
        execution: ExecutionMode,
        aggregation: AggregationMode,
    },
}

impl Default for GuardrailProviderConfig {
    fn default() -> Self {
        // Default: Simple regex guardrail with default settings
        Self::Regex(RegexGuardrailConfig::default())
    }
}

impl GuardrailProviderConfig {
    /// Get the RegexGuardrailConfig (if this is a Regex variant)
    pub fn as_regex_config(&self) -> Option<&RegexGuardrailConfig> {
        match self {
            Self::Regex(config) => Some(config),
            _ => None,
        }
    }

    /// Convert to LlamaGuardConfig (if this is a LlamaGuard config)
    pub fn to_llama_guard_config(&self) -> Option<LlamaGuardConfig> {
        match self {
            Self::LlamaGuard {
                api_url,
                model,
                timeout_secs,
                enabled_categories,
                api_key,
                ..
            } => Some(LlamaGuardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                enabled_categories: enabled_categories.clone(),
                timeout_secs: *timeout_secs,
                api_key: api_key.clone(),
            }),
            _ => None,
        }
    }

    /// Convert to GptOssSafeguardConfig (if this is a GptOssSafeguard config)
    pub fn to_gpt_oss_safeguard_config(&self) -> Option<GptOssSafeguardConfig> {
        match self {
            Self::GptOssSafeguard {
                api_url,
                model,
                policy,
                timeout_secs,
                api_key,
                ..
            } => Some(GptOssSafeguardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                policy: policy.clone(),
                timeout_secs: *timeout_secs,
                api_key: api_key.clone(),
            }),
            _ => None,
        }
    }
}

/// Factory function to create GuardrailProvider from configuration
pub fn create_guardrail_provider(
    config: &GuardrailProviderConfig,
) -> Result<Box<dyn crate::guardrails::provider::GuardrailProvider>, crate::error::CliError> {
    use crate::guardrails::{
        gpt_oss_safeguard::GptOssSafeguardProvider, hybrid::HybridGuardrail,
        llama_guard::LlamaGuardProvider, regex::RegexGuardrail,
    };

    match config {
        GuardrailProviderConfig::Regex(regex_config) => {
            Ok(Box::new(RegexGuardrail::new(regex_config.clone())))
        }

        GuardrailProviderConfig::LlamaGuard {
            api_url,
            model,
            timeout_secs,
            enabled_categories,
            api_key,
            api_key_name,
        } => {
            let resolved_api_key = resolve_api_key(api_key, api_key_name, "LlamaGuard")?;
            let llama_config = LlamaGuardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                enabled_categories: enabled_categories.clone(),
                timeout_secs: *timeout_secs,
                api_key: resolved_api_key,
            };
            Ok(Box::new(LlamaGuardProvider::new(llama_config)))
        }

        GuardrailProviderConfig::GptOssSafeguard {
            api_url,
            model,
            policy,
            timeout_secs,
            api_key,
            api_key_name,
        } => {
            let resolved_api_key = resolve_api_key(api_key, api_key_name, "GptOssSafeguard")?;
            let gpt_oss_config = GptOssSafeguardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                policy: policy.clone(),
                timeout_secs: *timeout_secs,
                api_key: resolved_api_key,
            };
            Ok(Box::new(GptOssSafeguardProvider::new(gpt_oss_config)))
        }

        GuardrailProviderConfig::LlamaPromptGuard {
            api_url,
            model,
            timeout_secs,
            threshold,
            api_key,
            api_key_name,
        } => {
            let resolved_api_key = resolve_api_key(api_key, api_key_name, "LlamaPromptGuard")?;
            let prompt_guard_config =
                crate::guardrails::llama_prompt_guard::LlamaPromptGuardConfig {
                    api_url: api_url.clone(),
                    model: model.clone(),
                    timeout_secs: *timeout_secs,
                    threshold: *threshold,
                    api_key: resolved_api_key,
                    api_key_name: None, // Already resolved to api_key
                };
            Ok(Box::new(
                crate::guardrails::llama_prompt_guard::LlamaPromptGuardProvider::new(
                    prompt_guard_config,
                ),
            ))
        }

        GuardrailProviderConfig::Composite {
            providers,
            execution,
            aggregation,
        } => {
            // Recursively create all providers
            let provider_instances: Result<Vec<_>, _> =
                providers.iter().map(create_guardrail_provider).collect();

            Ok(Box::new(HybridGuardrail::new(
                provider_instances?,
                *execution,
                *aggregation,
            )))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode_default() {
        assert_eq!(ExecutionMode::default(), ExecutionMode::Parallel);
    }

    #[test]
    fn test_aggregation_mode_default() {
        assert_eq!(AggregationMode::default(), AggregationMode::AllMustPass);
    }

    #[test]
    fn test_guardrail_config_default() {
        let config = GuardrailProviderConfig::default();
        match config {
            GuardrailProviderConfig::Regex(regex_config) => {
                assert_eq!(regex_config.max_length_bytes, 1048576);
                assert_eq!(regex_config.severity_threshold, Severity::Medium);
                assert!(regex_config.patterns_file.is_none());
            }
            _ => panic!("Default should be Regex"),
        }
    }

    #[test]
    fn test_as_regex_config() {
        let config = GuardrailProviderConfig::Regex(RegexGuardrailConfig {
            max_length_bytes: 1024,
            patterns_file: None,
            severity_threshold: Severity::High,
        });

        let regex_config = config.as_regex_config().unwrap();
        assert_eq!(regex_config.max_length_bytes, 1024);
        assert_eq!(regex_config.severity_threshold, Severity::High);
    }

    #[test]
    fn test_to_llama_guard_config() {
        let config = GuardrailProviderConfig::LlamaGuard {
            api_url: "http://test:8080".to_string(),
            model: "test-model".to_string(),
            timeout_secs: 60,
            enabled_categories: vec![LlamaGuardCategory::S1, LlamaGuardCategory::S9],
            api_key: None,
            api_key_name: None,
        };

        let llama_config = config.to_llama_guard_config().unwrap();
        assert_eq!(llama_config.api_url, "http://test:8080");
        assert_eq!(llama_config.model, "test-model");
        assert_eq!(llama_config.timeout_secs, 60);
        assert_eq!(llama_config.enabled_categories.len(), 2);
        assert_eq!(llama_config.api_key, None);
    }

    #[test]
    fn test_serde_regex_config() {
        let config = GuardrailProviderConfig::Regex(RegexGuardrailConfig {
            max_length_bytes: 1024,
            patterns_file: None,
            severity_threshold: Severity::High,
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"regex\""));
        assert!(json.contains("\"max_length_bytes\":1024"));

        let deserialized: GuardrailProviderConfig = serde_json::from_str(&json).unwrap();
        match deserialized {
            GuardrailProviderConfig::Regex(regex_config) => {
                assert_eq!(regex_config.max_length_bytes, 1024);
            }
            _ => panic!("Should deserialize to Regex"),
        }
    }

    #[test]
    fn test_serde_llama_guard_config() {
        let config = GuardrailProviderConfig::LlamaGuard {
            api_url: "http://localhost:11434".to_string(),
            model: "llama-guard3:8b".to_string(),
            timeout_secs: 30,
            enabled_categories: vec![LlamaGuardCategory::S1],
            api_key: None,
            api_key_name: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"llama_guard\""));
        assert!(json.contains("\"model\":\"llama-guard3:8b\""));
    }

    #[test]
    fn test_serde_composite_config() {
        let config = GuardrailProviderConfig::Composite {
            providers: vec![
                GuardrailProviderConfig::Regex(RegexGuardrailConfig {
                    max_length_bytes: 1024,
                    patterns_file: None,
                    severity_threshold: Severity::Medium,
                }),
                GuardrailProviderConfig::LlamaGuard {
                    api_url: "http://localhost:11434".to_string(),
                    model: "llama-guard3:8b".to_string(),
                    timeout_secs: 30,
                    enabled_categories: vec![LlamaGuardCategory::S1],
                    api_key: None,
                    api_key_name: None,
                },
            ],
            execution: ExecutionMode::Parallel,
            aggregation: AggregationMode::AllMustPass,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"composite\""));
        assert!(json.contains("\"execution\":\"parallel\""));
        assert!(json.contains("\"aggregation\":\"all_must_pass\""));
    }

    #[test]
    fn test_resolve_api_key_direct_value() {
        let result = resolve_api_key(&Some("test-key".to_string()), &None, "TestProvider");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("test-key".to_string()));
    }

    #[test]
    fn test_resolve_api_key_env_var() {
        std::env::set_var("TEST_GUARDRAIL_KEY", "env-test-key");
        let result = resolve_api_key(
            &None,
            &Some("TEST_GUARDRAIL_KEY".to_string()),
            "TestProvider",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("env-test-key".to_string()));
        std::env::remove_var("TEST_GUARDRAIL_KEY");
    }

    #[test]
    fn test_resolve_api_key_none() {
        let result = resolve_api_key(&None, &None, "TestProvider");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_resolve_api_key_both_specified_error() {
        let result = resolve_api_key(
            &Some("test-key".to_string()),
            &Some("TEST_ENV_VAR".to_string()),
            "TestProvider",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot specify both"));
    }

    #[test]
    fn test_resolve_api_key_missing_env_var_error() {
        let result = resolve_api_key(
            &None,
            &Some("NONEXISTENT_GUARDRAIL_KEY".to_string()),
            "TestProvider",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
