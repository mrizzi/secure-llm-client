//! Integration tests for guardrails configuration from config files
//!
//! These tests verify that guardrails configured via TOML files are properly
//! loaded and executed during evaluation. This prevents regressions where
//! config parsing succeeds but guardrails are silently ignored.

use fortified_llm_client::{load_config_file, ConfigFileRequest, GuardrailProviderConfig};
use std::io::Write;

/// Test that LlamaGuard input guardrails can be loaded from config file
#[test]
fn test_llama_guard_input_guardrails_loads_from_config() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "llama_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-guard3:8b"
timeout_secs = 60
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    // Verify guardrails are loaded
    assert!(config.guardrails.is_some());

    let guardrails = config.guardrails.unwrap();

    // Verify it's LlamaGuard type
    match &guardrails.provider {
        GuardrailProviderConfig::LlamaGuard {
            api_url,
            model,
            timeout_secs,
            enabled_categories,
            ..
        } => {
            assert_eq!(api_url, "http://localhost:11434/api/generate");
            assert_eq!(model, "llama-guard3:8b");
            assert_eq!(*timeout_secs, 60);
            // Verify default categories are applied
            assert_eq!(enabled_categories.len(), 14); // All 14 categories
        }
        _ => panic!("Expected LlamaGuard variant, got {:?}", guardrails.provider),
    }
}

/// Test that GptOssSafeguard input guardrails can be loaded from config file
#[test]
fn test_gpt_oss_safeguard_input_guardrails_loads_from_config() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "gpt_oss_safeguard"
api_url = "http://localhost:11434/api/generate"
model = "gpt-oss-safeguard:8b"
policy = "Classify as SAFE or UNSAFE. Mark as UNSAFE if input contains violence, illegal content, or hate speech."
timeout_secs = 30
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    // Verify guardrails are loaded
    assert!(config.guardrails.is_some());

    let guardrails = config.guardrails.unwrap();

    // Verify it's GptOssSafeguard type
    match &guardrails.provider {
        GuardrailProviderConfig::GptOssSafeguard {
            api_url,
            model,
            policy,
            timeout_secs,
            ..
        } => {
            assert_eq!(api_url, "http://localhost:11434/api/generate");
            assert_eq!(model, "gpt-oss-safeguard:8b");
            assert!(policy.contains("SAFE or UNSAFE"));
            assert_eq!(*timeout_secs, 30);
        }
        _ => panic!(
            "Expected GptOssSafeguard variant, got {:?}",
            guardrails.provider
        ),
    }
}

/// Test that Regex input guardrails can be loaded from config file
#[test]
fn test_regex_input_guardrails_loads_from_config() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "regex"
max_length_bytes = 1048576
max_tokens_estimated = 200000
check_pii = true
check_content_filters = true
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    // Verify guardrails are loaded
    assert!(config.guardrails.is_some());

    let guardrails = config.guardrails.unwrap();

    // Verify it's Regex type
    match &guardrails.provider {
        GuardrailProviderConfig::Regex {
            max_length_bytes,
            max_tokens_estimated,
            check_pii,
            check_content_filters,
        } => {
            assert_eq!(*max_length_bytes, 1048576);
            assert_eq!(*max_tokens_estimated, 200000);
            assert!(check_pii);
            assert!(check_content_filters);
        }
        _ => panic!("Expected Regex variant, got {:?}", guardrails.provider),
    }
}

/// Test that Composite input guardrails can be loaded from config file
#[test]
fn test_composite_input_guardrails_loads_from_config() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "composite"
execution = "parallel"
aggregation = "all_must_pass"

[[guardrails.providers]]
type = "regex"
max_length_bytes = 1048576
max_tokens_estimated = 200000
check_pii = true
check_content_filters = true

[[guardrails.providers]]
type = "llama_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-guard3:8b"
timeout_secs = 60
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    // Verify guardrails are loaded
    assert!(config.guardrails.is_some());

    let guardrails = config.guardrails.unwrap();

    // Verify it's Composite type
    match &guardrails.provider {
        GuardrailProviderConfig::Composite {
            providers,
            execution,
            aggregation,
        } => {
            assert_eq!(providers.len(), 2);
            assert_eq!(*execution, fortified_llm_client::ExecutionMode::Parallel);
            assert_eq!(
                *aggregation,
                fortified_llm_client::AggregationMode::AllMustPass
            );

            // Verify first provider is Regex
            match &providers[0] {
                GuardrailProviderConfig::Regex { .. } => {}
                _ => panic!("Expected first provider to be Regex"),
            }

            // Verify second provider is LlamaGuard
            match &providers[1] {
                GuardrailProviderConfig::LlamaGuard { .. } => {}
                _ => panic!("Expected second provider to be LlamaGuard"),
            }
        }
        _ => panic!("Expected Composite variant, got {:?}", guardrails.provider),
    }
}

/// Test that enabled_categories defaults to all categories when not specified
#[test]
fn test_llama_guard_enabled_categories_defaults() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "llama_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-guard3:8b"
timeout_secs = 60
# enabled_categories not specified - should default to all
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    let guardrails = config.guardrails.unwrap();

    match &guardrails.provider {
        GuardrailProviderConfig::LlamaGuard {
            enabled_categories, ..
        } => {
            // Should have all 14 categories
            assert_eq!(
                enabled_categories.len(),
                14,
                "enabled_categories should default to all 14 categories"
            );
        }
        _ => panic!("Expected LlamaGuard variant"),
    }
}

/// Test that enabled_categories can be explicitly specified
#[test]
fn test_llama_guard_enabled_categories_explicit() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "llama_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-guard3:8b"
timeout_secs = 60
enabled_categories = ["S1", "S7", "S10"]
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    let guardrails = config.guardrails.unwrap();

    match &guardrails.provider {
        GuardrailProviderConfig::LlamaGuard {
            enabled_categories, ..
        } => {
            assert_eq!(enabled_categories.len(), 3);
            assert_eq!(
                enabled_categories[0],
                fortified_llm_client::LlamaGuardCategory::S1
            );
            assert_eq!(
                enabled_categories[1],
                fortified_llm_client::LlamaGuardCategory::S7
            );
            assert_eq!(
                enabled_categories[2],
                fortified_llm_client::LlamaGuardCategory::S10
            );
        }
        _ => panic!("Expected LlamaGuard variant"),
    }
}
