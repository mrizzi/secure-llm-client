// Integration tests for config file override pattern
//
// These tests verify that CLI arguments correctly override config file values
// Priority: CLI arg > config file > hardcoded default

use fortified_llm_client::load_config_file;
use tempfile::NamedTempFile;

#[test]
fn test_config_file_temperature_default() {
    // Config file with no explicit temperature should use default (0.0)
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.temperature, 0.0); // Default temperature

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_temperature_explicit() {
    // Config file with explicit temperature should use that value
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello",
        "temperature": 0.7
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.temperature, 0.7);

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_max_tokens_default() {
    // Config file with no explicit max_tokens should use default (4000)
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.max_tokens, None); // Default (use model's maximum)

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_max_tokens_explicit() {
    // Config file with explicit max_tokens should use that value
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello",
        "max_tokens": 2000
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.max_tokens, Some(2000));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_timeout_default() {
    // Config file with no explicit timeout should use default (300)
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.timeout_secs, 300);

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_optional_user_prompt() {
    // Config file without user_prompt should still load successfully
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful."
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert!(config.user_prompt.is_none());

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_with_user_prompt() {
    // Config file with user_prompt should preserve it
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "What is 2+2?"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.user_prompt, Some("What is 2+2?".to_string()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_all_parameters() {
    // Config file with all parameters should preserve all values
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3:70b",
        "system_prompt": "You are an expert.",
        "user_prompt": "Analyze this.",
        "temperature": 0.5,
        "max_tokens": 8000,
        "timeout_secs": 600,
        "validate_tokens": true,
        "context_limit": 128000,
        "api_key": "test-key"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();

    // Verify all values
    assert_eq!(config.api_url, "http://localhost:11434/api/generate");
    assert_eq!(config.model, "llama3:70b");
    assert_eq!(config.system_prompt, Some("You are an expert.".to_string()));
    assert_eq!(config.user_prompt, Some("Analyze this.".to_string()));
    assert_eq!(config.temperature, 0.5);
    assert_eq!(config.max_tokens, Some(8000));
    assert_eq!(config.timeout_secs, 600);
    assert!(config.validate_tokens);
    assert_eq!(config.context_limit, Some(128000));
    assert_eq!(config.api_key, Some("test-key".to_string()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_toml_config_format() {
    // TOML config should work identically to JSON
    let toml = r#"
        api_url = "http://localhost:11434/api/generate"
        model = "llama3"
        system_prompt = "You are helpful."
        user_prompt = "Hello"
        temperature = 0.7
        max_tokens = 2000
    "#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("toml");
    std::fs::write(&path, toml).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.max_tokens, Some(2000));
    assert_eq!(config.user_prompt, Some("Hello".to_string()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_response_format_text() {
    // Config file with response_format="text"
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello",
        "response_format": "text"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.response_format, Some("text".to_string()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_response_format_json_object() {
    // Config file with response_format="json-object"
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello",
        "response_format": "json-object"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.response_format, Some("json-object".to_string()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_config_file_response_format_json_schema() {
    // Config file with response_format="json-schema"
    let json = r#"{
        "api_url": "http://localhost:11434/api/generate",
        "model": "llama3",
        "system_prompt": "You are helpful.",
        "user_prompt": "Hello",
        "response_format": "json-schema",
        "response_format_schema": "path/to/schema.json",
        "response_format_schema_strict": true
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    std::fs::write(&path, json).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.response_format, Some("json-schema".to_string()));
    assert_eq!(
        config.response_format_schema,
        Some("path/to/schema.json".to_string())
    );
    assert_eq!(config.response_format_schema_strict, Some(true));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_toml_config_response_format() {
    // TOML config with response_format
    let toml = r#"
        api_url = "http://localhost:11434/api/generate"
        model = "llama3"
        system_prompt = "You are helpful."
        user_prompt = "Hello"
        response_format = "json-object"
    "#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("toml");
    std::fs::write(&path, toml).unwrap();

    let config = load_config_file(&path).unwrap();
    assert_eq!(config.response_format, Some("json-object".to_string()));

    std::fs::remove_file(&path).ok();
}
