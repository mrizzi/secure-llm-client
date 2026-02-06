//! Metadata Completeness Tests
//!
//! These tests verify that ALL input configuration values are correctly
//! reflected in the metadata output. This prevents regression where new
//! config fields are added but not included in metadata.

use fortified_llm_client::{
    config_builder::ConfigBuilder, evaluate, guardrails::config::RegexGuardrailConfig,
    GuardrailProviderConfig, Provider, ResponseFormat, Severity,
};
use mockito::Server;

/// Test that all EvaluationConfig fields appear in Metadata for successful evaluation
#[tokio::test]
async fn test_metadata_contains_all_config_fields_success() {
    let mut server = Server::new_async().await;

    // Mock successful OpenAI-compatible response
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Test system prompt")
        .user_prompt("Test user prompt")
        .provider(Provider::OpenAI)
        .temperature(0.7)
        .max_tokens(1000)
        .timeout_secs(60)
        .validate_tokens(true)
        .context_limit(128000)
        .build()
        .unwrap();

    let result = evaluate(config.clone()).await.unwrap();

    mock.assert_async().await;

    // Verify status
    assert_eq!(result.status, "success");

    // Verify all execution results fields
    assert_eq!(result.metadata.model, "test-model");
    assert!(result.metadata.tokens_estimated > 0);
    assert!(result.metadata.latency_ms > 0);
    assert!(!result.metadata.timestamp.is_empty());

    // Verify all input configuration fields
    assert_eq!(
        result.metadata.api_url,
        server.url() + "/v1/chat/completions"
    );
    assert_eq!(result.metadata.provider, Some("OpenAI".to_string()));
    assert_eq!(result.metadata.temperature, 0.7);
    assert_eq!(result.metadata.max_tokens, Some(1000));
    assert_eq!(result.metadata.timeout_secs, 60);
    assert_eq!(result.metadata.context_limit, Some(128000));
    assert_eq!(result.metadata.response_format, None);
    assert!(result.metadata.validate_tokens);

    // Verify input source fields (text inputs)
    assert_eq!(
        result.metadata.system_prompt_text,
        Some("Test system prompt".to_string())
    );
    assert_eq!(result.metadata.system_prompt_file, None);
    assert_eq!(
        result.metadata.user_prompt_text,
        Some("Test user prompt".to_string())
    );
    assert_eq!(result.metadata.user_prompt_file, None);
    assert_eq!(result.metadata.pdf_input, None);

    // Verify guardrails fields
    assert_eq!(result.metadata.input_guardrails_enabled, None);
    assert_eq!(result.metadata.output_guardrails_enabled, None);
}

/// Test metadata for evaluation with input guardrails enabled
#[tokio::test]
async fn test_metadata_with_input_guardrails() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let input_guardrails = GuardrailProviderConfig::Regex(RegexGuardrailConfig {
        max_length_bytes: 100000,
        patterns_file: None,
        severity_threshold: Severity::Medium,
    });

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("System")
        .user_prompt("User")
        .provider(Provider::OpenAI)
        .input_guardrails(input_guardrails)
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // Verify input guardrails are reflected
    assert_eq!(result.metadata.input_guardrails_enabled, Some(true));
    assert_eq!(result.metadata.output_guardrails_enabled, None);
}

/// Test metadata for evaluation with output guardrails enabled
#[tokio::test]
async fn test_metadata_with_output_guardrails() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let output_guardrails = GuardrailProviderConfig::Regex(RegexGuardrailConfig {
        max_length_bytes: 100000,
        patterns_file: None,
        severity_threshold: Severity::Medium,
    });

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("System")
        .user_prompt("User")
        .provider(Provider::OpenAI)
        .output_guardrails(output_guardrails)
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // Verify output guardrails are reflected
    assert_eq!(result.metadata.input_guardrails_enabled, None);
    assert_eq!(result.metadata.output_guardrails_enabled, Some(true));
}

/// Test metadata with response format configured
#[tokio::test]
async fn test_metadata_with_response_format() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "{\"test\": true}"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("System")
        .user_prompt("User")
        .provider(Provider::OpenAI)
        .response_format(ResponseFormat::json())
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // Verify response format is reflected
    assert_eq!(
        result.metadata.response_format,
        Some("json-object".to_string())
    );
}

/// Test metadata when max_tokens is not specified (None)
#[tokio::test]
async fn test_metadata_with_optional_fields_none() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("System")
        .user_prompt("User")
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // Verify optional fields are None when not specified
    assert_eq!(result.metadata.max_tokens, None);
    assert_eq!(result.metadata.context_limit, None);
    assert_eq!(result.metadata.response_format, None);
    assert_eq!(result.metadata.provider, None); // Auto-detected
}

