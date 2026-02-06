// Response format integration tests
//
// Tests that the response field is correctly parsed as native JSON for json-object/json-schema
// formats and wrapped as string for text format.

use fortified_llm_client::{
    evaluate, EvaluationConfig, JsonSchemaDefinition, Provider, ResponseFormat,
};
use mockito::Server;
use serde_json::json;

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
async fn test_json_object_response_parsed_as_native_json() {
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
                        "content": "{\"name\":\"John\",\"age\":30,\"active\":true}"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/v1/chat/completions").await;
    config.response_format = Some(ResponseFormat::JsonObject);

    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed with json-object format");
    let output = result.unwrap();

    // Response should be parsed as native JSON object, not a string
    assert!(output.response.is_some(), "Response should exist");
    let response = output.response.as_ref().unwrap();
    assert!(
        response.is_object(),
        "Response should be a JSON object, not a string"
    );

    // Should be able to access fields directly without parsing
    assert_eq!(response["name"], "John");
    assert_eq!(response["age"], 30);
    assert_eq!(response["active"], true);

    // Verify final serialized output has no double-escaping
    let serialized = serde_json::to_string(&output).unwrap();
    assert!(
        serialized.contains(r#""name":"John""#),
        "Serialized output should contain clean JSON, not escaped quotes: {serialized}"
    );
    assert!(
        !serialized.contains(r#"\"{\\\"name\""#),
        "Should not have double-escaped JSON"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_json_schema_response_parsed_as_native_json() {
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
                        "content": "{\"items\":[\"apple\",\"banana\"],\"count\":2}"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/v1/chat/completions").await;
    config.response_format = Some(ResponseFormat::JsonSchema {
        json_schema: JsonSchemaDefinition {
            name: "test_schema".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "items": {"type": "array"},
                    "count": {"type": "number"}
                }
            }),
            strict: Some(true),
        },
    });

    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed with json-schema format");
    let output = result.unwrap();

    // Response should be parsed as native JSON
    let response = output.response.as_ref().unwrap();
    assert!(
        response.is_object(),
        "Response should be a JSON object for json-schema format"
    );

    // Verify array/object access works correctly
    assert!(response["items"].is_array(), "Items should be an array");
    assert_eq!(response["items"][0], "apple");
    assert_eq!(response["items"][1], "banana");
    assert_eq!(response["count"], 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_text_response_wrapped_as_string() {
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
                        "content": "Hello, this is a plain text response!"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/v1/chat/completions").await;
    config.response_format = Some(ResponseFormat::Text);

    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed with text format");
    let output = result.unwrap();

    // Response should be wrapped as a JSON string value
    let response = output.response.as_ref().unwrap();
    assert!(
        response.is_string(),
        "Response should be a JSON string for text format"
    );
    assert_eq!(
        response.as_str().unwrap(),
        "Hello, this is a plain text response!"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_invalid_json_with_json_object_format_falls_back_to_string() {
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
                        "content": "This is not valid JSON {broken syntax"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/v1/chat/completions").await;
    config.response_format = Some(ResponseFormat::JsonObject);

    let result = evaluate(config).await;

    assert!(
        result.is_ok(),
        "Should succeed and gracefully fallback to string when LLM returns invalid JSON"
    );
    let output = result.unwrap();

    // Should fallback to string representation
    let response = output.response.as_ref().unwrap();
    assert!(
        response.is_string(),
        "Response should fallback to string when JSON parsing fails"
    );
    assert_eq!(
        response.as_str().unwrap(),
        "This is not valid JSON {broken syntax"
    );

    mock.assert_async().await;

    // Note: A warning should be logged (checked manually in logs)
}

#[tokio::test]
async fn test_no_response_format_treats_as_text() {
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
                        "content": "Default text response when no format specified"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let config = create_test_config(server.url() + "/v1/chat/completions").await;
    // response_format is None (not specified)

    let result = evaluate(config).await;

    assert!(result.is_ok(), "Should succeed when no format specified");
    let output = result.unwrap();

    // Should default to string wrapping
    let response = output.response.as_ref().unwrap();
    assert!(
        response.is_string(),
        "Response should be wrapped as string when no format specified"
    );
    assert_eq!(
        response.as_str().unwrap(),
        "Default text response when no format specified"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_serialized_output_structure() {
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
                        "content": "{\"name\":\"Alice\",\"role\":\"admin\"}"
                    }
                }]
            }"#,
        )
        .create_async()
        .await;

    let mut config = create_test_config(server.url() + "/v1/chat/completions").await;
    config.response_format = Some(ResponseFormat::JsonObject);

    let result = evaluate(config).await;
    assert!(result.is_ok());
    let output = result.unwrap();

    // Serialize the entire CliOutput to JSON
    let serialized = serde_json::to_string_pretty(&output).unwrap();

    // Verify final JSON output structure is clean
    // Should NOT contain escaped quotes like: "response": "{\"name\":\"Alice\"}"
    // Should contain native JSON like: "response": {"name": "Alice"}
    assert!(
        serialized.contains(r#""name": "Alice""#) || serialized.contains(r#""name":"Alice""#),
        "Serialized output should have native JSON structure, not escaped string: {serialized}"
    );

    // Verify it doesn't have double-escaping
    assert!(
        !serialized.contains(r#"\"{\\\"name\""#),
        "Should not have double-escaped JSON in output"
    );

    // Verify overall structure contains expected fields
    assert!(
        serialized.contains(r#""status""#),
        "Should have status field"
    );
    assert!(
        serialized.contains(r#""response""#),
        "Should have response field"
    );
    assert!(
        serialized.contains(r#""metadata""#),
        "Should have metadata field"
    );

    mock.assert_async().await;
}
