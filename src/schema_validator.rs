//! JSON Schema validation utilities
//!
//! Provides validation of user-provided JSON Schemas against the JSON Schema metaschema
//! to ensure schemas are well-formed before using them with LLM response_format.

use crate::error::CliError;
use serde_json::Value;

/// Validate a JSON Schema by attempting to compile it
///
/// This ensures the schema is well-formed and can be used for validation.
/// Uses the jsonschema crate's built-in validation which checks against
/// JSON Schema Draft 7 specification.
///
/// # Arguments
///
/// * `schema` - The JSON Schema to validate
///
/// # Returns
///
/// Ok(()) if the schema is valid and compilable, or CliError with detailed validation errors
///
/// # Example
///
/// ```no_run
/// use serde_json::json;
/// use fortified_llm_client::schema_validator::validate_json_schema;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" }
///     },
///     "required": ["name"]
/// });
///
/// validate_json_schema(&schema)?;
/// # Ok::<(), fortified_llm_client::CliError>(())
/// ```
pub fn validate_json_schema(schema: &Value) -> Result<(), CliError> {
    // Attempt to compile the schema - this validates it against Draft 7 metaschema
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(schema)
        .map_err(|e| {
            CliError::InvalidArguments(format!(
                "JSON Schema validation failed: {e}\n\
                \n\
                The provided schema does not conform to JSON Schema Draft 7 specification.\n\
                \n\
                Common issues:\n\
                - Invalid 'type' value (must be: object, array, string, number, integer, boolean, null)\n\
                - Malformed '$ref' references\n\
                - Invalid regex patterns in 'pattern' fields\n\
                - Incorrect format for 'properties', 'items', or 'required' fields\n\
                \n\
                References:\n\
                - JSON Schema specification: https://json-schema.org/draft-07/json-schema-release-notes.html\n\
                - Schema validator: https://www.jsonschemavalidator.net/"
            ))
        })?;

    Ok(())
}

/// Perform basic sanity checks on a JSON Schema
///
/// This is a lightweight check that doesn't require full metaschema validation.
/// Used for quick feedback on obviously invalid schemas.
///
/// # Checks
///
/// - Schema is a JSON object (not array, string, etc.)
/// - Schema has at least one of: "type", "properties", "items", "anyOf", "oneOf", "allOf"
/// - If "type" is present, it's a valid type or array of types
/// - If "properties" is present, it's an object
/// - If "required" is present, it's an array of strings
///
/// # Returns
///
/// Ok(()) if basic sanity checks pass, or CliError with helpful message
pub fn basic_schema_sanity_check(schema: &Value) -> Result<(), CliError> {
    // Must be an object
    let obj = schema.as_object().ok_or_else(|| {
        CliError::InvalidArguments(format!(
            "JSON Schema must be an object, got {}",
            match schema {
                Value::Array(_) => "array",
                Value::String(_) => "string",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::Null => "null",
                Value::Object(_) => "object", // unreachable
            }
        ))
    })?;

    // Must have at least one schema-defining keyword
    let has_schema_keyword = obj.contains_key("type")
        || obj.contains_key("properties")
        || obj.contains_key("items")
        || obj.contains_key("anyOf")
        || obj.contains_key("oneOf")
        || obj.contains_key("allOf")
        || obj.contains_key("$ref");

    if !has_schema_keyword {
        log::warn!(
            "Schema appears to be missing schema-defining keywords. \
            Expected at least one of: 'type', 'properties', 'items', 'anyOf', 'oneOf', 'allOf', '$ref'"
        );
    }

    // If "type" is present, validate it
    if let Some(type_val) = obj.get("type") {
        let valid_types = [
            "object", "array", "string", "number", "integer", "boolean", "null",
        ];

        let is_valid = match type_val {
            Value::String(s) => valid_types.contains(&s.as_str()),
            Value::Array(arr) => arr.iter().all(|t| {
                t.as_str()
                    .map(|s| valid_types.contains(&s))
                    .unwrap_or(false)
            }),
            _ => false,
        };

        if !is_valid {
            return Err(CliError::InvalidArguments(format!(
                "Invalid 'type' value: {type_val}\n\
                Valid types: {}",
                valid_types.join(", ")
            )));
        }
    }

    // If "properties" is present, must be an object
    if let Some(props) = obj.get("properties") {
        if !props.is_object() {
            return Err(CliError::InvalidArguments(
                "'properties' must be an object".to_string(),
            ));
        }
    }

    // If "required" is present, must be an array of strings
    if let Some(required) = obj.get("required") {
        if !required.is_array() {
            return Err(CliError::InvalidArguments(
                "'required' must be an array".to_string(),
            ));
        }

        if let Some(arr) = required.as_array() {
            if !arr.iter().all(|v| v.is_string()) {
                return Err(CliError::InvalidArguments(
                    "'required' array must contain only strings".to_string(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_simple_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        });

        assert!(validate_json_schema(&schema).is_ok());
    }

    #[test]
    fn test_valid_complex_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "email": { "type": "string", "format": "email" }
                    },
                    "required": ["id", "email"]
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            }
        });

        assert!(validate_json_schema(&schema).is_ok());
    }

    #[test]
    fn test_invalid_schema_bad_type() {
        let schema = json!({
            "type": "invalid_type",
            "properties": {}
        });

        let result = validate_json_schema(&schema);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("validation failed") || err_msg.contains("Invalid"));
    }

    #[test]
    fn test_invalid_schema_properties_not_object() {
        let schema = json!({
            "type": "object",
            "properties": "should_be_object"
        });

        let result = validate_json_schema(&schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_sanity_check_not_object() {
        let schema = json!("not an object");
        let result = basic_schema_sanity_check(&schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be an object"));
    }

    #[test]
    fn test_basic_sanity_check_invalid_type() {
        let schema = json!({
            "type": "invalid_type"
        });

        let result = basic_schema_sanity_check(&schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid 'type' value"));
    }

    #[test]
    fn test_basic_sanity_check_valid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        assert!(basic_schema_sanity_check(&schema).is_ok());
    }

    #[test]
    fn test_basic_sanity_check_required_not_array() {
        let schema = json!({
            "type": "object",
            "required": "should_be_array"
        });

        let result = basic_schema_sanity_check(&schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("'required' must be an array"));
    }

    #[test]
    fn test_valid_schema_with_refs() {
        let schema = json!({
            "definitions": {
                "address": {
                    "type": "object",
                    "properties": {
                        "street": { "type": "string" },
                        "city": { "type": "string" }
                    }
                }
            },
            "type": "object",
            "properties": {
                "billing": { "$ref": "#/definitions/address" },
                "shipping": { "$ref": "#/definitions/address" }
            }
        });

        assert!(validate_json_schema(&schema).is_ok());
    }
}
