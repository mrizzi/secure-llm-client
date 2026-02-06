//! Integration tests for Llama Prompt Guard 2 configuration and provider creation
//!
//! These tests verify that Llama Prompt Guard can be configured via TOML files
//! and that the provider is properly created with correct settings.

use fortified_llm_client::{
    create_guardrail_provider, load_config_file, AggregationMode, ConfigFileRequest, ExecutionMode,
    GuardrailProvider, GuardrailProviderConfig, LlamaPromptGuardConfig, LlamaPromptGuardProvider,
};
use std::io::Write;

/// Test that LlamaPromptGuard can be loaded from config file
#[test]
fn test_llama_prompt_guard_loads_from_config() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "llama_prompt_guard"
api_url = "https://api.groq.com/openai/v1/chat/completions"
model = "meta-llama/llama-prompt-guard-2-22m"
timeout_secs = 10
threshold = 0.5
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    // Verify guardrails are loaded
    assert!(config.guardrails.is_some());

    let guardrails = config.guardrails.unwrap();

    // Verify it's LlamaPromptGuard type
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
        GuardrailProviderConfig::LlamaPromptGuard {
            api_url,
            model,
            timeout_secs,
            threshold,
            ..
        } => {
            assert_eq!(api_url, "https://api.groq.com/openai/v1/chat/completions");
            assert_eq!(model, "meta-llama/llama-prompt-guard-2-22m");
            assert_eq!(*timeout_secs, 10);
            assert_eq!(*threshold, 0.5);
        }
        _ => panic!("Expected LlamaPromptGuard variant"),
    }
}

/// Test that LlamaPromptGuard threshold defaults to 0.5
#[test]
fn test_llama_prompt_guard_config_default_threshold() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "llama_prompt_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-prompt-guard-2-22m"
timeout_secs = 10
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
        GuardrailProviderConfig::LlamaPromptGuard { threshold, .. } => {
            assert_eq!(*threshold, 0.5); // Default threshold
        }
        _ => panic!("Expected LlamaPromptGuard variant"),
    }
}

/// Test that LlamaPromptGuard can be created as a provider
#[test]
fn test_llama_prompt_guard_provider_creation() {
    let config = GuardrailProviderConfig::LlamaPromptGuard {
        api_url: "http://localhost:11434/api/generate".to_string(),
        model: "llama-prompt-guard-2-22m".to_string(),
        timeout_secs: 10,
        threshold: 0.5,
        api_key: None,
        api_key_name: None,
    };

    let provider = create_guardrail_provider(&config);
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "LlamaPromptGuard2");
}

/// Test that LlamaPromptGuard can be used in composite guardrails
#[test]
fn test_llama_prompt_guard_in_composite() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"
user_prompt = "Test user"

[guardrails]
type = "composite"
execution = "sequential"
aggregation = "all_must_pass"

[[guardrails.providers]]
type = "llama_prompt_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-prompt-guard-2-22m"
timeout_secs = 10
threshold = 0.5

[[guardrails.providers]]
type = "regex"
max_length_bytes = 50000
max_tokens_estimated = 10000
check_pii = true
check_content_filters = true
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
        GuardrailProviderConfig::Composite {
            providers,
            execution,
            aggregation,
        } => {
            assert_eq!(providers.len(), 2);
            assert_eq!(*execution, ExecutionMode::Sequential);
            assert_eq!(*aggregation, AggregationMode::AllMustPass);

            // Verify first provider is LlamaPromptGuard
            match &providers[0] {
                GuardrailProviderConfig::LlamaPromptGuard { .. } => {}
                _ => panic!("Expected first provider to be LlamaPromptGuard"),
            }

            // Verify second provider is Regex
            match &providers[1] {
                GuardrailProviderConfig::Regex { .. } => {}
                _ => panic!("Expected second provider to be Regex"),
            }
        }
        _ => panic!("Expected Composite variant"),
    }
}

/// Test LlamaPromptGuardConfig struct creation
#[test]
fn test_llama_prompt_guard_config_struct() {
    let config = LlamaPromptGuardConfig {
        api_url: "https://api.groq.com/openai/v1/chat/completions".to_string(),
        model: "meta-llama/llama-prompt-guard-2-22m".to_string(),
        timeout_secs: 5,
        threshold: 0.3,
        api_key: None,
        api_key_name: None,
    };

    assert_eq!(
        config.api_url,
        "https://api.groq.com/openai/v1/chat/completions"
    );
    assert_eq!(config.model, "meta-llama/llama-prompt-guard-2-22m");
    assert_eq!(config.timeout_secs, 5);
    assert_eq!(config.threshold, 0.3);
}

/// Test LlamaPromptGuardProvider creation
#[test]
fn test_llama_prompt_guard_provider_struct() {
    let config = LlamaPromptGuardConfig::default();
    let provider = LlamaPromptGuardProvider::new(config);

    assert_eq!(provider.name(), "LlamaPromptGuard2");
}

/// Test composite provider with Prompt Guard and LlamaGuard
#[test]
fn test_composite_prompt_guard_and_llama_guard() {
    let config_content = r#"
api_url = "http://localhost:11434/api/generate"
model = "test-model"
system_prompt = "Test system"

[guardrails]
type = "composite"
execution = "sequential"
aggregation = "all_must_pass"

[[guardrails.providers]]
type = "llama_prompt_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-prompt-guard-2-22m"
timeout_secs = 10
threshold = 0.5

[[guardrails.providers]]
type = "llama_guard"
api_url = "http://localhost:11434/api/generate"
model = "llama-guard3:8b"
timeout_secs = 30
"#;

    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config: ConfigFileRequest = load_config_file(temp_file.path().to_str().unwrap()).unwrap();

    let guardrails = config.guardrails.unwrap();

    // Verify it's a composite with 2 providers
    let provider_config = guardrails
        .input
        .as_ref()
        .or(guardrails.provider.as_ref())
        .expect("Should have guardrail config");
    match provider_config {
        GuardrailProviderConfig::Composite { providers, .. } => {
            assert_eq!(providers.len(), 2);

            // Verify first is LlamaPromptGuard
            match &providers[0] {
                GuardrailProviderConfig::LlamaPromptGuard { model, .. } => {
                    assert_eq!(model, "llama-prompt-guard-2-22m");
                }
                _ => panic!("Expected LlamaPromptGuard"),
            }

            // Verify second is LlamaGuard
            match &providers[1] {
                GuardrailProviderConfig::LlamaGuard { model, .. } => {
                    assert_eq!(model, "llama-guard3:8b");
                }
                _ => panic!("Expected LlamaGuard"),
            }
        }
        _ => panic!("Expected Composite variant"),
    }

    // Verify provider can be created
    let provider = create_guardrail_provider(guardrails.provider.as_ref().unwrap());
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "CompositeGuardrail");
}
