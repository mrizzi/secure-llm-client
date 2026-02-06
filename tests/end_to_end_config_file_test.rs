// End-to-end integration test verifying config file workflow
//
// This test simulates the ACTUAL USER WORKFLOW:
// 1. User creates config file with file paths for prompts
// 2. User runs the CLI with that config file
// 3. Verify the API receives FILE CONTENT, not file paths
//
// REGRESSION: This would have caught the bug where file paths were sent to API

use fortified_llm_client::{config::load_config_file, evaluate, EvaluationConfig};
use mockito::Server;
use std::fs;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_end_to_end_config_with_file_paths() {
    // Step 1: Create prompt files (simulating user's prompt files)
    let system_file = NamedTempFile::new().unwrap();
    let system_path = system_file.path().with_extension("md");
    let system_content = "You are a document analysis expert.\nAnalyze the following document.";
    fs::write(&system_path, system_content).unwrap();

    let user_file = NamedTempFile::new().unwrap();
    let user_path = user_file.path().with_extension("md");
    let user_content = "# Technical Document\n\n## System: TestApp\n\nThis is a test document.";
    fs::write(&user_path, user_content).unwrap();

    // Step 2: Create mock API server
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"choices": [{"message": {"role": "assistant", "content": "Analysis complete"}}]}"#,
        )
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "test-model",
            "messages": [
                {
                    "role": "system",
                    // CRITICAL: Must match FILE CONTENT, not file path
                    "content": system_content
                },
                {
                    "role": "user",
                    // CRITICAL: Must match FILE CONTENT, not file path
                    "content": user_content
                }
            ],
            "temperature": 0.0
        })))
        .create_async()
        .await;

    // Step 3: Create config file with file paths (like a real user would)
    let config_toml = format!(
        r#"
api_url = "{}/v1/chat/completions"
model = "test-model"
system_prompt_file = "{}"
user_prompt_file = "{}"
temperature = 0.0
"#,
        server.url(),
        system_path.to_str().unwrap(),
        user_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("toml");
    fs::write(&config_path, config_toml).unwrap();

    // Step 4: Load config (simulating CLI loading config file)
    let file_config = load_config_file(&config_path).unwrap();

    // Verify config loaded file content
    assert_eq!(
        file_config.system_prompt,
        Some(system_content.to_string()),
        "Config should have loaded system prompt FILE CONTENT"
    );
    assert_eq!(
        file_config.user_prompt,
        Some(user_content.to_string()),
        "Config should have loaded user prompt FILE CONTENT"
    );

    // Step 5: Build evaluation config
    let eval_config = EvaluationConfig {
        api_url: file_config.api_url,
        model: file_config.model,
        system_prompt: file_config.system_prompt.unwrap(),
        user_prompt: file_config.user_prompt.unwrap(),
        provider: None,
        temperature: file_config.temperature,
        max_tokens: file_config.max_tokens,
        seed: file_config.seed,
        api_key: None,
        timeout_secs: file_config.timeout_secs,
        validate_tokens: file_config.validate_tokens,
        context_limit: file_config.context_limit,
        response_format: None,
        pdf_input: None,
        input_guardrails: None,
        output_guardrails: None,
        system_prompt_file: None,
        user_prompt_file: None,
    };

    // Step 6: Execute evaluation (makes actual HTTP request to mock server)
    let result = evaluate(eval_config).await;

    // Verify success
    assert!(result.is_ok(), "Evaluation should succeed");

    // CRITICAL: Verify mock was called with FILE CONTENT, not file paths
    // If this fails, it means the API received file paths instead of content
    mock.assert_async().await;

    // Cleanup
    fs::remove_file(&system_path).ok();
    fs::remove_file(&user_path).ok();
    fs::remove_file(&config_path).ok();
}

