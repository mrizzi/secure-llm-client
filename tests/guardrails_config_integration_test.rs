//! Integration tests for guardrails configuration from config files
//!
//! These tests verify that guardrails configured via TOML files are properly
//! loaded and executed during evaluation. This prevents regressions where
//! config parsing succeeds but guardrails are silently ignored.

use fortified_llm_client::{
    config_builder::ConfigBuilder, load_config_file, ConfigFileRequest, GuardrailProviderConfig,
};
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

    // Verify it's LlamaGuard type (using flattened provider field)
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
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
        _ => panic!("Expected LlamaGuard variant, got {:?}", provider_config),
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
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
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
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
        GuardrailProviderConfig::Regex(regex_config) => {
            assert_eq!(regex_config.max_length_bytes, 1048576);
        }
        _ => panic!("Expected Regex variant, got {:?}", provider_config),
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
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
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
        _ => panic!("Expected Composite variant, got {:?}", provider_config),
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

    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
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

    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
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

/// Test that flattened provider field applies to BOTH input and output guardrails
#[test]
fn test_flattened_provider_applies_to_both_input_and_output() {
    let config_content = r#"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
system_prompt = "test system"
user_prompt = "test user"

[guardrails]
type = "regex"
max_length_bytes = 1000
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    assert!(config.guardrails.is_some());
    let guardrails = config.guardrails.unwrap();

    // The flattened provider field should be present
    assert!(guardrails.provider.is_some());

    // Neither explicit input nor output should be set
    assert!(guardrails.input.is_none());
    assert!(guardrails.output.is_none());

    // Verify the provider is regex with correct config
    match guardrails.provider.as_ref().unwrap() {
        GuardrailProviderConfig::Regex(config) => {
            assert_eq!(config.max_length_bytes, 1000);
        }
        _ => panic!("Expected Regex variant"),
    }
}

/// Test that explicit output field overrides flattened provider
#[test]
fn test_explicit_output_overrides_flattened_provider() {
    let config_content = r#"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
system_prompt = "test system"
user_prompt = "test user"

[guardrails]
type = "regex"
max_length_bytes = 1000

[guardrails.output]
type = "regex"
max_length_bytes = 2000
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    assert!(config.guardrails.is_some());
    let guardrails = config.guardrails.unwrap();

    // Flattened provider should be present
    assert!(guardrails.provider.is_some());

    // Explicit output should be present
    assert!(guardrails.output.is_some());

    // Input should be None (uses flattened)
    assert!(guardrails.input.is_none());

    // Verify flattened provider has 1000
    match guardrails.provider.as_ref().unwrap() {
        GuardrailProviderConfig::Regex(config) => {
            assert_eq!(config.max_length_bytes, 1000);
        }
        _ => panic!("Expected Regex variant for flattened provider"),
    }

    // Verify explicit output has 2000 (override)
    match guardrails.output.as_ref().unwrap() {
        GuardrailProviderConfig::Regex(config) => {
            assert_eq!(config.max_length_bytes, 2000);
        }
        _ => panic!("Expected Regex variant for output"),
    }
}

/// Test that ConfigBuilder correctly applies flattened provider to output guardrails
#[test]
fn test_config_builder_applies_flattened_to_output() {
    let config_content = r#"
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
system_prompt = "test system"
user_prompt = "test user"

[guardrails]
type = "regex"
max_length_bytes = 1000
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let file_config: ConfigFileRequest =
        load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    let builder = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .user_prompt("test")
        .system_prompt("system")
        .merge_file_config(&file_config);

    let eval_config = builder.build().unwrap();

    // Both input and output should have guardrails from flattened provider
    assert!(eval_config.input_guardrails.is_some());
    assert!(eval_config.output_guardrails.is_some());

    // Verify both are Regex with 1000 bytes
    match eval_config.input_guardrails.as_ref().unwrap() {
        GuardrailProviderConfig::Regex(config) => {
            assert_eq!(config.max_length_bytes, 1000);
        }
        _ => panic!("Expected Regex variant for input"),
    }

    match eval_config.output_guardrails.as_ref().unwrap() {
        GuardrailProviderConfig::Regex(config) => {
            assert_eq!(config.max_length_bytes, 1000);
        }
        _ => panic!("Expected Regex variant for output"),
    }
}
