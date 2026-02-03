use crate::{error::CliError, guardrails::GuardrailConfig};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Configuration file request format (supports both JSON and TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileRequest {
    /// LLM API endpoint URL
    pub api_url: String,

    /// Model name/identifier
    pub model: String,

    /// Provider type (optional: "ollama" or "openai", auto-detected if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// API key for authentication (optional, conflicts with api_key_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Environment variable name containing the API key (optional, conflicts with api_key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_name: Option<String>,

    /// System prompt inline text (conflicts with system_prompt_file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// System prompt from file path (conflicts with system_prompt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_file: Option<String>,

    /// User prompt inline text (conflicts with user_prompt_file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt: Option<String>,

    /// User prompt from file path (conflicts with user_prompt and pdf_file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt_file: Option<String>,

    /// PDF file to extract text from (conflicts with user_prompt and user_prompt_file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_file: Option<String>,

    /// Sampling temperature (optional, default: 0.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum response tokens (optional, default: None = model's maximum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Random seed for reproducible sampling (optional, default: None = non-deterministic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    /// Request timeout in seconds (optional, default: 300)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Enable token validation (optional, default: false)
    #[serde(default)]
    pub validate_tokens: bool,

    /// Model's context window limit (required if validate_tokens is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_limit: Option<usize>,

    /// Response format (optional: "text", "json-object", or "json-schema")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Path to JSON Schema file (required if response_format is "json-schema")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format_schema: Option<String>,

    /// Use strict schema validation (optional, default: true for json-schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format_schema_strict: Option<bool>,

    /// Guardrail configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<GuardrailConfig>,
}

fn default_temperature() -> f32 {
    0.0
}

fn default_timeout() -> u64 {
    300
}

impl ConfigFileRequest {
    /// Validate and resolve file paths to content
    /// This ensures that if `*_file` fields are used, their content is loaded
    /// and conflicts between inline text and file paths are detected
    pub fn resolve_file_paths(&mut self) -> Result<(), CliError> {
        // Validate API key configuration (can't specify both api_key and api_key_name)
        if self.api_key.is_some() && self.api_key_name.is_some() {
            return Err(CliError::InvalidArguments(
                "Config file cannot specify both 'api_key' and 'api_key_name'".to_string(),
            ));
        }

        // Validate and resolve system prompt
        match (&self.system_prompt, &self.system_prompt_file) {
            (Some(_), Some(_)) => {
                return Err(CliError::InvalidArguments(
                    "Config file cannot specify both 'system_prompt' and 'system_prompt_file'"
                        .to_string(),
                ));
            }
            (None, Some(file_path)) => {
                // Load file content
                let content = fs::read_to_string(file_path).map_err(|e| {
                    CliError::FileNotFound(format!(
                        "Failed to read system prompt file '{file_path}': {e}"
                    ))
                })?;
                self.system_prompt = Some(content);
                // Keep file path for metadata tracking (don't clear it)
            }
            (None, None) => {
                return Err(CliError::InvalidArguments(
                    "Config file must specify either 'system_prompt' or 'system_prompt_file'"
                        .to_string(),
                ));
            }
            (Some(_), None) => {
                // Inline text provided, all good
            }
        }

        // Validate and resolve user prompt (optional, conflicts with pdf_file)
        let user_prompt_count = [
            self.user_prompt.as_ref().map(|_| 1).unwrap_or(0),
            self.user_prompt_file.as_ref().map(|_| 1).unwrap_or(0),
            self.pdf_file.as_ref().map(|_| 1).unwrap_or(0),
        ]
        .iter()
        .sum::<usize>();

        if user_prompt_count > 1 {
            return Err(CliError::InvalidArguments(
                "Config file cannot specify more than one of: 'user_prompt', 'user_prompt_file', 'pdf_file'".to_string()
            ));
        }

        // Resolve user_prompt_file if provided
        if let Some(file_path) = &self.user_prompt_file {
            let content = fs::read_to_string(file_path).map_err(|e| {
                CliError::FileNotFound(format!(
                    "Failed to read user prompt file '{file_path}': {e}"
                ))
            })?;
            self.user_prompt = Some(content);
            // Keep file path for metadata tracking (don't clear it)
        }

        // Note: pdf_file validation happens in ConfigBuilder, not here
        // (PDF extraction is done later, we just need the path)

        Ok(())
    }
}

/// Load config from file (auto-detects JSON vs TOML from extension)
pub fn load_config_file<P: AsRef<Path>>(path: P) -> Result<ConfigFileRequest, CliError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|e| {
        CliError::FileNotFound(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Auto-detect format from extension and parse
    let mut config: ConfigFileRequest = if path.extension().and_then(|s| s.to_str()) == Some("toml")
    {
        // Parse TOML
        toml::from_str(&contents)
            .map_err(|e| CliError::InvalidArguments(format!("Failed to parse TOML config: {e}")))?
    } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
        // Parse JSON
        serde_json::from_str(&contents)
            .map_err(|e| CliError::InvalidArguments(format!("Failed to parse JSON config: {e}")))?
    } else {
        return Err(CliError::InvalidArguments(
            "Config file must have .json or .toml extension".to_string(),
        ));
    };

    // Resolve file paths to actual content
    config.resolve_file_paths()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_json_config() {
        let json = r#"{
            "api_url": "http://localhost:11434/api/generate",
            "model": "llama3",
            "system_prompt": "You are helpful.",
            "user_prompt": "Hello",
            "temperature": 0.5,
            "max_tokens": 1000
        }"#;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("json");
        std::fs::write(&path, json).unwrap();

        let config = load_config_file(&path).unwrap();
        assert_eq!(config.api_url, "http://localhost:11434/api/generate");
        assert_eq!(config.model, "llama3");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, Some(1000));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_toml_config() {
        let toml = r#"
            api_url = "http://localhost:11434/api/generate"
            model = "llama3"
            system_prompt = "You are helpful."
            user_prompt = "Hello"
            temperature = 0.5
            max_tokens = 1000
        "#;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("toml");
        std::fs::write(&path, toml).unwrap();

        let config = load_config_file(&path).unwrap();
        assert_eq!(config.api_url, "http://localhost:11434/api/generate");
        assert_eq!(config.model, "llama3");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, Some(1000));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_invalid_extension() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("txt");
        std::fs::write(&path, "invalid").unwrap();

        let result = load_config_file(&path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have .json or .toml extension"));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_default_values() {
        let json = r#"{
            "api_url": "http://localhost:11434/api/generate",
            "model": "llama3",
            "system_prompt": "You are helpful.",
            "user_prompt": "Hello"
        }"#;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("json");
        std::fs::write(&path, json).unwrap();

        let config = load_config_file(&path).unwrap();
        assert_eq!(config.temperature, 0.0); // default
        assert_eq!(config.max_tokens, None); // default (use model's maximum)
        assert_eq!(config.timeout_secs, 300); // default
        assert!(!config.validate_tokens); // default

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_optional_user_prompt() {
        let json = r#"{
            "api_url": "http://localhost:11434/api/generate",
            "model": "llama3",
            "system_prompt": "You are helpful."
        }"#;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("json");
        std::fs::write(&path, json).unwrap();

        let config = load_config_file(&path).unwrap();
        assert_eq!(config.api_url, "http://localhost:11434/api/generate");
        assert_eq!(config.model, "llama3");
        assert_eq!(config.system_prompt, Some("You are helpful.".to_string()));
        assert!(config.user_prompt.is_none()); // user_prompt is optional

        std::fs::remove_file(&path).ok();
    }
}