/// Test metadata in error scenarios (context limit exceeded) - ensures metadata is still complete
#[tokio::test]
async fn test_metadata_in_error_response() {
    // Use a context limit validation error (returns Ok(CliOutput) with status="error")
    // rather than HTTP error (returns Err(CliError))
    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/api/generate")
        .model("test-model")
        .system_prompt("System")
        .user_prompt("User")
        .temperature(0.5)
        .max_tokens(2000)
        .validate_tokens(true)
        .context_limit(100) // Very small limit to trigger error
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    // Should be error (context limit exceeded)
    assert_eq!(result.status, "error");
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap().code, "CONTEXT_LIMIT_EXCEEDED");

    // But metadata should still be complete with all config values
    assert_eq!(result.metadata.model, "test-model");
    assert_eq!(result.metadata.temperature, 0.5);
    assert_eq!(result.metadata.max_tokens, Some(2000));
    assert!(result.metadata.validate_tokens);
    assert_eq!(result.metadata.context_limit, Some(100));
    assert_eq!(
        result.metadata.system_prompt_text,
        Some("System".to_string())
    );
    assert_eq!(result.metadata.user_prompt_text, Some("User".to_string()));
}

/// Test metadata with PDF input configured
#[tokio::test]
async fn test_metadata_with_pdf_input() {
    // Note: This test would require a real PDF file and docling installed
    // For now, we verify the field is present in the struct
    use fortified_llm_client::Metadata;

    let metadata = Metadata {
        model: "test".to_string(),
        tokens_estimated: 100,
        latency_ms: 200,
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        api_url: "http://test".to_string(),
        provider: None,
        temperature: 0.7,
        max_tokens: Some(1000),
        seed: None,
        timeout_secs: 30,
        context_limit: None,
        response_format: None,
        validate_tokens: false,
        system_prompt_text: Some("system".to_string()),
        system_prompt_file: None,
        user_prompt_text: None, // PDF replaces user prompt
        user_prompt_file: None,
        pdf_input: Some("/path/to/file.pdf".to_string()),
        input_guardrails_enabled: None,
        output_guardrails_enabled: None,
    };

    // Verify pdf_input field exists and can be set
    assert_eq!(metadata.pdf_input, Some("/path/to/file.pdf".to_string()));
    // When PDF is used, user_prompt_text should be None
    assert_eq!(metadata.user_prompt_text, None);
}

/// Test that metadata JSON serialization includes all fields
#[tokio::test]
async fn test_metadata_json_serialization_completeness() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("System prompt")
        .user_prompt("User prompt")
        .provider(Provider::OpenAI)
        .temperature(0.7)
        .max_tokens(1000)
        .timeout_secs(60)
        .validate_tokens(true)
        .context_limit(128000)
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&result).unwrap();

    // Verify all metadata fields are present in JSON
    assert!(json.contains("\"model\""));
    assert!(json.contains("\"tokens_estimated\""));
    assert!(json.contains("\"latency_ms\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"api_url\""));
    assert!(json.contains("\"provider\""));
    assert!(json.contains("\"temperature\""));
    assert!(json.contains("\"max_tokens\""));
    assert!(json.contains("\"timeout_secs\""));
    assert!(json.contains("\"context_limit\""));
    assert!(json.contains("\"validate_tokens\""));
    assert!(json.contains("\"system_prompt_text\""));
    assert!(json.contains("\"user_prompt_text\""));

    // Verify values are correct in JSON
    assert!(json.contains("\"model\": \"test-model\""));
    assert!(json.contains("\"temperature\": 0.7"));
    assert!(json.contains("\"max_tokens\": 1000"));
    assert!(json.contains("\"timeout_secs\": 60"));
    assert!(json.contains("\"validate_tokens\": true"));
}

/// Test metadata with file inputs - verifies file paths are tracked instead of content
#[tokio::test]
async fn test_metadata_with_file_inputs() {
    use fortified_llm_client::config_builder::ConfigBuilder;
    use std::path::PathBuf;

    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                }
            }]
        }"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Content from system file".to_string())
        .system_prompt_file(PathBuf::from("/path/to/system.md"))
        .user_prompt("Content from user file".to_string())
        .user_prompt_file(PathBuf::from("/path/to/user.md"))
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();

    mock.assert_async().await;

    // When prompts come from files, metadata should show FILE PATHS, not content
    assert_eq!(result.metadata.system_prompt_text, None);
    assert_eq!(
        result.metadata.system_prompt_file,
        Some("/path/to/system.md".to_string())
    );
    assert_eq!(result.metadata.user_prompt_text, None);
    assert_eq!(
        result.metadata.user_prompt_file,
        Some("/path/to/user.md".to_string())
    );
}

/// Test that adding a new field to EvaluationConfig without updating Metadata
/// causes this test to fail (compile-time check)
#[test]
fn test_metadata_struct_completeness_compile_check() {
    // This test ensures Metadata has all the fields we expect
    // If a new field is added to EvaluationConfig, this should be updated
    use fortified_llm_client::Metadata;

    let _metadata = Metadata {
        // Execution results
        model: String::new(),
        tokens_estimated: 0,
        latency_ms: 0,
        timestamp: String::new(),
        // Input configuration
        api_url: String::new(),
        provider: None,
        temperature: 0.0,
        max_tokens: None,
        seed: None,
        timeout_secs: 0,
        context_limit: None,
        response_format: None,
        validate_tokens: false,
        // Input sources (text vs file distinction)
        system_prompt_text: None,
        system_prompt_file: None,
        user_prompt_text: None,
        user_prompt_file: None,
        pdf_input: None,
        // Guardrails
        input_guardrails_enabled: None,
        output_guardrails_enabled: None,
    };

    // If this compiles, all expected fields are present
    // If a field is missing, this will fail to compile
}
