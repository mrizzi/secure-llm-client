// Integration test to verify ALL config file parameters are actually used
//
// This test ensures we don't have bugs where config file fields are defined
// but silently ignored during configuration building.
//
// IMPORTANT: When adding new fields to ConfigFileRequest, you MUST update this test!
// This test should include EVERY field defined in ConfigFileRequest struct.

use secure_llm_client::{config_builder::ConfigBuilder, load_config_file};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_all_config_file_parameters_are_used() {
    // Create a config file with EVERY possible parameter
    // NOTE: This list MUST be kept in sync with ConfigFileRequest struct!
    let json = r#"{
        "api_url": "http://test.example.com:8080/v1/chat/completions",
        "model": "test-model-123",
        "provider": "openai",
        "system_prompt": "System prompt from config",
        "user_prompt": "User prompt from config",
        "temperature": 0.7,
        "max_tokens": 8192,
        "timeout_secs": 600,
        "validate_tokens": true,
        "context_limit": 131072,
        "api_key": "test-api-key-12345",
        "response_format": "json-object",
        "response_format_schema": null,
        "response_format_schema_strict": null
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, json).unwrap();

    // Load config file
    let file_config = load_config_file(&path).unwrap();

    // Build configuration using ConfigBuilder (simulating main.rs flow)
    let builder = ConfigBuilder::new();
    let builder = builder.merge_file_config(&file_config);
    let config = builder.build().unwrap();

    // Verify EVERY field from config file is present in final config
    assert_eq!(
        config.api_url, "http://test.example.com:8080/v1/chat/completions",
        "api_url not applied from config file"
    );
    assert_eq!(
        config.model, "test-model-123",
        "model not applied from config file"
    );
    assert_eq!(
        config.system_prompt, "System prompt from config",
        "system_prompt not applied from config file"
    );
    assert_eq!(
        config.user_prompt, "User prompt from config",
        "user_prompt not applied from config file"
    );
    assert_eq!(
        config.temperature, 0.7,
        "temperature not applied from config file"
    );
    assert_eq!(
        config.max_tokens,
        Some(8192),
        "max_tokens not applied from config file"
    );
    assert_eq!(
        config.timeout_secs, 600,
        "timeout_secs not applied from config file"
    );
    assert!(
        config.validate_tokens,
        "validate_tokens not applied from config file"
    );
    assert_eq!(
        config.context_limit,
        Some(131072),
        "context_limit not applied from config file"
    );
    assert_eq!(
        config.api_key,
        Some("test-api-key-12345".to_string()),
        "api_key not applied from config file"
    );

    // Verify response_format is applied
    assert!(
        config.response_format.is_some(),
        "response_format not applied from config file"
    );
    match config.response_format {
        Some(secure_llm_client::ResponseFormat::JsonObject) => {} // Expected
        _ => panic!(
            "Expected JsonObject response format, got {:?}",
            config.response_format
        ),
    }

    // Verify provider is applied from config file
    assert_eq!(
        config.provider,
        Some(secure_llm_client::Provider::OpenAI),
        "provider not applied from config file"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn test_api_key_name_from_config_file() {
    // Test that api_key_name from config file loads API key from environment
    std::env::set_var("TEST_CONFIG_API_KEY", "secret-key-from-env");

    let json = r#"{
        "api_url": "http://test.example.com/api",
        "model": "test-model",
        "system_prompt": "System",
        "user_prompt": "User",
        "api_key_name": "TEST_CONFIG_API_KEY"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, json).unwrap();

    let file_config = load_config_file(&path).unwrap();

    // Verify api_key_name is loaded from config
    assert_eq!(
        file_config.api_key_name,
        Some("TEST_CONFIG_API_KEY".to_string()),
        "api_key_name not loaded from config file"
    );

    // Note: The actual environment variable reading happens in main.rs,
    // not in ConfigBuilder. This test just verifies the field is deserialized correctly.

    std::env::remove_var("TEST_CONFIG_API_KEY");
    fs::remove_file(&path).ok();
}

