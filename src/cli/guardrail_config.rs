use fortified_llm_client::{ConfigFileRequest, GuardrailProviderConfig};

/// Configure input guardrails from CLI args or config file
///
/// Priority: CLI args > config file
///
/// # Arguments
///
/// * `enable_validation` - Whether CLI-based validation is enabled
/// * `max_input_length` - Optional max input length from CLI
/// * `max_input_tokens` - Optional max input tokens from CLI
/// * `file_config` - Optional config file data
///
/// # Returns
///
/// - `Some(GuardrailProviderConfig)` if guardrails are configured
/// - `None` if no guardrails configured
pub fn configure_guardrails(
    enable_validation: bool,
    max_input_length: Option<usize>,
    max_input_tokens: Option<usize>,
    file_config: Option<&ConfigFileRequest>,
) -> Option<GuardrailProviderConfig> {
    if enable_validation {
        // CLI-based input validation (regex pattern matching with default patterns)
        log::debug!("Input validation enabled via CLI (using default regex patterns)");
        Some(GuardrailProviderConfig::Regex {
            max_length_bytes: max_input_length
                .unwrap_or(fortified_llm_client::constants::input_limits::MAX_INPUT_BYTES),
            max_tokens_estimated: max_input_tokens
                .unwrap_or(fortified_llm_client::constants::input_limits::MAX_TOKENS_ESTIMATED),
            check_pii: true,
            check_content_filters: true,
        })
    } else if let Some(guardrail_cfg) = file_config.and_then(|c| c.guardrails.as_ref()) {
        // Config file-based guardrails (supports all provider types)
        log::debug!("Input validation enabled via config file");
        Some(guardrail_cfg.provider.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_guardrails_cli_enabled() {
        let config = configure_guardrails(true, None, None, None);
        assert!(config.is_some());
        match config.unwrap() {
            GuardrailProviderConfig::Regex {
                check_pii,
                check_content_filters,
                ..
            } => {
                assert!(check_pii);
                assert!(check_content_filters);
            }
            _ => panic!("Expected Regex variant"),
        }
    }

    #[test]
    fn test_configure_guardrails_cli_with_custom_limits() {
        let config = configure_guardrails(true, Some(500_000), Some(100_000), None);
        assert!(config.is_some());
        match config.unwrap() {
            GuardrailProviderConfig::Regex {
                max_length_bytes,
                max_tokens_estimated,
                ..
            } => {
                assert_eq!(max_length_bytes, 500_000);
                assert_eq!(max_tokens_estimated, 100_000);
            }
            _ => panic!("Expected Regex variant"),
        }
    }

    #[test]
    fn test_configure_guardrails_disabled() {
        let config = configure_guardrails(false, None, None, None);
        assert!(config.is_none());
    }
}
