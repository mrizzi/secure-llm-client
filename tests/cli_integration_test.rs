// CLI binary integration tests
//
// These tests execute the actual binary and verify end-to-end behavior.
// This catches bugs in argument parsing, config loading, and error handling.

use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_cli_no_args_shows_help() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("fortified-llm-client"));
}

#[test]
fn test_cli_help_flag() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LLM API endpoint URL"))
        .stdout(predicate::str::contains("--api-url"));
}

#[test]
fn test_cli_version_flag() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_short_version_flag() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_missing_required_config() {
    // No config file and no CLI args = should fail
    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--model")
        .arg("test")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Should fail when required args missing"
    );

    // Error may be in stdout (JSON) or stderr
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("required")
            || combined.to_lowercase().contains("missing")
            || combined.to_lowercase().contains("must be provided"),
        "Error should mention required parameters"
    );
}

#[test]
fn test_cli_invalid_temperature() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--temperature")
        .arg("5.0") // Out of range (0.0-2.0)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Temperature"));
}

#[test]
fn test_cli_invalid_max_tokens() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--max-tokens")
        .arg("0") // Must be >= 1
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be"));
}

#[test]
fn test_cli_invalid_timeout() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--timeout")
        .arg("0") // Must be >= 1
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be"));
}

#[test]
fn test_cli_invalid_byte_size() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--max-input-length")
        .arg("invalid") // Not a valid size
        .assert()
        .failure();
}

#[test]
fn test_cli_nonexistent_config_file() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg("/nonexistent/path/config.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_cli_nonexistent_system_file() {
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--system-file")
        .arg("/nonexistent/prompt.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_cli_config_file_loads_all_params() {
    // Create a complete config file
    let config = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test-model",
        "system_prompt": "Test system.",
        "user_prompt": "Test user.",
        "temperature": 0.7,
        "max_tokens": 2000,
        "response_format": "json-object"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, config).unwrap();

    // Should fail on API connection, but that means config was loaded successfully
    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg(path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(!output.status.success(), "Should fail on API connection");

    // Error in stdout (JSON) or stderr
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("connection")
            || combined.to_lowercase().contains("not found")
            || combined.to_lowercase().contains("refused")
            || combined.to_lowercase().contains("api"),
        "Should fail with API-related error"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn test_cli_args_override_config_file() {
    // Config file with model=config-model
    let config = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "config-model",
        "system_prompt": "Test.",
        "user_prompt": "Test."
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, config).unwrap();

    // Override model with CLI arg
    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg(path.to_str().unwrap())
        .arg("--model")
        .arg("cli-model")
        .arg("--verbose")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should see cli-model in debug output, not config-model
    assert!(
        stderr.contains("cli-model"),
        "CLI arg should override config file model"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn test_cli_response_format_json_object() {
    let config = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test",
        "system_prompt": "Test.",
        "user_prompt": "Test.",
        "response_format": "json-object"
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, config).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg(path.to_str().unwrap())
        .arg("--verbose")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify response format was loaded from config
    assert!(
        stderr.contains("json-object"),
        "Response format from config file should be used"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn test_cli_response_format_schema_without_format_flag() {
    // Providing schema path without response_format should fail
    // First create a temp schema file so file validation passes
    let schema_file = NamedTempFile::new().unwrap();
    fs::write(&schema_file, r#"{"type": "object"}"#).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--response-format-schema")
        .arg(schema_file.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Should fail when schema provided without format flag"
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("can only be used with")
            || combined.contains("requires")
            || combined.contains("json-schema")
            || combined.contains("must be provided"),
        "Should explain that schema requires json-schema format, got: {combined}"
    );
}

#[test]
fn test_cli_system_file_and_system_text_conflict() {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, "test").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--system-file")
        .arg(file.path())
        .arg("--system-text")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("conflicts with").or(predicate::str::contains("cannot")));
}

#[test]
fn test_cli_user_file_and_user_text_conflict() {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, "test").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--user-file")
        .arg(file.path())
        .arg("--user-text")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("conflicts with").or(predicate::str::contains("cannot")));
}

#[test]
fn test_cli_output_file_creation() {
    let config = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test",
        "system_prompt": "Test.",
        "user_prompt": "Test."
    }"#;

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config).unwrap();

    let output_file = NamedTempFile::new().unwrap();
    let output_path = output_file.path().with_extension("json");

    // Should fail on API but create output file
    assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg(config_path.to_str().unwrap())
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .assert()
        .failure();

    // Output file should exist with error in JSON format
    assert!(output_path.exists(), "Output file should be created");
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("error"), "Output should contain error");
    assert!(content.starts_with("{"), "Output should be JSON");

    fs::remove_file(&config_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_cli_verbose_flag_enables_debug_logging() {
    let config = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test",
        "system_prompt": "Test.",
        "user_prompt": "Test."
    }"#;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("json");
    fs::write(&path, config).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("fortified-llm-client")
        .arg("--config-file")
        .arg(path.to_str().unwrap())
        .arg("--verbose")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should see DEBUG level logs
    assert!(
        stderr.contains("DEBUG") || stderr.contains("Evaluation Parameters"),
        "Verbose flag should enable DEBUG logging"
    );

    fs::remove_file(&path).ok();
}
