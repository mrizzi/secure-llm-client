use crate::{
    error::CliError,
    guardrails::{
        gpt_oss_safeguard::GptOssSafeguardConfig,
        input::InputGuardrailConfig,
        llama_guard::{LlamaGuardCategory, LlamaGuardConfig},
    },
};
use serde::{Deserialize, Serialize};

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
    #[serde(flatten)]
    pub provider: GuardrailProviderConfig,
}

/// Provider-specific configuration (tagged enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardrailProviderConfig {
    /// Regex-based guardrail (fast, deterministic)
    Regex {
        max_length_bytes: usize,
        max_tokens_estimated: usize,
        check_pii: bool,
        check_content_filters: bool,
    },

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

    /// Llama Prompt Guard 2 (prompt injection detection)
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
        // Default: Composite (regex + llama-guard, parallel, all must pass)
        Self::Composite {
            providers: vec![
                Self::Regex {
                    max_length_bytes: 1_048_576, // 1MB
                    max_tokens_estimated: 200_000,
                    check_pii: true,
                    check_content_filters: true,
                },
                Self::LlamaGuard {
                    api_url: "http://localhost:11434/api/generate".to_string(),
                    model: "llama-guard3:8b".to_string(),
                    timeout_secs: 30,
                    enabled_categories: LlamaGuardCategory::all(),
                    api_key: None,
                    api_key_name: None,
                },
            ],
            execution: ExecutionMode::default(),
            aggregation: AggregationMode::default(),
        }
    }
}

/// Output guardrail provider configuration (tagged enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputGuardrailProviderConfig {
    /// Regex-based output validation (fast, pattern matching)
    Regex {
        max_length_bytes: usize,
        check_safety: bool,
        check_hallucination: bool,
        check_format: bool,
        min_quality_score: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_patterns_file: Option<std::path::PathBuf>,
    },

    /// Llama Guard 3 for output validation
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

    /// GPT-OSS-Safeguard for output validation
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

    /// Composite output guardrail (combines multiple providers)
    Composite {
        providers: Vec<OutputGuardrailProviderConfig>,
        execution: ExecutionMode,
        aggregation: AggregationMode,
    },
}

impl Default for OutputGuardrailProviderConfig {
    fn default() -> Self {
        use crate::constants::{guardrails, output_limits};

        // Default: Regex with safety and hallucination checks
        Self::Regex {
            max_length_bytes: output_limits::MAX_OUTPUT_BYTES,
            check_safety: true,
            check_hallucination: true,
            check_format: true,
            min_quality_score: guardrails::DEFAULT_MIN_QUALITY_SCORE,
            custom_patterns_file: None,
        }
    }
}

impl OutputGuardrailProviderConfig {
    /// Convert Regex variant to legacy OutputGuardrailConfig
    pub fn to_output_config(&self) -> Option<crate::guardrails::output::OutputGuardrailConfig> {
        match self {
            Self::Regex {
                max_length_bytes,
                check_safety,
                check_hallucination,
                check_format,
                min_quality_score,
                custom_patterns_file,
            } => Some(crate::guardrails::output::OutputGuardrailConfig {
                max_length_bytes: *max_length_bytes,
                check_safety: *check_safety,
                check_hallucination: *check_hallucination,
                check_format: *check_format,
                min_quality_score: *min_quality_score,
                custom_patterns_file: custom_patterns_file.clone(),
            }),
            _ => None,
        }
    }
}

