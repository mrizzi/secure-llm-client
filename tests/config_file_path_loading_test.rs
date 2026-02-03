// Critical integration tests for config file path loading
//
// These tests verify that config files can specify FILE PATHS for prompts
// and that those files are actually LOADED (not sent as literal paths to the API)
//
// REGRESSION: This bug existed where "system_prompt = 'path/to/file.md'" in config
// would send the literal string "path/to/file.md" to the API instead of file content

use fortified_llm_client::config::load_config_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_config_loads_system_prompt_from_file() {
    // Create a system prompt file
    let system_file = NamedTempFile::new().unwrap();
    let system_path = system_file.path().with_extension("md");
    let system_content = "You are a security expert specialized in architecture reviews.";
    fs::write(&system_path, system_content).unwrap();

    // Create a config that references the file
    let config_json = format!(
        r#"{{
            "api_url": "http://localhost:11434/v1/chat/completions",
            "model": "test-model",
            "system_prompt_file": "{}",
            "user_prompt": "Test user prompt"
        }}"#,
        system_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Load config
    let config = load_config_file(&config_path).unwrap();

    // CRITICAL: Verify the config contains FILE CONTENT (loaded from file)
    assert_eq!(
        config.system_prompt,
        Some(system_content.to_string()),
        "Config should contain file CONTENT"
    );

    // Verify the file path is PRESERVED for metadata tracking
    assert_eq!(
        config.system_prompt_file,
        Some(system_path.display().to_string()),
        "File path should be preserved for metadata tracking"
    );

    // Cleanup
    fs::remove_file(&system_path).ok();
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_loads_user_prompt_from_file() {
    // Create a user prompt file
    let user_file = NamedTempFile::new().unwrap();
    let user_path = user_file.path().with_extension("md");
    let user_content = "# Technical Document\n\nThis is a sample document for analysis.";
    fs::write(&user_path, user_content).unwrap();

    // Create a config that references the file
    let config_json = format!(
        r#"{{
            "api_url": "http://localhost:11434/v1/chat/completions",
            "model": "test-model",
            "system_prompt": "You are helpful",
            "user_prompt_file": "{}"
        }}"#,
        user_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Load config
    let config = load_config_file(&config_path).unwrap();

    // CRITICAL: Verify the config contains FILE CONTENT (loaded from file)
    assert_eq!(
        config.user_prompt,
        Some(user_content.to_string()),
        "Config should contain file CONTENT"
    );

    // Verify the file path is PRESERVED for metadata tracking
    assert_eq!(
        config.user_prompt_file,
        Some(user_path.display().to_string()),
        "File path should be preserved for metadata tracking"
    );

    // Cleanup
    fs::remove_file(&user_path).ok();
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_loads_both_prompts_from_files() {
    // Create both prompt files
    let system_file = NamedTempFile::new().unwrap();
    let system_path = system_file.path().with_extension("md");
    let system_content = "System prompt content from file";
    fs::write(&system_path, system_content).unwrap();

    let user_file = NamedTempFile::new().unwrap();
    let user_path = user_file.path().with_extension("md");
    let user_content = "User prompt content from file";
    fs::write(&user_path, user_content).unwrap();

    // Create a config that references both files
    let config_toml = format!(
        r#"
api_url = "http://localhost:11434/v1/chat/completions"
model = "test-model"
system_prompt_file = "{}"
user_prompt_file = "{}"
temperature = 0.0
"#,
        system_path.to_str().unwrap(),
        user_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("toml");
    fs::write(&config_path, config_toml).unwrap();

    // Load config
    let config = load_config_file(&config_path).unwrap();

    // CRITICAL: Verify both prompts contain FILE CONTENT, not FILE PATHS
    assert_eq!(
        config.system_prompt,
        Some(system_content.to_string()),
        "System prompt should contain file CONTENT"
    );
    assert_eq!(
        config.user_prompt,
        Some(user_content.to_string()),
        "User prompt should contain file CONTENT"
    );

    // Verify both file paths are PRESERVED for metadata tracking
    assert_eq!(
        config.system_prompt_file,
        Some(system_path.display().to_string()),
        "System prompt file path should be preserved for metadata tracking"
    );
    assert_eq!(
        config.user_prompt_file,
        Some(user_path.display().to_string()),
        "User prompt file path should be preserved for metadata tracking"
    );

    // Cleanup
    fs::remove_file(&system_path).ok();
    fs::remove_file(&user_path).ok();
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_rejects_both_inline_and_file_for_system_prompt() {
    // Create a system prompt file
    let system_file = NamedTempFile::new().unwrap();
    let system_path = system_file.path().with_extension("md");
    fs::write(&system_path, "Content from file").unwrap();

    // Create a config that INCORRECTLY specifies both inline AND file
    let config_json = format!(
        r#"{{
            "api_url": "http://localhost:11434/v1/chat/completions",
            "model": "test-model",
            "system_prompt": "Inline system prompt",
            "system_prompt_file": "{}",
            "user_prompt": "Test"
        }}"#,
        system_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should fail with clear error message
    let result = load_config_file(&config_path);
    assert!(
        result.is_err(),
        "Should reject config with both inline text and file path"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("both") && error_msg.contains("system_prompt"),
        "Error should mention conflict for system_prompt, got: {error_msg}"
    );

    // Cleanup
    fs::remove_file(&system_path).ok();
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_rejects_both_inline_and_file_for_user_prompt() {
    // Create a user prompt file
    let user_file = NamedTempFile::new().unwrap();
    let user_path = user_file.path().with_extension("md");
    fs::write(&user_path, "Content from file").unwrap();

    // Create a config that INCORRECTLY specifies both inline AND file
    let config_json = format!(
        r#"{{
            "api_url": "http://localhost:11434/v1/chat/completions",
            "model": "test-model",
            "system_prompt": "Test system",
            "user_prompt": "Inline user prompt",
            "user_prompt_file": "{}"
        }}"#,
        user_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should fail with clear error message
    let result = load_config_file(&config_path);
    assert!(
        result.is_err(),
        "Should reject config with both inline text and file path"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        (error_msg.contains("both") && error_msg.contains("user_prompt"))
            || (error_msg.contains("more than one") && error_msg.contains("user_prompt")),
        "Error should mention conflict for user_prompt, got: {error_msg}"
    );

    // Cleanup
    fs::remove_file(&user_path).ok();
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_rejects_missing_system_prompt() {
    // Config with NO system prompt (neither inline nor file)
    let config_json = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test-model",
        "user_prompt": "Test"
    }"#;

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should fail because system_prompt is required
    let result = load_config_file(&config_path);
    assert!(
        result.is_err(),
        "Should reject config without system_prompt"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("system_prompt"),
        "Error should mention missing system_prompt, got: {error_msg}"
    );

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_handles_nonexistent_system_prompt_file() {
    // Config referencing a file that DOESN'T EXIST
    let config_json = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test-model",
        "system_prompt_file": "/nonexistent/path/to/system.md",
        "user_prompt": "Test"
    }"#;

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should fail with clear error about missing file
    let result = load_config_file(&config_path);
    assert!(
        result.is_err(),
        "Should reject config with nonexistent file"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("system prompt file") || error_msg.contains("system.md"),
        "Error should mention the missing file, got: {error_msg}"
    );

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_handles_nonexistent_user_prompt_file() {
    // Config referencing a file that DOESN'T EXIST
    let config_json = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test-model",
        "system_prompt": "Test system",
        "user_prompt_file": "/nonexistent/path/to/user.md"
    }"#;

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should fail with clear error about missing file
    let result = load_config_file(&config_path);
    assert!(
        result.is_err(),
        "Should reject config with nonexistent file"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("user prompt file") || error_msg.contains("user.md"),
        "Error should mention the missing file, got: {error_msg}"
    );

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_config_supports_inline_prompts_without_files() {
    // Traditional config with inline text (no files)
    let config_json = r#"{
        "api_url": "http://localhost:11434/v1/chat/completions",
        "model": "test-model",
        "system_prompt": "Inline system prompt text",
        "user_prompt": "Inline user prompt text"
    }"#;

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Should work fine with inline text
    let config = load_config_file(&config_path).unwrap();

    assert_eq!(
        config.system_prompt,
        Some("Inline system prompt text".to_string())
    );
    assert_eq!(
        config.user_prompt,
        Some("Inline user prompt text".to_string())
    );

    // Cleanup
    fs::remove_file(&config_path).ok();
}
