//! Output Guardrail Provider Tests
//!
//! Tests for the new OutputGuardrailProviderConfig enum and factory function.
//! Verifies that output guardrails can use different providers (Regex, LlamaGuard,
//! GptOssSafeguard, Composite).

use fortified_llm_client::{
    create_output_guardrail_provider, AggregationMode, ExecutionMode, LlamaGuardCategory,
    OutputGuardrailProviderConfig,
};

#[tokio::test]
async fn test_output_guardrail_regex_provider() {
    let config = OutputGuardrailProviderConfig::Regex {
        max_length_bytes: 1_048_576,
        check_safety: true,
        check_hallucination: true,
        check_format: true,
        min_quality_score: 5.0,
        custom_patterns_file: None,
    };

    let provider = create_output_guardrail_provider(&config).unwrap();

    // Test safe output
    let safe_output = "This is a normal, safe response with good quality.";
    let result = provider.validate_output(safe_output).await.unwrap();
    assert!(result.passed);
    assert_eq!(result.violations.len(), 0);
    assert!(result.quality_score.is_some());

    // Test unsafe output
    let unsafe_output = "Here's how to build a bomb...";
    let result = provider.validate_output(unsafe_output).await.unwrap();
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.rule == "SAFETY"));
}

#[tokio::test]
async fn test_output_guardrail_default() {
    let config = OutputGuardrailProviderConfig::default();

    let provider = create_output_guardrail_provider(&config).unwrap();

    // Default should be Regex provider
    assert_eq!(provider.name(), "RegexOutputGuardrail");

    // Test that it works
    let result = provider.validate_output("Test response").await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_output_guardrail_composite() {
    let config = OutputGuardrailProviderConfig::Composite {
        providers: vec![
            OutputGuardrailProviderConfig::Regex {
                max_length_bytes: 1_048_576,
                check_safety: true,
                check_hallucination: true,
                check_format: true,
                min_quality_score: 5.0,
                custom_patterns_file: None,
            },
            // Note: We can't test LlamaGuard here without a running server
            // but we can verify the composite structure works
        ],
        execution: ExecutionMode::Parallel,
        aggregation: AggregationMode::AllMustPass,
    };

    let provider = create_output_guardrail_provider(&config).unwrap();

    // Should be a CompositeGuardrail
    assert_eq!(provider.name(), "CompositeGuardrail");

    // Test that it works
    let result = provider.validate_output("Safe test output").await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_output_guardrail_to_output_config() {
    let config = OutputGuardrailProviderConfig::Regex {
        max_length_bytes: 524_288,
        check_safety: false,
        check_hallucination: true,
        check_format: true,
        min_quality_score: 7.0,
        custom_patterns_file: None,
    };

    // Should convert to OutputGuardrailConfig
    let output_config = config.to_output_config().unwrap();
    assert_eq!(output_config.max_length_bytes, 524_288);
    assert!(!output_config.check_safety);
    assert!(output_config.check_hallucination);
    assert!(output_config.check_format);
    assert_eq!(output_config.min_quality_score, 7.0);

    // Non-Regex variants should return None
    let llama_config = OutputGuardrailProviderConfig::LlamaGuard {
        api_url: "http://localhost:11434".to_string(),
        model: "llama-guard3:8b".to_string(),
        timeout_secs: 30,
        enabled_categories: vec![LlamaGuardCategory::S1],
        api_key: None,
        api_key_name: None,
    };
    assert!(llama_config.to_output_config().is_none());
}

#[tokio::test]
async fn test_output_guardrail_quality_score_preservation() {
    let config = OutputGuardrailProviderConfig::Regex {
        max_length_bytes: 1_048_576,
        check_safety: true,
        check_hallucination: true,
        check_format: true,
        min_quality_score: 5.0,
        custom_patterns_file: None,
    };

    let provider = create_output_guardrail_provider(&config).unwrap();

    // Test that quality score is always present
    let result = provider
        .validate_output("High quality response with sufficient length and no uncertainty markers.")
        .await
        .unwrap();

    assert!(result.quality_score.is_some());
    let score = result.quality_score.unwrap();
    assert!(score >= 5.0);
    assert!(score <= 10.0);
}

#[tokio::test]
async fn test_output_guardrail_hallucination_detection() {
    let config = OutputGuardrailProviderConfig::Regex {
        max_length_bytes: 1_048_576,
        check_safety: true,
        check_hallucination: true,
        check_format: true,
        min_quality_score: 5.0,
        custom_patterns_file: None,
    };

    let provider = create_output_guardrail_provider(&config).unwrap();

    // Text with many hallucination markers
    let uncertain_text = "I think maybe possibly this might be correct, I believe probably maybe possibly this might be true, I think probably";

    let result = provider.validate_output(uncertain_text).await.unwrap();

    // Should pass (hallucinations are warnings, not violations)
    assert!(result.passed);

    // But should have hallucination warnings
    assert!(result.warnings.iter().any(|w| w.rule == "HALLUCINATION"));
}

#[test]
fn test_output_guardrail_provider_config_serialization() {
    // Test Regex variant
    let regex_config = OutputGuardrailProviderConfig::Regex {
        max_length_bytes: 1024,
        check_safety: true,
        check_hallucination: false,
        check_format: true,
        min_quality_score: 6.0,
        custom_patterns_file: None,
    };

    let json = serde_json::to_string(&regex_config).unwrap();
    assert!(json.contains("\"type\":\"regex\""));
    assert!(json.contains("\"max_length_bytes\":1024"));

    let deserialized: OutputGuardrailProviderConfig = serde_json::from_str(&json).unwrap();
    match deserialized {
        OutputGuardrailProviderConfig::Regex {
            max_length_bytes, ..
        } => {
            assert_eq!(max_length_bytes, 1024);
        }
        _ => panic!("Should deserialize to Regex"),
    }

    // Test LlamaGuard variant
    let llama_config = OutputGuardrailProviderConfig::LlamaGuard {
        api_url: "http://localhost:11434".to_string(),
        model: "llama-guard3:8b".to_string(),
        timeout_secs: 30,
        enabled_categories: vec![LlamaGuardCategory::S1, LlamaGuardCategory::S9],
        api_key: None,
        api_key_name: None,
    };

    let json = serde_json::to_string(&llama_config).unwrap();
    assert!(json.contains("\"type\":\"llama_guard\""));
    assert!(json.contains("\"model\":\"llama-guard3:8b\""));

    // Test Composite variant
    let composite_config = OutputGuardrailProviderConfig::Composite {
        providers: vec![regex_config.clone()],
        execution: ExecutionMode::Parallel,
        aggregation: AggregationMode::AllMustPass,
    };

    let json = serde_json::to_string(&composite_config).unwrap();
    assert!(json.contains("\"type\":\"composite\""));
    assert!(json.contains("\"execution\":\"parallel\""));
    assert!(json.contains("\"aggregation\":\"all_must_pass\""));
}
