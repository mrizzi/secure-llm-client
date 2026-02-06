//! Configuration builder for merging CLI args with config files
//!
//! Provides a clean separation between CLI argument parsing and configuration construction.
//! Follows the Builder pattern for testability and reusability.

use crate::{
    config::ConfigFileRequest, constants::llm_defaults, error::CliError, model_registry,
    schema_validator, EvaluationConfig, Provider, ResponseFormat,
};
use std::path::PathBuf;

/// Minimum values for validation
const MIN_TOKENS: u32 = 1;
const MIN_TIMEOUT: u64 = 1;
const MIN_CONTEXT_LIMIT: usize = 100;

/// Builder for constructing EvaluationConfig from CLI args and config files
///
/// Handles merging of CLI arguments (highest priority), config file values (medium priority),
/// and sensible defaults (lowest priority).
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    // Required fields (must be provided via CLI or config file)
    pub api_url: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,

    // Optional fields (CLI > config > default)
    pub provider: Option<Provider>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u64>,
    pub api_key: Option<String>,
    pub timeout_secs: Option<u64>,
    pub validate_tokens: Option<bool>,
    pub context_limit: Option<usize>,
    pub response_format: Option<ResponseFormat>,
    pub pdf_input: Option<PathBuf>,
    pub input_guardrails: Option<crate::GuardrailProviderConfig>,
    pub output_guardrails: Option<crate::GuardrailProviderConfig>,

    // Source tracking (for metadata reproducibility)
    pub system_prompt_file: Option<PathBuf>,
    pub user_prompt_file: Option<PathBuf>,
}

impl ConfigBuilder {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge values from a config file (lower priority than CLI args)
    pub fn merge_file_config(mut self, file_config: &ConfigFileRequest) -> Self {
        // Only set if not already set (CLI args take precedence)
        if self.api_url.is_none() {
            self.api_url = Some(file_config.api_url.clone());
        }
        if self.model.is_none() {
            self.model = Some(file_config.model.clone());
        }
        if self.provider.is_none() {
            if let Some(provider_str) = &file_config.provider {
                // Parse provider string ("ollama", "openai")
                match provider_str.to_lowercase().as_str() {
                    "ollama" => self.provider = Some(Provider::Ollama),
                    "openai" => self.provider = Some(Provider::OpenAI),
                    _ => log::warn!("Unknown provider '{provider_str}' in config file. Valid values: 'ollama', 'openai'"),
                }
            }
        }
        if self.system_prompt.is_none() {
            self.system_prompt = file_config.system_prompt.clone();
        }
        // Track if system prompt came from file (for metadata)
        if self.system_prompt_file.is_none() {
            if let Some(file_path) = &file_config.system_prompt_file {
                self.system_prompt_file = Some(PathBuf::from(file_path));
            }
        }
        if self.user_prompt.is_none() {
            self.user_prompt = file_config.user_prompt.clone();
        }
        // Track if user prompt came from file (for metadata)
        if self.user_prompt_file.is_none() {
            if let Some(file_path) = &file_config.user_prompt_file {
                self.user_prompt_file = Some(PathBuf::from(file_path));
            }
        }
        if self.pdf_input.is_none() {
            if let Some(pdf_path) = &file_config.pdf_file {
                self.pdf_input = Some(PathBuf::from(pdf_path));
            }
        }
        if self.temperature.is_none() {
            self.temperature = Some(file_config.temperature);
        }
        if self.max_tokens.is_none() {
            self.max_tokens = file_config.max_tokens;
        }
        if self.seed.is_none() {
            self.seed = file_config.seed;
        }
        if self.timeout_secs.is_none() {
            self.timeout_secs = Some(file_config.timeout_secs);
        }
        if self.validate_tokens.is_none() {
            self.validate_tokens = Some(file_config.validate_tokens);
        }
        if self.context_limit.is_none() {
            self.context_limit = file_config.context_limit;
        }
        if self.api_key.is_none() {
            self.api_key = file_config.api_key.clone();
        }
        if self.input_guardrails.is_none() {
            self.input_guardrails = file_config.guardrails.as_ref().and_then(|g| {
                // Prefer explicit input field, fallback to flattened provider field
                g.input.clone().or_else(|| g.provider.clone())
            });
        }
        if self.output_guardrails.is_none() {
            self.output_guardrails = file_config.guardrails.as_ref().and_then(|g| {
                // Prefer explicit output field, fallback to flattened provider field
                g.output.clone().or_else(|| g.provider.clone())
            });
        }

        // Handle response_format from config file (only if not set via CLI)
        if self.response_format.is_none() {
            if let Some(format_str) = &file_config.response_format {
                match format_str.as_str() {
                    "text" => {
                        self.response_format = Some(ResponseFormat::text());
                    }
                    "json-object" | "json_object" => {
                        self.response_format = Some(ResponseFormat::json());
                    }
                    "json-schema" | "json_schema" => {
                        // Load schema from file
                        if let Some(schema_path) = &file_config.response_format_schema {
                            let path = PathBuf::from(schema_path);
                            let strict = file_config.response_format_schema_strict.unwrap_or(true);

                            // Use load_json_schema to load and validate
                            match load_json_schema(&path, strict) {
                                Ok(response_format) => {
                                    self.response_format = Some(response_format);
                                }
                                Err(e) => {
                                    log::warn!(
                                        "Failed to load JSON schema from config file '{schema_path}': {e}"
                                    );
                                }
                            }
                        } else {
                            log::warn!(
                                "Config file specifies response_format='json-schema' but \
                                response_format_schema is not set. Ignoring response_format."
                            );
                        }
                    }
                    _ => {
                        log::warn!(
                            "Unknown response_format '{format_str}' in config file. \
                            Valid values: 'text', 'json-object', 'json-schema'"
                        );
                    }
                }
            }
        }

        self
    }

