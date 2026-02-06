use fortified_llm_client::guardrails::{
    config::RegexGuardrailConfig, GuardrailProvider, RegexGuardrail, Severity,
};

#[tokio::test]
async fn test_input_guardrail_trait_implementation() {
    let config = RegexGuardrailConfig::default();
    let guardrail = RegexGuardrail::new(config);

    // Test clean input (no patterns file, so only length validation)
    let result = guardrail.validate("Clean input").await.unwrap();
    assert!(result.passed);
    assert_eq!(guardrail.name(), "RegexGuardrail");
    assert!(result.quality_score.is_none()); // Regex guardrails don't have quality score
}

#[tokio::test]
async fn test_input_guardrail_max_length() {
    let config = RegexGuardrailConfig {
        max_length_bytes: 10,
        patterns_file: None,
        severity_threshold: Severity::Medium,
    };
    let guardrail = RegexGuardrail::new(config);

    let result = guardrail
        .validate("This is a very long input")
        .await
        .unwrap();
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.rule == "MAX_LENGTH"));
}

#[tokio::test]
async fn test_output_guardrail_trait_implementation() {
    let config = RegexGuardrailConfig::default();
    let guardrail = RegexGuardrail::new(config);

    // Test clean response
    let result = guardrail.validate("Clean response").await.unwrap();
    assert!(result.passed);
    assert_eq!(guardrail.name(), "RegexGuardrail");
    assert!(result.quality_score.is_none()); // Regex guardrails don't have quality score
}

#[tokio::test]
async fn test_output_guardrail_max_length() {
    let config = RegexGuardrailConfig {
        max_length_bytes: 10,
        patterns_file: None,
        severity_threshold: Severity::High,
    };
    let guardrail = RegexGuardrail::new(config);

    let result = guardrail
        .validate("This is a very long output that exceeds the limit")
        .await
        .unwrap();
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.rule == "MAX_LENGTH"));
}

#[tokio::test]
async fn test_input_guardrail_clean_input() {
    let config = RegexGuardrailConfig::default();
    let guardrail = RegexGuardrail::new(config);

    let result = guardrail.validate("Clean user input").await.unwrap();
    assert!(result.passed);
    assert_eq!(result.violations.len(), 0);
}

#[tokio::test]
async fn test_output_guardrail_clean_output() {
    let config = RegexGuardrailConfig::default();
    let guardrail = RegexGuardrail::new(config);

    let result = guardrail.validate("Clean output response").await.unwrap();
    assert!(result.passed);
    assert_eq!(result.violations.len(), 0);
}

#[tokio::test]
async fn test_severity_threshold() {
    // Test that severity threshold filtering works
    let config = RegexGuardrailConfig {
        max_length_bytes: 100000,
        patterns_file: None,
        severity_threshold: Severity::Critical, // Very high threshold
    };
    let guardrail = RegexGuardrail::new(config);

    let result = guardrail.validate("test").await.unwrap();
    assert!(result.passed);
}