impl GuardrailProviderConfig {
    /// Convert to InputGuardrailConfig (if this is a Regex config)
    pub fn to_input_config(&self) -> Option<InputGuardrailConfig> {
        match self {
            Self::Regex {
                max_length_bytes,
                max_tokens_estimated,
                check_pii,
                check_content_filters,
            } => Some(InputGuardrailConfig {
                max_length_bytes: *max_length_bytes,
                max_tokens_estimated: *max_tokens_estimated,
                check_pii: *check_pii,
                check_content_filters: *check_content_filters,
                severity_threshold: crate::guardrails::provider::Severity::Critical,
                custom_patterns_file: None,
            }),
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

// Auto-migration from old InputGuardrailConfig
impl From<InputGuardrailConfig> for GuardrailConfig {
    fn from(old: InputGuardrailConfig) -> Self {
        Self {
            provider: GuardrailProviderConfig::Regex {
                max_length_bytes: old.max_length_bytes,
                max_tokens_estimated: old.max_tokens_estimated,
                check_pii: old.check_pii,
                check_content_filters: old.check_content_filters,
            },
        }
    }
}

/// Factory function to create GuardrailProvider from configuration
pub fn create_guardrail_provider(
    config: &GuardrailProviderConfig,
) -> Result<Box<dyn crate::guardrails::provider::GuardrailProvider>, crate::error::CliError> {
    use crate::guardrails::{
        gpt_oss_safeguard::GptOssSafeguardProvider, hybrid::HybridGuardrail, input::InputGuardrail,
        llama_guard::LlamaGuardProvider, provider::Severity,
    };

    match config {
        GuardrailProviderConfig::Regex {
            max_length_bytes,
            max_tokens_estimated,
            check_pii,
            check_content_filters,
        } => {
            let input_config = InputGuardrailConfig {
                max_length_bytes: *max_length_bytes,
                max_tokens_estimated: *max_tokens_estimated,
                check_pii: *check_pii,
                check_content_filters: *check_content_filters,
                severity_threshold: Severity::Critical,
                custom_patterns_file: None,
            };
            Ok(Box::new(InputGuardrail::new(input_config)))
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

/// Create output guardrail provider from configuration
pub fn create_output_guardrail_provider(
    config: &OutputGuardrailProviderConfig,
) -> Result<Box<dyn crate::guardrails::provider::GuardrailProvider>, crate::error::CliError> {
    use crate::guardrails::{
        gpt_oss_safeguard::GptOssSafeguardProvider, hybrid::HybridGuardrail,
        llama_guard::LlamaGuardProvider, output::OutputGuardrail,
    };

    match config {
        OutputGuardrailProviderConfig::Regex {
            max_length_bytes,
            check_safety,
            check_hallucination,
            check_format,
            min_quality_score,
            custom_patterns_file,
        } => {
            let output_config = crate::guardrails::output::OutputGuardrailConfig {
                max_length_bytes: *max_length_bytes,
                check_safety: *check_safety,
                check_hallucination: *check_hallucination,
                check_format: *check_format,
                min_quality_score: *min_quality_score,
                custom_patterns_file: custom_patterns_file.clone(),
            };
            Ok(Box::new(OutputGuardrail::new(output_config)))
        }

        OutputGuardrailProviderConfig::LlamaGuard {
            api_url,
            model,
            timeout_secs,
            enabled_categories,
            api_key,
            api_key_name,
        } => {
            let resolved_api_key = resolve_api_key(api_key, api_key_name, "LlamaGuard (output)")?;
            let llama_config = LlamaGuardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                enabled_categories: enabled_categories.clone(),
                timeout_secs: *timeout_secs,
                api_key: resolved_api_key,
            };
            Ok(Box::new(LlamaGuardProvider::new(llama_config)))
        }

        OutputGuardrailProviderConfig::GptOssSafeguard {
            api_url,
            model,
            policy,
            timeout_secs,
            api_key,
            api_key_name,
        } => {
            let resolved_api_key =
                resolve_api_key(api_key, api_key_name, "GptOssSafeguard (output)")?;
            let gpt_oss_config = GptOssSafeguardConfig {
                api_url: api_url.clone(),
                model: model.clone(),
                policy: policy.clone(),
                timeout_secs: *timeout_secs,
                api_key: resolved_api_key,
            };
            Ok(Box::new(GptOssSafeguardProvider::new(gpt_oss_config)))
        }

        OutputGuardrailProviderConfig::Composite {
            providers,
            execution,
            aggregation,
        } => {
            // Recursively create provider instances
            let provider_instances = providers
                .iter()
                .map(create_output_guardrail_provider)
                .collect::<Result<Vec<_>, crate::error::CliError>>()?;

            Ok(Box::new(HybridGuardrail::new(
                provider_instances,
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
            GuardrailProviderConfig::Composite {
                execution,
                aggregation,
                providers,
            } => {
                assert_eq!(execution, ExecutionMode::Parallel);
                assert_eq!(aggregation, AggregationMode::AllMustPass);
                assert_eq!(providers.len(), 2); // Regex + LlamaGuard
            }
            _ => panic!("Default should be Composite"),
        }
    }

    #[test]
    fn test_to_input_config() {
        let config = GuardrailProviderConfig::Regex {
            max_length_bytes: 1024,
            max_tokens_estimated: 500,
            check_pii: true,
            check_content_filters: false,
        };

        let input_config = config.to_input_config().unwrap();
        assert_eq!(input_config.max_length_bytes, 1024);
        assert_eq!(input_config.max_tokens_estimated, 500);
        assert!(input_config.check_pii);
        assert!(!input_config.check_content_filters);
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
        let config = GuardrailProviderConfig::Regex {
            max_length_bytes: 1024,
            max_tokens_estimated: 500,
            check_pii: true,
            check_content_filters: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"regex\""));
        assert!(json.contains("\"max_length_bytes\":1024"));

        let deserialized: GuardrailProviderConfig = serde_json::from_str(&json).unwrap();
        match deserialized {
            GuardrailProviderConfig::Regex {
                max_length_bytes, ..
            } => {
                assert_eq!(max_length_bytes, 1024);
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
                GuardrailProviderConfig::Regex {
                    max_length_bytes: 1024,
                    max_tokens_estimated: 500,
                    check_pii: true,
                    check_content_filters: true,
                },
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
