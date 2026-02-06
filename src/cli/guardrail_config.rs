use fortified_llm_client::{
    guardrails::config::RegexGuardrailConfig, ConfigFileRequest, GuardrailProviderConfig, Severity,
};

/// Configure input guardrails from CLI args or config file
///
/// Priority: CLI args > config file
///
/// # Arguments
///
/// * `enable_validation` - Whether CLI-based validation is enabled
/// * `max_input_length` - Optional max input length from CLI
/// * `file_config` - Optional config file data
///
/// # Returns
///
/// - `Some(GuardrailProviderConfig)` if guardrails are configured
/// - `None` if no guardrails configured
pub fn configure_guardrails(
    enable_validation: bool,
    max_input_length: Option<usize>,
    file_config: Option<&ConfigFileRequest>,
) -> Option<GuardrailProviderConfig> {
    if enable_validation {
        // CLI-based input validation
        log::debug!("Input validation enabled via CLI");
        Some(GuardrailProviderConfig::Regex(RegexGuardrailConfig {
            max_length_bytes: max_input_length
                .unwrap_or(fortified_llm_client::constants::input_limits::MAX_INPUT_BYTES),
            patterns_file: None,
            severity_threshold: Severity::Medium,
        }))
    } else if let Some(guardrail_cfg) = file_config.and_then(|c| c.guardrails.as_ref()) {
        // Config file-based guardrails (supports all provider types)
        log::debug!("Input validation enabled via config file");
        // Use input field, or fallback to flattened provider field
        guardrail_cfg
            .input
            .clone()
            .or_else(|| guardrail_cfg.provider.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_guardrails_cli_enabled() {
        let config = configure_guardrails(true, None, None);
        assert!(config.is_some());
        match config.unwrap() {
            GuardrailProviderConfig::Regex(regex_config) => {
                assert!(regex_config.patterns_file.is_none());
                assert_eq!(regex_config.severity_threshold, Severity::Medium);
            }
            _ => panic!("Expected Regex variant"),
        }
    }

    #[test]
    fn test_configure_guardrails_cli_with_custom_limits() {
        let config = configure_guardrails(true, Some(500_000), None);
        assert!(config.is_some());
        match config.unwrap() {
            GuardrailProviderConfig::Regex(regex_config) => {
                assert_eq!(regex_config.max_length_bytes, 500_000);
            }
            _ => panic!("Expected Regex variant"),
        }
    }

    #[test]
    fn test_configure_guardrails_disabled() {
        let config = configure_guardrails(false, None, None);
        assert!(config.is_none());
    }
}
