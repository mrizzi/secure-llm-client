//! Generic LLM evaluation library
//!
//! Provides embeddable API for LLM invocation with guardrails and validation.

mod client;
pub mod config;
pub mod config_builder;
pub mod constants;
mod error;
pub mod guardrails;
pub mod model_registry;
mod models;
mod output;
mod pdf;
mod provider;
pub mod providers;
pub mod schema_validator;
mod token_estimator;

pub use client::{LlmClient, Provider};
pub use config::{load_config_file, ConfigFileRequest};
pub use error::CliError;
pub use guardrails::{
    create_guardrail_provider,
    create_output_guardrail_provider,

    AggregationMode,
    ExecutionMode,
    // Configuration
    GuardrailConfig,
    // Trait types
    GuardrailProvider,
    GuardrailProviderConfig,
    GuardrailResult,
    HybridGuardrail,

    // Concrete implementations
    InputGuardrail,
    InputGuardrailConfig,
    LlamaGuardCategory,
    LlamaGuardConfig,
    LlamaGuardProvider,
    LlamaPromptGuardConfig,
    LlamaPromptGuardProvider,
    LlamaPromptGuardResult,
    OutputGuardrail,
    OutputGuardrailConfig,
    OutputGuardrailProviderConfig,
    ProviderSpecificResult,

    // Common types
    Severity,
    Violation,
};
pub use models::*;
pub use output::{CliOutput, ErrorInfo, Metadata};
pub use pdf::{
    extract_text_from_pdf, is_docling_available, to_markdown, ContentFormat, PdfContent,
};
pub use provider::{InvokeParams, LlmProvider, ProviderType};
pub use providers::{create_provider, detect_provider_type, OllamaProvider, OpenAIProvider};
pub use token_estimator::TokenEstimator;

use std::{path::PathBuf, time::Instant};

/// Configuration for LLM evaluation
#[derive(Debug, Clone)]
pub struct EvaluationConfig {
    pub api_url: String,
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub provider: Option<Provider>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub seed: Option<u64>,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub validate_tokens: bool,
    pub context_limit: Option<usize>,
    pub response_format: Option<ResponseFormat>,
    pub pdf_input: Option<PathBuf>,
    pub input_guardrails: Option<GuardrailProviderConfig>,
    // Source tracking for metadata (mutually exclusive with inline text)
    pub system_prompt_file: Option<PathBuf>,
    pub user_prompt_file: Option<PathBuf>,
}

/// Helper to create Metadata from config
fn create_metadata(
    config: &EvaluationConfig,
    user_prompt: &str,
    tokens_estimated: usize,
    latency_ms: u64,
    output_guardrails_enabled: bool,
) -> Metadata {
    Metadata {
        // Execution results
        model: config.model.clone(),
        tokens_estimated,
        latency_ms,
        timestamp: chrono::Utc::now().to_rfc3339(),

        // Input configuration (for reproducibility)
        api_url: config.api_url.clone(),
        provider: config.provider.map(|p| format!("{p:?}")),
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        seed: config.seed,
        timeout_secs: config.timeout_secs,
        context_limit: config.context_limit,
        response_format: config.response_format.as_ref().map(|f| f.to_string()),
        validate_tokens: config.validate_tokens,

        // Input sources (distinguish between text and file inputs)
        system_prompt_text: if config.system_prompt_file.is_none() {
            Some(config.system_prompt.clone())
        } else {
            None
        },
        system_prompt_file: config
            .system_prompt_file
            .as_ref()
            .map(|p| p.display().to_string()),
        user_prompt_text: if config.user_prompt_file.is_none() && config.pdf_input.is_none() {
            Some(user_prompt.to_string())
        } else {
            None
        },
        user_prompt_file: config
            .user_prompt_file
            .as_ref()
            .map(|p| p.display().to_string()),
        pdf_input: config.pdf_input.as_ref().map(|p| p.display().to_string()),

        // Guardrails
        input_guardrails_enabled: config.input_guardrails.as_ref().map(|_| true),
        output_guardrails_enabled: if output_guardrails_enabled {
            Some(true)
        } else {
            None
        },
    }
}