    /// Set API URL (highest priority - typically from CLI)
    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        self.api_url = Some(api_url.into());
        self
    }

    /// Set model name (highest priority - typically from CLI)
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set system prompt (highest priority - typically from CLI)
    pub fn system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set user prompt (highest priority - typically from CLI)
    pub fn user_prompt(mut self, user_prompt: impl Into<String>) -> Self {
        self.user_prompt = Some(user_prompt.into());
        self
    }

    /// Set system prompt file path (for metadata tracking)
    pub fn system_prompt_file(mut self, file_path: PathBuf) -> Self {
        self.system_prompt_file = Some(file_path);
        self
    }

    /// Set user prompt file path (for metadata tracking)
    pub fn user_prompt_file(mut self, file_path: PathBuf) -> Self {
        self.user_prompt_file = Some(file_path);
        self
    }

    /// Set provider
    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set temperature (validates range)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set random seed for reproducible sampling
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set timeout in seconds
    pub fn timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Set whether to validate tokens
    pub fn validate_tokens(mut self, validate_tokens: bool) -> Self {
        self.validate_tokens = Some(validate_tokens);
        self
    }

    /// Set context limit
    pub fn context_limit(mut self, context_limit: usize) -> Self {
        self.context_limit = Some(context_limit);
        self
    }

    /// Set response format
    pub fn response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Set PDF input path (mutually exclusive with user_prompt text)
    pub fn pdf_input(mut self, pdf_path: PathBuf) -> Self {
        self.pdf_input = Some(pdf_path);
        self
    }

    /// Set input guardrails configuration
    pub fn input_guardrails(mut self, guardrails: crate::GuardrailProviderConfig) -> Self {
        self.input_guardrails = Some(guardrails);
        self
    }

    /// Set output guardrails configuration
    pub fn output_guardrails(mut self, guardrails: crate::GuardrailProviderConfig) -> Self {
        self.output_guardrails = Some(guardrails);
        self
    }

    /// Build the final EvaluationConfig, applying defaults and validation
    ///
    /// # Errors
    ///
    /// Returns `CliError::InvalidArguments` if:
    /// - Required fields are missing (api_url, model, system_prompt, user_prompt)
    /// - Values are out of valid ranges
    pub fn build(self) -> Result<EvaluationConfig, CliError> {
        // Validate required fields
        let api_url = self.api_url.ok_or_else(|| {
            CliError::InvalidArguments(
                "API URL must be provided via --api-url or in config file (--config-file)"
                    .to_string(),
            )
        })?;

        let model = self.model.ok_or_else(|| {
            CliError::InvalidArguments(
                "Model must be provided via --model or in config file (--config-file)".to_string(),
            )
        })?;

        let system_prompt = self.system_prompt.ok_or_else(|| {
            CliError::InvalidArguments(
                "System prompt must be provided via --system-file/--system-text or in config file (--config-file)"
                    .to_string(),
            )
        })?;

        // User prompt required UNLESS pdf_input is provided
        let user_prompt = if self.pdf_input.is_some() {
            String::new() // PDF will be extracted later
        } else {
            self.user_prompt.ok_or_else(|| {
                CliError::InvalidArguments(
                    "User prompt must be provided via --user-file/--user-text/--pdf-file or in config file (--config-file)"
                        .to_string(),
                )
            })?
        };

        // Apply defaults and validate optional fields
        let temperature = self
            .temperature
            .unwrap_or(llm_defaults::DEFAULT_TEMPERATURE);
        if !(llm_defaults::MIN_TEMPERATURE..=llm_defaults::MAX_TEMPERATURE).contains(&temperature) {
            return Err(CliError::InvalidArguments(format!(
                "temperature must be between {} and {}, got {temperature}",
                llm_defaults::MIN_TEMPERATURE,
                llm_defaults::MAX_TEMPERATURE
            )));
        }

        // Validate max_tokens if provided (None = use model's maximum)
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens < MIN_TOKENS {
                return Err(CliError::InvalidArguments(format!(
                    "max_tokens must be >= {MIN_TOKENS}, got {max_tokens}"
                )));
            }
        }

        let timeout_secs = self
            .timeout_secs
            .unwrap_or(llm_defaults::DEFAULT_TIMEOUT_SECS);
        if timeout_secs < MIN_TIMEOUT {
            return Err(CliError::InvalidArguments(format!(
                "timeout_secs must be >= {MIN_TIMEOUT}, got {timeout_secs}"
            )));
        }

        // Auto-detect context limit from model registry if not explicitly set
        // Validate user-provided limit first (early return on error)
        if let Some(limit) = self.context_limit {
            if limit < MIN_CONTEXT_LIMIT {
                return Err(CliError::InvalidArguments(format!(
                    "context_limit must be >= {MIN_CONTEXT_LIMIT}, got {limit}"
                )));
            }
        }

        // Use user-provided limit or auto-detect from registry
        let context_limit = self.context_limit.or_else(|| {
            model_registry::lookup_model(&model).map(|model_info| {
                log::info!(
                    "Auto-detected context window for {}: {} tokens",
                    model,
                    model_info.context_window
                );
                model_info.context_window
            })
        });

        // Log if not auto-detected
        if context_limit.is_none() && self.context_limit.is_none() {
            log::debug!(
                "Model '{model}' not in registry - context limit not auto-detected. \
                Use --context-limit to set manually."
            );
        }

        let validate_tokens = self.validate_tokens.unwrap_or(false);

        Ok(EvaluationConfig {
            api_url,
            model,
            system_prompt,
            user_prompt,
            provider: self.provider,
            temperature,
            max_tokens: self.max_tokens, // None = use model's maximum
            seed: self.seed,
            api_key: self.api_key,
            timeout_secs,
            validate_tokens,
            context_limit, // Use auto-detected or user-provided value
            response_format: self.response_format,
            pdf_input: self.pdf_input,
            input_guardrails: self.input_guardrails,
            output_guardrails: self.output_guardrails,
            system_prompt_file: self.system_prompt_file,
            user_prompt_file: self.user_prompt_file,
        })
    }
}