#[tokio::test]
async fn test_end_to_end_config_with_inline_text() {
    // Test that inline text (non-file) still works
    let mut server = Server::new_async().await;
    let system_content = "You are helpful";
    let user_content = "Say hello";

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": "Hello!"}}]}"#)
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "test-model",
            "messages": [
                {"role": "system", "content": system_content},
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.5
        })))
        .create_async()
        .await;

    // Config with inline text (traditional approach)
    let config_json = format!(
        r#"{{
            "api_url": "{}/v1/chat/completions",
            "model": "test-model",
            "system_prompt": "{}",
            "user_prompt": "{}",
            "temperature": 0.5
        }}"#,
        server.url(),
        system_content,
        user_content
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("json");
    fs::write(&config_path, config_json).unwrap();

    // Load and execute
    let file_config = load_config_file(&config_path).unwrap();

    let eval_config = EvaluationConfig {
        api_url: file_config.api_url,
        model: file_config.model,
        system_prompt: file_config.system_prompt.unwrap(),
        user_prompt: file_config.user_prompt.unwrap(),
        provider: None,
        temperature: file_config.temperature,
        max_tokens: file_config.max_tokens,
        seed: file_config.seed,
        api_key: None,
        timeout_secs: file_config.timeout_secs,
        validate_tokens: file_config.validate_tokens,
        context_limit: file_config.context_limit,
        response_format: None,
        pdf_input: None,
        input_guardrails: None,
        output_guardrails: None,
        system_prompt_file: None,
        user_prompt_file: None,
    };

    let result = evaluate(eval_config).await;
    assert!(result.is_ok());
    mock.assert_async().await;

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[tokio::test]
async fn test_actual_document_analysis_workflow() {
    // Simulate the ACTUAL document analysis workflow user would do

    // Create a realistic technical document
    let doc_content = r#"# Technical Document: E-Commerce Platform

## 1. System Overview
The system is an e-commerce platform handling customer orders and payments.

## 2. Assets
- Customer PII (names, addresses, emails)
- Payment card data (processed via Stripe)
- Order history database

## 3. Data Classification
- Customer PII: CONFIDENTIAL
- Payment data: RESTRICTED (PCI-DSS)
- Public catalog: PUBLIC

## 4. Security Controls
- TLS 1.3 for all communications
- AES-256 encryption at rest
- Role-based access control (RBAC)
"#;

    let doc_file = NamedTempFile::new().unwrap();
    let doc_path = doc_file.path().with_extension("md");
    fs::write(&doc_path, doc_content).unwrap();

    // Create document analysis system prompt
    let system_prompt_content = r#"You are a technical document analyst.
Analyze the provided technical document.
Evaluate completeness and identify gaps."#;

    let system_file = NamedTempFile::new().unwrap();
    let system_path = system_file.path().with_extension("md");
    fs::write(&system_path, system_prompt_content).unwrap();

    // Mock API
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": "Document analysis: 75% complete. Missing: deployment architecture, security controls."}}]}"#)
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "gpt-oss:20b",
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt_content  // Must be file content
                },
                {
                    "role": "user",
                    "content": doc_content  // Must be document file content
                }
            ]
        })))
        .create_async()
        .await;

    // Create realistic config (like user would have)
    let config_toml = format!(
        r#"
# Document Analysis Configuration
api_url = "{}/v1/chat/completions"
model = "gpt-oss:20b"

# Prompts loaded from files
system_prompt_file = "{}"
user_prompt_file = "{}"

# Model parameters
temperature = 0.0
"#,
        server.url(),
        system_path.to_str().unwrap(),
        doc_path.to_str().unwrap()
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_path = config_file.path().with_extension("toml");
    fs::write(&config_path, config_toml).unwrap();

    // Load config
    let file_config = load_config_file(&config_path).unwrap();

    // CRITICAL ASSERTIONS: Verify file content was loaded, not paths
    assert!(
        file_config
            .system_prompt
            .as_ref()
            .unwrap()
            .contains("technical document analyst"),
        "System prompt should contain actual content, not file path"
    );
    assert!(
        file_config
            .user_prompt
            .as_ref()
            .unwrap()
            .contains("E-Commerce Platform"),
        "User prompt should contain actual document content, not file path"
    );

    // Execute
    let eval_config = EvaluationConfig {
        api_url: file_config.api_url,
        model: file_config.model,
        system_prompt: file_config.system_prompt.unwrap(),
        user_prompt: file_config.user_prompt.unwrap(),
        provider: None,
        temperature: file_config.temperature,
        max_tokens: file_config.max_tokens,
        seed: file_config.seed,
        api_key: None,
        timeout_secs: file_config.timeout_secs,
        validate_tokens: file_config.validate_tokens,
        context_limit: file_config.context_limit,
        response_format: None,
        pdf_input: None,
        input_guardrails: None,
        output_guardrails: None,
        system_prompt_file: None,
        user_prompt_file: None,
    };

    let result = evaluate(eval_config).await;
    assert!(result.is_ok());

    // Verify API received file content, not paths
    mock.assert_async().await;

    // Cleanup
    fs::remove_file(&doc_path).ok();
    fs::remove_file(&system_path).ok();
    fs::remove_file(&config_path).ok();
}