/// Simplified evaluation function (no output guardrails)
pub async fn evaluate(config: EvaluationConfig) -> Result<CliOutput, CliError> {
    evaluate_internal(config, None).await
}

/// Comprehensive evaluation with full guardrails pipeline
pub async fn evaluate_with_guardrails(
    config: EvaluationConfig,
    output_guardrails: Option<OutputGuardrailProviderConfig>,
) -> Result<CliOutput, CliError> {
    evaluate_internal(config, output_guardrails).await
}

/// Internal evaluation implementation (shared by evaluate and evaluate_with_guardrails)
async fn evaluate_internal(
    config: EvaluationConfig,
    output_guardrails: Option<OutputGuardrailProviderConfig>,
) -> Result<CliOutput, CliError> {
    let start_time = Instant::now();

    // 1. PDF extraction (if PDF input provided)
    let user_prompt = if let Some(pdf_path) = &config.pdf_input {
        // Validate PDF file size before extraction (security protection)
        let file_metadata = std::fs::metadata(pdf_path).map_err(|e| {
            CliError::FileNotFound(format!(
                "Failed to read PDF file metadata '{}': {e}",
                pdf_path.display()
            ))
        })?;

        let file_size = file_metadata.len();
        if file_size > constants::pdf_limits::MAX_PDF_SIZE_BYTES {
            let metadata = create_metadata(
                &config,
                "", // No user prompt yet
                0,  // No tokens estimated yet
                start_time.elapsed().as_millis() as u64,
                output_guardrails.is_some(),
            );
            return Ok(CliOutput::error(
                "FILE_TOO_LARGE".to_string(),
                format!(
                    "PDF file size ({} bytes, {:.2} MB) exceeds maximum allowed size ({} bytes, {:.2} MB). \
                    This limit prevents resource exhaustion from large files.",
                    file_size,
                    file_size as f64 / 1_048_576.0,
                    constants::pdf_limits::MAX_PDF_SIZE_BYTES,
                    constants::pdf_limits::MAX_PDF_SIZE_BYTES as f64 / 1_048_576.0
                ),
                metadata,
            ));
        }

        log::debug!(
            "PDF file size: {} bytes ({:.2} MB)",
            file_size,
            file_size as f64 / 1_048_576.0
        );

        let content = extract_text_from_pdf(pdf_path).await?;
        let char_count = content.text.len();
        let word_count = content.text.split_whitespace().count();
        log::debug!(
            "Extracted {} characters ({} words) from PDF using {} (format: {:?})",
            char_count,
            word_count,
            content.extractor_used,
            content.format
        );
        if let Some(size) = content.file_size_bytes {
            log::debug!(
                "PDF file size: {} bytes ({:.2} KB)",
                size,
                size as f64 / 1024.0
            );
        }
        for warning in &content.warnings {
            log::debug!("PDF extraction: {warning}");
        }
        content.text
    } else {
        config.user_prompt.clone()
    };

    // 2. Input guardrails (AFTER PDF extraction)
    // NOTE: Only validate user-provided content, NOT system prompt
    // System prompts are trusted, developer-controlled content
    if let Some(guardrail_config) = &config.input_guardrails {
        log::info!("Running input guardrails validation");
        let guardrail = create_guardrail_provider(guardrail_config)?;
        // SECURITY: Only validate user input, not system prompt
        let validation = guardrail.validate_input(&user_prompt).await?;

        if !validation.passed {
            log::error!("Input guardrails validation FAILED");
            let metadata = create_metadata(
                &config,
                &user_prompt,
                0,
                start_time.elapsed().as_millis() as u64,
                output_guardrails.is_some(),
            );

            let error_msg = validation
                .violations
                .iter()
                .map(|v| format!("{}: {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");

            log::error!("Violations: {error_msg}");

            return Ok(CliOutput::error(
                "INPUT_VALIDATION_FAILED".to_string(),
                error_msg,
                metadata,
            ));
        }

        log::info!("Input guardrails validation PASSED");

        // Log warnings
        for warning in validation.warnings {
            log::warn!("{}: {}", warning.rule, warning.message);
        }
    }

    // 3. Token validation (if enabled)
    let tokens_estimated = if config.validate_tokens {
        // Use model-specific token estimation if model is recognized
        // For estimation purposes only, use DEFAULT_MAX_TOKENS if not specified
        let estimator = TokenEstimator::new_for_model(
            &config.system_prompt,
            &user_prompt,
            config
                .max_tokens
                .unwrap_or(constants::llm_defaults::DEFAULT_MAX_TOKENS),
            &config.model,
        );
        let required = estimator.total_tokens_required();

        // Log token breakdown
        let breakdown = estimator.breakdown();
        log::debug!(
            "Token estimate: system={}, user={}, response_buffer={}, total={}",
            breakdown.system_tokens,
            breakdown.user_tokens,
            breakdown.response_buffer,
            breakdown.total_required
        );

        if let Some(limit) = config.context_limit {
            if required > limit {
                let metadata = create_metadata(
                    &config,
                    &user_prompt,
                    required,
                    start_time.elapsed().as_millis() as u64,
                    output_guardrails.is_some(),
                );
                return Ok(CliOutput::error(
                    "CONTEXT_LIMIT_EXCEEDED".to_string(),
                    format!(
                        "Context requirement ({} tokens) exceeds model limit ({} tokens) by {} tokens",
                        required, limit, required - limit
                    ),
                    metadata,
                ));
            }
        }
        required
    } else {
        // Even when validation is disabled, use model-specific estimation for metadata
        // For estimation purposes only, use DEFAULT_MAX_TOKENS if not specified
        TokenEstimator::new_for_model(
            &config.system_prompt,
            &user_prompt,
            config
                .max_tokens
                .unwrap_or(constants::llm_defaults::DEFAULT_MAX_TOKENS),
            &config.model,
        )
        .total_tokens_required()
    };

    // 4. LLM invocation
    let client = LlmClient::new(config.api_url.clone(), config.provider);
    let response = client
        .invoke(InvokeParams {
            model: &config.model,
            system_prompt: &config.system_prompt,
            user_prompt: &user_prompt,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            seed: config.seed,
            api_key: config.api_key.as_deref(),
            timeout_secs: config.timeout_secs,
            response_format: config.response_format.as_ref(),
        })
        .await?;

    // 5. Output guardrails (if enabled)
    let output_guardrails_enabled = output_guardrails.is_some();
    if let Some(guardrail_config) = &output_guardrails {
        let guardrail = create_output_guardrail_provider(guardrail_config)?;
        let validation = guardrail.validate_output(&response).await?;

        if !validation.passed {
            let metadata = create_metadata(
                &config,
                &user_prompt,
                tokens_estimated,
                start_time.elapsed().as_millis() as u64,
                true, // output guardrails are enabled (we're in this block)
            );

            let error_msg = validation
                .violations
                .iter()
                .map(|v| format!("{}: {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");

            return Ok(CliOutput::error(
                "OUTPUT_VALIDATION_FAILED".to_string(),
                error_msg,
                metadata,
            ));
        }

        // Log quality score and warnings
        if let Some(score) = validation.quality_score {
            log::info!("Response quality score: {score:.1}/10");
        }
        for warning in validation.warnings {
            let rule = &warning.rule;
            let message = &warning.message;
            log::warn!("{rule}: {message}");
        }
    }

    // 6. Create output
    let metadata = create_metadata(
        &config,
        &user_prompt,
        tokens_estimated,
        start_time.elapsed().as_millis() as u64,
        output_guardrails_enabled,
    );

    Ok(CliOutput::success(
        response,
        metadata,
        config.response_format.as_ref(),
    ))
}
