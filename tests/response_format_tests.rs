use fortified_llm_client::ResponseFormat;
use serde_json::json;

#[test]
fn test_text_format_serialization() {
    let format = ResponseFormat::text();
    let serialized = serde_json::to_value(&format).unwrap();
    assert_eq!(serialized, json!({"type": "text"}));
}

#[test]
fn test_json_object_serialization() {
    let format = ResponseFormat::json();
    let serialized = serde_json::to_value(&format).unwrap();
    assert_eq!(serialized, json!({"type": "json_object"}));
}

#[test]
fn test_json_schema_serialization_strict() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer", "minimum": 0 }
        },
        "required": ["name", "age"]
    });

    let format = ResponseFormat::json_schema("person".to_string(), schema.clone(), true);
    let serialized = serde_json::to_value(&format).unwrap();

    assert_eq!(serialized["type"], "json_schema");
    assert_eq!(serialized["json_schema"]["name"], "person");
    assert_eq!(serialized["json_schema"]["strict"], true);
    assert_eq!(serialized["json_schema"]["schema"], schema);
}

#[test]
fn test_json_schema_serialization_non_strict() {
    let schema = json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" }
        }
    });

    let format = ResponseFormat::json_schema("document".to_string(), schema.clone(), false);
    let serialized = serde_json::to_value(&format).unwrap();

    assert_eq!(serialized["type"], "json_schema");
    assert_eq!(serialized["json_schema"]["name"], "document");
    assert_eq!(serialized["json_schema"]["strict"], false);
    assert_eq!(serialized["json_schema"]["schema"], schema);
}

#[test]
fn test_text_format_deserialization() {
    let json_str = r#"{"type": "text"}"#;
    let format: ResponseFormat = serde_json::from_str(json_str).unwrap();

    match format {
        ResponseFormat::Text => {} // Success
        _ => panic!("Expected ResponseFormat::Text"),
    }
}

#[test]
fn test_json_object_deserialization() {
    let json_str = r#"{"type": "json_object"}"#;
    let format: ResponseFormat = serde_json::from_str(json_str).unwrap();

    match format {
        ResponseFormat::JsonObject => {} // Success
        _ => panic!("Expected ResponseFormat::JsonObject"),
    }
}

#[test]
fn test_json_schema_deserialization() {
    let json_str = r#"{
        "type": "json_schema",
        "json_schema": {
            "name": "user",
            "strict": true,
            "schema": {
                "type": "object",
                "properties": {
                    "username": { "type": "string" }
                }
            }
        }
    }"#;

    let format: ResponseFormat = serde_json::from_str(json_str).unwrap();

    match format {
        ResponseFormat::JsonSchema { json_schema } => {
            assert_eq!(json_schema.name, "user");
            assert_eq!(json_schema.strict, Some(true));
            assert_eq!(json_schema.schema["type"], "object");
            assert_eq!(
                json_schema.schema["properties"]["username"]["type"],
                "string"
            );
        }
        _ => panic!("Expected ResponseFormat::JsonSchema"),
    }
}

#[test]
fn test_response_format_clone() {
    let schema = json!({"type": "object"});
    let format1 = ResponseFormat::json_schema("test".to_string(), schema, true);
    let format2 = format1.clone();

    let serialized1 = serde_json::to_value(&format1).unwrap();
    let serialized2 = serde_json::to_value(&format2).unwrap();

    assert_eq!(serialized1, serialized2);
}

#[test]
fn test_response_format_debug() {
    let format = ResponseFormat::text();
    let debug_str = format!("{format:?}");
    assert!(debug_str.contains("Text"));

    let format = ResponseFormat::json();
    let debug_str = format!("{format:?}");
    assert!(debug_str.contains("JsonObject"));

    let schema = json!({"type": "string"});
    let format = ResponseFormat::json_schema("name".to_string(), schema, false);
    let debug_str = format!("{format:?}");
    assert!(debug_str.contains("JsonSchema"));
}

#[test]
fn test_json_schema_with_complex_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "users": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" },
                        "roles": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["id", "name"]
                }
            },
            "total": { "type": "integer", "minimum": 0 }
        },
        "required": ["users", "total"],
        "additionalProperties": false
    });

    let format = ResponseFormat::json_schema("users_list".to_string(), schema.clone(), true);
    let serialized = serde_json::to_value(&format).unwrap();

    assert_eq!(serialized["type"], "json_schema");
    assert_eq!(serialized["json_schema"]["name"], "users_list");
    assert_eq!(serialized["json_schema"]["strict"], true);

    // Verify complex schema structure is preserved
    assert_eq!(
        serialized["json_schema"]["schema"]["properties"]["users"]["type"],
        "array"
    );
    assert_eq!(
        serialized["json_schema"]["schema"]["properties"]["users"]["items"]["properties"]["id"]
            ["type"],
        "integer"
    );
}

#[test]
fn test_roundtrip_serialization_text() {
    let original = ResponseFormat::text();
    let json_str = serde_json::to_string(&original).unwrap();
    let deserialized: ResponseFormat = serde_json::from_str(&json_str).unwrap();
    let reserialized = serde_json::to_string(&deserialized).unwrap();

    assert_eq!(json_str, reserialized);
}

#[test]
fn test_roundtrip_serialization_json_object() {
    let original = ResponseFormat::json();
    let json_str = serde_json::to_string(&original).unwrap();
    let deserialized: ResponseFormat = serde_json::from_str(&json_str).unwrap();
    let reserialized = serde_json::to_string(&deserialized).unwrap();

    assert_eq!(json_str, reserialized);
}

#[test]
fn test_roundtrip_serialization_json_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "value": { "type": "number" }
        }
    });

    let original = ResponseFormat::json_schema("metric".to_string(), schema, false);
    let json_str = serde_json::to_string(&original).unwrap();
    let deserialized: ResponseFormat = serde_json::from_str(&json_str).unwrap();
    let reserialized = serde_json::to_string(&deserialized).unwrap();

    assert_eq!(json_str, reserialized);
}

#[test]
fn test_openai_api_format_text() {
    // Verify the exact format that OpenAI expects for text mode
    let format = ResponseFormat::text();
    let serialized = serde_json::to_string(&format).unwrap();
    assert_eq!(serialized, r#"{"type":"text"}"#);
}

#[test]
fn test_openai_api_format_json_object() {
    // Verify the exact format that OpenAI expects for json_object mode
    let format = ResponseFormat::json();
    let serialized = serde_json::to_string(&format).unwrap();
    assert_eq!(serialized, r#"{"type":"json_object"}"#);
}

#[test]
fn test_openai_api_format_json_schema() {
    // Verify the exact format that OpenAI expects for json_schema mode
    let schema = json!({"type": "object", "properties": {}});
    let format = ResponseFormat::json_schema("test".to_string(), schema, true);
    let serialized = serde_json::to_value(&format).unwrap();

    // Must have these exact fields at top level
    assert_eq!(serialized.get("type").unwrap(), "json_schema");

    // Must have json_schema object with name, schema, and strict
    let json_schema = serialized.get("json_schema").unwrap();
    assert_eq!(json_schema.get("name").unwrap(), "test");
    assert_eq!(json_schema.get("strict").unwrap(), true);
    assert!(json_schema.get("schema").is_some());
}
