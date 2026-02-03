// Comprehensive error handling tests
//
// Ensures all error variants have proper messages, exit codes, and context

use fortified_llm_client::CliError;

#[test]
fn test_all_error_variants_have_non_empty_messages() {
    let errors = vec![
        CliError::FileNotFound("test.txt".to_string()),
        CliError::InvalidArguments("bad argument".to_string()),
        CliError::InvalidResponse("API response parsing failed".to_string()),
        CliError::AuthenticationFailed("Invalid API key".to_string()),
        CliError::ContextLimitExceeded {
            required: 100000,
            limit: 8192,
            excess: 91808,
        },
        CliError::PdfProcessingFailed("Failed to extract PDF".to_string()),
    ];

    for error in errors {
        let msg = error.to_string();
        assert!(
            !msg.is_empty(),
            "Error message should not be empty for {error:?}"
        );
        assert!(
            msg.len() > 10,
            "Error message should be descriptive for {error:?}"
        );
    }
}

#[test]
fn test_error_display_includes_context() {
    let err = CliError::FileNotFound("missing.txt".to_string());
    let display = format!("{err}");
    assert!(
        display.contains("missing.txt"),
        "Error should include filename"
    );

    let err = CliError::InvalidArguments("temperature must be 0.0-2.0".to_string());
    let display = format!("{err}");
    assert!(
        display.contains("temperature"),
        "Error should include parameter name"
    );

    let err = CliError::ContextLimitExceeded {
        required: 100000,
        limit: 8192,
        excess: 91808,
    };
    let display = format!("{err}");
    assert!(
        display.contains("100000"),
        "Error should include required context"
    );
    assert!(display.contains("8192"), "Error should include limit");
}

#[test]
fn test_error_exit_codes_are_unique() {
    use std::collections::HashSet;

    let errors = vec![
        CliError::FileNotFound("test".to_string()),
        CliError::InvalidArguments("test".to_string()),
        CliError::InvalidResponse("test".to_string()),
        CliError::AuthenticationFailed("test".to_string()),
        CliError::ContextLimitExceeded {
            required: 100,
            limit: 50,
            excess: 50,
        },
        CliError::PdfProcessingFailed("test".to_string()),
    ];

    let mut codes = HashSet::new();
    for error in errors {
        let code = error.exit_code();
        assert!(
            code > 0 && code < 128,
            "Exit code should be in range 1-127, got {code}"
        );
        assert!(
            codes.insert(code),
            "Exit code {code} should be unique, but was used multiple times"
        );
    }
}

#[test]
fn test_error_code_strings() {
    let errors = vec![
        (CliError::FileNotFound("test".to_string()), "FILE_NOT_FOUND"),
        (
            CliError::InvalidArguments("test".to_string()),
            "INVALID_ARGUMENTS",
        ),
        (
            CliError::InvalidResponse("test".to_string()),
            "INVALID_RESPONSE",
        ),
        (
            CliError::AuthenticationFailed("test".to_string()),
            "AUTH_FAILED",
        ),
        (
            CliError::ContextLimitExceeded {
                required: 100,
                limit: 50,
                excess: 50,
            },
            "CONTEXT_LIMIT_EXCEEDED",
        ),
        (
            CliError::PdfProcessingFailed("test".to_string()),
            "PDF_PROCESSING_FAILED",
        ),
    ];

    for (error, expected_code) in errors {
        let code_str = error.code();
        assert_eq!(
            code_str, expected_code,
            "Error code should match expected value"
        );
        assert!(
            !code_str.is_empty(),
            "Error code string should not be empty"
        );
        assert!(
            code_str.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
            "Error code should be SCREAMING_SNAKE_CASE, got '{code_str}'"
        );
    }
}

#[test]
fn test_file_not_found_error_formatting() {
    let err = CliError::FileNotFound("config.json".to_string());
    let msg = err.to_string();

    assert!(msg.contains("config.json"));
    assert!(msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("failed"));
}

#[test]
fn test_invalid_arguments_error_formatting() {
    let err = CliError::InvalidArguments("temperature must be between 0.0 and 2.0".to_string());
    let msg = err.to_string();

    assert!(msg.contains("temperature"));
    assert!(msg.contains("0.0"));
    assert!(msg.contains("2.0"));
}

