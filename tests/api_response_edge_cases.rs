// API response edge case tests
//
// Tests how the client handles malformed, incomplete, or unexpected API responses

use fortified_llm_client::{evaluate, EvaluationConfig, Provider};
use mockito::Server;

async fn create_test_config(api_url: String) -> EvaluationConfig {
    EvaluationConfig {
        api_url,
        model: "test-model".to_string(),
        system_prompt: "Test system".to_string(),
        user_prompt: "Test user".to_string(),
        provider: Some(Provider::OpenAI),
        temperature: 0.0,
        max_tokens: Some(100),
        seed: None,
        api_key: Some("test-key".to_string()),
        timeout_secs: 5,
        validate_tokens: false,
        context_limit: None,
        response_format: None,
        pdf_input: None,
        input_guardrails: None,
        output_guardrails: None,
        system_prompt_file: None,
        user_prompt_file: None,
    }
}

#[tokio::test]
async fn test_api_malformed_json_response() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{invalid json syntax")
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on malformed JSON");
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(
            msg.to_lowercase().contains("json") || msg.to_lowercase().contains("parse"),
            "Error should mention JSON parsing issue"
        );
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_missing_choices_field() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": "test-123", "model": "test"}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail when choices field is missing");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_empty_choices_array() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": []}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail when choices array is empty");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_missing_message_field() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"index": 0}]}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail when message field is missing");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_missing_content_field() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant"}}]}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(
        result.is_err(),
        "Should fail when message content is missing"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_empty_response_body() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("")
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on empty response body");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_404_error() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Model not found", "type": "invalid_request_error"}}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on 404 error");
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("404"), "Error should mention 404 status");
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_500_error() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Internal server error", "type": "server_error"}}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on 500 error");
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("500"), "Error should mention 500 status");
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_rate_limit_429() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Rate limit exceeded", "type": "rate_limit_error"}}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on rate limit");
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("429"), "Error should mention 429 status");
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_unauthorized_401() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Invalid API key", "type": "authentication_error"}}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail on authentication error");
    if let Err(err) = result {
        let msg = err.to_string();
        eprintln!("Actual error message: {msg}");
        assert!(
            msg.contains("401")
                || msg.to_lowercase().contains("authentication")
                || msg.to_lowercase().contains("unauthorized"),
            "Error should mention 401 or authentication, got: {msg}"
        );
    }

    mock.assert_async().await;
}

// Note: Timeout test commented out due to deprecated mockito API.
// TODO: Re-enable this test with mockito's with_chunked_body API
// #[tokio::test]
// async fn test_api_timeout() {
//     // Test that timeout is handled correctly
// }

#[tokio::test]
async fn test_api_non_json_content_type() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Plain text response")
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail when content-type is not JSON");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_unexpected_field_types() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": "not-an-array"}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(
        result.is_err(),
        "Should fail when field types don't match schema"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_null_content() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": null}}]}"#)
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_err(), "Should fail when content is null");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_api_extra_fields_ignored() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "id": "test-123",
            "model": "test-model",
            "created": 1234567890,
            "extra_field": "ignored",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                },
                "finish_reason": "stop",
                "extra_choice_field": "also ignored"
            }],
            "usage": {"total_tokens": 50}
        }"#,
        )
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed and ignore extra fields");
    let output = result.unwrap();
    assert_eq!(
        output.response,
        Some(serde_json::Value::String("Test response".to_string()))
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_ollama_api_format() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "Ollama response text"}"#)
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/api/generate").await;
    config.provider = Some(Provider::Ollama);

    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed with Ollama format");
    let output = result.unwrap();
    assert_eq!(
        output.response,
        Some(serde_json::Value::String(
            "Ollama response text".to_string()
        ))
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_ollama_missing_response_field() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"model": "test"}"#)
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/api/generate").await;
    config.provider = Some(Provider::Ollama);

    let result = evaluate(config).await;

    assert!(
        result.is_err(),
        "Should fail when Ollama response field is missing"
    );

    mock.assert_async().await;
}