/// Helper to load JSON schema from file and create ResponseFormat
///
/// Performs validation and extracts schema name from filename.
pub fn load_json_schema(schema_path: &PathBuf, strict: bool) -> Result<ResponseFormat, CliError> {
    // Read and parse the JSON schema file
    let schema_content = std::fs::read_to_string(schema_path).map_err(|e| {
        CliError::FileNotFound(format!(
            "Failed to read schema file '{}': {e}\n\
            Please verify:\n\
            - File path is correct\n\
            - File exists and is readable\n\
            - You have permission to access the file",
            schema_path.display()
        ))
    })?;

    let schema: serde_json::Value = serde_json::from_str(&schema_content).map_err(|e| {
        CliError::InvalidArguments(format!(
            "Invalid JSON in schema file '{}': {e}\n\
            Please verify:\n\
            - File contains valid JSON (try 'jq . {}' or 'python -m json.tool {}')\n\
            - Schema follows JSON Schema Draft 7+ specification\n\
            - All quotes and braces are properly matched",
            schema_path.display(),
            schema_path.display(),
            schema_path.display()
        ))
    })?;

    // Perform basic sanity checks first (quick feedback)
    schema_validator::basic_schema_sanity_check(&schema).map_err(|e| {
        CliError::InvalidArguments(format!(
            "Schema file '{}' failed basic validation: {e}\n\
            Please check your schema syntax.",
            schema_path.display()
        ))
    })?;

    // Validate against JSON Schema Draft 7 metaschema (comprehensive)
    schema_validator::validate_json_schema(&schema).map_err(|e| {
        CliError::InvalidArguments(format!(
            "Schema file '{}' validation failed:\n{e}\n\n\
            Your schema does not conform to JSON Schema Draft 7 specification.\n\
            References:\n\
            - JSON Schema specification: https://json-schema.org/draft-07/json-schema-release-notes.html\n\
            - Schema validator: https://www.jsonschemavalidator.net/\n\
            - Examples: See examples/schemas/ directory",
            schema_path.display()
        ))
    })?;

    // Extract schema name from filename (without extension)
    let schema_name = schema_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| {
            log::warn!(
                "Could not extract valid schema name from '{}', using 'schema'. \
                Consider using a descriptive filename like 'person_schema.json'.",
                schema_path.display()
            );
            "schema".to_string()
        });

    Ok(ResponseFormat::json_schema(schema_name, schema, strict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_required_fields() {
        let result = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            .user_prompt("Say hello")
            .build();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.api_url, "http://localhost:11434");
        assert_eq!(config.model, "llama3");
        assert_eq!(config.system_prompt, "You are helpful");
        assert_eq!(config.user_prompt, "Say hello");
    }

    #[test]
    fn test_builder_missing_required_field() {
        let result = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            // Missing user_prompt
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("User prompt"));
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            .user_prompt("Say hello")
            .build()
            .unwrap();

        assert_eq!(config.temperature, llm_defaults::DEFAULT_TEMPERATURE);
        assert_eq!(config.max_tokens, None); // No default, use model's maximum
        assert_eq!(config.timeout_secs, llm_defaults::DEFAULT_TIMEOUT_SECS);
        assert!(!config.validate_tokens);
    }

    #[test]
    fn test_builder_with_overrides() {
        let config = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            .user_prompt("Say hello")
            .temperature(0.9)
            .max_tokens(1000)
            .timeout_secs(60)
            .validate_tokens(true)
            .build()
            .unwrap();

        assert_eq!(config.temperature, 0.9);
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.timeout_secs, 60);
        assert!(config.validate_tokens);
    }

    #[test]
    fn test_builder_temperature_validation() {
        let result = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            .user_prompt("Say hello")
            .temperature(3.0) // Invalid: > 2.0
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("temperature"));
    }

    #[test]
    fn test_builder_pdf_input_without_user_prompt() {
        let config = ConfigBuilder::new()
            .api_url("http://localhost:11434")
            .model("llama3")
            .system_prompt("You are helpful")
            .pdf_input(PathBuf::from("/tmp/test.pdf"))
            .build()
            .unwrap();

        assert_eq!(config.user_prompt, "");
        assert_eq!(config.pdf_input, Some(PathBuf::from("/tmp/test.pdf")));
    }
}