#[test]
fn test_pdf_file_from_config() {
    // Test that pdf_file from config file is loaded
    let json = r#"{
        "api_url": "http://test.example.com/api",
        "model": "test-model",
        "system_prompt": "System",
        "pdf_file": "tests/fixtures/simple.pdf"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, json).unwrap();

    let file_config = load_config_file(&path).unwrap();
    let builder = ConfigBuilder::new();
    let builder = builder.merge_file_config(&file_config);
    let config = builder.build().unwrap();

    // Verify pdf_file is loaded into pdf_input
    assert!(
        config.pdf_input.is_some(),
        "pdf_file not applied from config file"
    );
    assert_eq!(
        config.pdf_input.unwrap().to_str().unwrap(),
        "tests/fixtures/simple.pdf",
        "pdf_file path incorrect"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn test_all_config_file_parameters_with_json_schema() {
    // Test with json-schema response format
    // We'll use a minimal valid schema
    let schema_json = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "test": { "type": "string" }
        }
    }"#;

    // Create a properly named schema file (not using tempfile to control the name)
    let temp_dir = std::env::temp_dir();
    let schema_path = temp_dir.join("test_schema.json");
    fs::write(&schema_path, schema_json).unwrap();

    let config_json = format!(
        r#"{{
        "api_url": "http://test.example.com:8080/v1/chat/completions",
        "model": "test-model-456",
        "system_prompt": "System prompt",
        "user_prompt": "User prompt",
        "temperature": 0.0,
        "max_tokens": 4096,
        "timeout_secs": 300,
        "validate_tokens": false,
        "context_limit": 65536,
        "api_key": "test-key",
        "response_format": "json-schema",
        "response_format_schema": "{}",
        "response_format_schema_strict": true
    }}"#,
        schema_path.display()
    );

    let config_path = temp_dir.join("test_config_schema.json");
    fs::write(&config_path, config_json).unwrap();

    // Load and build config
    let file_config = load_config_file(&config_path).unwrap();
    let builder = ConfigBuilder::new();
    let builder = builder.merge_file_config(&file_config);
    let config = builder.build().unwrap();

    // Verify all fields
    assert_eq!(
        config.api_url,
        "http://test.example.com:8080/v1/chat/completions"
    );
    assert_eq!(config.model, "test-model-456");
    assert_eq!(config.system_prompt, "System prompt");
    assert_eq!(config.user_prompt, "User prompt");
    assert_eq!(config.temperature, 0.0);
    assert_eq!(config.max_tokens, Some(4096));
    assert_eq!(config.timeout_secs, 300);
    assert!(!config.validate_tokens);
    assert_eq!(config.context_limit, Some(65536));
    assert_eq!(config.api_key, Some("test-key".to_string()));

    // Verify response_format is json-schema variant
    assert!(config.response_format.is_some());
    match &config.response_format {
        Some(secure_llm_client::ResponseFormat::JsonSchema { json_schema }) => {
            assert_eq!(json_schema.name, "test_schema");
            assert_eq!(json_schema.strict, Some(true));
        }
        _ => panic!(
            "Expected JsonSchema response format, got {:?}",
            config.response_format
        ),
    }

    fs::remove_file(&config_path).ok();
    fs::remove_file(&schema_path).ok();
}

#[test]
fn test_toml_config_all_parameters() {
    // Test TOML format with all parameters
    let toml = r#"
        api_url = "http://toml.example.com:9000/api/generate"
        model = "toml-model"
        system_prompt = "TOML system"
        user_prompt = "TOML user"
        temperature = 0.9
        max_tokens = 16384
        timeout_secs = 900
        validate_tokens = true
        context_limit = 200000
        api_key = "toml-api-key"
        response_format = "text"
    "#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("toml");
    fs::write(&path, toml).unwrap();

    // Load and build
    let file_config = load_config_file(&path).unwrap();
    let builder = ConfigBuilder::new();
    let builder = builder.merge_file_config(&file_config);
    let config = builder.build().unwrap();

    // Verify all fields
    assert_eq!(config.api_url, "http://toml.example.com:9000/api/generate");
    assert_eq!(config.model, "toml-model");
    assert_eq!(config.system_prompt, "TOML system");
    assert_eq!(config.user_prompt, "TOML user");
    assert_eq!(config.temperature, 0.9);
    assert_eq!(config.max_tokens, Some(16384));
    assert_eq!(config.timeout_secs, 900);
    assert!(config.validate_tokens);
    assert_eq!(config.context_limit, Some(200000));
    assert_eq!(config.api_key, Some("toml-api-key".to_string()));

    // Verify response_format is Text variant
    assert!(config.response_format.is_some());
    match config.response_format {
        Some(secure_llm_client::ResponseFormat::Text) => {} // Expected
        _ => panic!(
            "Expected Text response format, got {:?}",
            config.response_format
        ),
    }

    fs::remove_file(&path).ok();
}

#[test]
fn test_cli_args_override_config_file() {
    // Verify that CLI args take precedence over config file
    let json = r#"{
        "api_url": "http://config-file.example.com/api",
        "model": "config-model",
        "system_prompt": "Config system",
        "user_prompt": "Config user",
        "temperature": 0.5,
        "max_tokens": 4000,
        "response_format": "text"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, json).unwrap();

    let file_config = load_config_file(&path).unwrap();

    // Simulate CLI args taking precedence
    let builder = ConfigBuilder::new();
    let builder = builder
        .api_url("http://cli-override.example.com/api")
        .model("cli-model")
        .temperature(0.8);

    // Merge config file (should NOT override CLI args)
    let builder = builder.merge_file_config(&file_config);
    let config = builder.build().unwrap();

    // CLI args should win
    assert_eq!(
        config.api_url, "http://cli-override.example.com/api",
        "CLI api_url should override config file"
    );
    assert_eq!(
        config.model, "cli-model",
        "CLI model should override config file"
    );
    assert_eq!(
        config.temperature, 0.8,
        "CLI temperature should override config file"
    );

    // Config file values should be used for fields not set via CLI
    assert_eq!(config.user_prompt, "Config user");
    assert_eq!(config.max_tokens, Some(4000));

    fs::remove_file(&path).ok();
}