#[test]
fn test_invalid_response_error_formatting() {
    let err = CliError::InvalidResponse("HTTP 404 error: Not Found".to_string());
    let msg = err.to_string();

    assert!(msg.contains("404"));
    assert!(msg.contains("Not Found"));
}

#[test]
fn test_authentication_failed_error_formatting() {
    let err = CliError::AuthenticationFailed("Invalid API key provided".to_string());
    let msg = err.to_string();

    assert!(msg.contains("API key") || msg.contains("Invalid"));
}

#[test]
fn test_context_limit_exceeded_formatting() {
    let err = CliError::ContextLimitExceeded {
        required: 150000,
        limit: 131072,
        excess: 18928,
    };
    let msg = err.to_string();

    assert!(msg.contains("150000") || msg.contains("150,000"));
    assert!(msg.contains("131072") || msg.contains("131,072"));
    assert!(
        msg.to_lowercase().contains("context")
            || msg.to_lowercase().contains("token")
            || msg.to_lowercase().contains("limit")
    );
}

#[test]
fn test_pdf_processing_error_formatting() {
    let err = CliError::PdfProcessingFailed("Failed to parse PDF structure".to_string());
    let msg = err.to_string();

    assert!(msg.contains("PDF"));
    assert!(msg.contains("parse") || msg.contains("failed") || msg.contains("processing"));
}

#[test]
fn test_error_code_matches_error_type() {
    assert_eq!(
        CliError::FileNotFound("test".to_string()).code(),
        "FILE_NOT_FOUND"
    );
    assert_eq!(
        CliError::InvalidArguments("test".to_string()).code(),
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        CliError::InvalidResponse("test".to_string()).code(),
        "INVALID_RESPONSE"
    );
    assert_eq!(
        CliError::AuthenticationFailed("test".to_string()).code(),
        "AUTH_FAILED"
    );
    assert_eq!(
        CliError::ContextLimitExceeded {
            required: 100,
            limit: 50,
            excess: 50,
        }
        .code(),
        "CONTEXT_LIMIT_EXCEEDED"
    );
    assert_eq!(
        CliError::PdfProcessingFailed("test".to_string()).code(),
        "PDF_PROCESSING_FAILED"
    );
}

#[test]
fn test_error_debug_output_is_useful() {
    let err = CliError::FileNotFound("test.txt".to_string());
    let debug = format!("{err:?}");

    // Debug output should show variant name and inner value
    assert!(debug.contains("FileNotFound"));
    assert!(debug.contains("test.txt"));
}

// Note: These tests are commented out because CliError doesn't currently
// implement From for these error types. This is intentional - errors are
// handled at the point they occur rather than converted automatically.
// Keeping these tests as documentation of what COULD be tested if we
// add automatic error conversions in the future.

// #[test]
// fn test_error_from_io_error() {
//     use std::io;
//     let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
//     let cli_err: CliError = io_err.into();
//     let msg = cli_err.to_string();
//     assert!(msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("file"));
// }

// #[test]
// fn test_error_from_serde_json_error() {
//     let json_err = serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
//     let cli_err: CliError = json_err.into();
//     let msg = cli_err.to_string();
//     assert!(!msg.is_empty());
// }

// #[test]
// fn test_error_from_toml_error() {
//     let toml_err = toml::from_str::<toml::Value>("[invalid").unwrap_err();
//     let cli_err: CliError = toml_err.into();
//     let msg = cli_err.to_string();
//     assert!(!msg.is_empty());
// }

// #[test]
// fn test_error_chain_preserves_context() {
//     // Test that converting from underlying errors preserves useful context
//     let json_err = serde_json::from_str::<serde_json::Value>("{bad json}").unwrap_err();
//     let cli_err: CliError = json_err.into();
//     // The error message should mention JSON
//     let msg = cli_err.to_string().to_lowercase();
//     assert!(
//         msg.contains("json") || msg.contains("parse") || msg.contains("invalid"),
//         "Error message should indicate JSON parsing issue, got: {}",
//         cli_err
//     );
// }
