mod cli;

use clap::{CommandFactory, Parser};
use cli::{
    configure_guardrails, load_prompt, validate_byte_size, validate_context_limit,
    validate_file_exists, validate_positive_u32, validate_positive_u64, validate_positive_usize,
    validate_temperature, write_output,
};
use figment::{
    providers::{Format, Json, Serialized, Toml},
    Figment,
};
use fortified_llm_client::{
    config_builder::{self, ConfigBuilder},
    evaluate, CliError, CliOutput, Metadata, Provider,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process};

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "fortified-llm-client")]
#[command(about = "LLM client fortified by multi-layered security guardrails and multi-provider support", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[serde(default)]
struct Args {
    /// Config file (JSON or TOML) with default evaluation parameters
    /// Note: any CLI argument will override the corresponding config file value
    #[arg(long, short = 'c', value_parser = validate_file_exists)]
    #[serde(skip)]
    config_file: Option<PathBuf>,

    /// LLM API endpoint URL
    #[arg(long, short = 'a')]
    #[serde(skip_serializing_if = "Option::is_none")]
    api_url: Option<String>,

    /// Model name/identifier
    #[arg(long, short = 'm')]
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,

    /// Force specific provider format (overrides auto-detection)
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderArg>,

    /// System prompt from file
    #[arg(long, short = 's', conflicts_with = "system_text", value_parser = validate_file_exists)]
    #[serde(skip_serializing_if = "Option::is_none")]
    system_file: Option<PathBuf>,

    /// System prompt as text (no short form, use --system-text)
    #[arg(long, conflicts_with = "system_file")]
    #[serde(skip_serializing_if = "Option::is_none")]
    system_text: Option<String>,

    /// User prompt from file
    #[arg(long, short = 'u', conflicts_with_all = ["user_text", "pdf_file"], value_parser = validate_file_exists)]
    #[serde(skip_serializing_if = "Option::is_none")]
    user_file: Option<PathBuf>,

    /// User prompt as text (no short form, use --user-text)
    #[arg(long, conflicts_with_all = ["user_file", "pdf_file"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    user_text: Option<String>,

    /// PDF file to extract text from (replaces user prompt)
    #[arg(long, short = 'p', conflicts_with_all = ["user_file", "user_text"], value_parser = validate_file_exists)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf_file: Option<PathBuf>,

    /// Sampling temperature
    #[arg(long, short = 't', value_parser = validate_temperature)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    /// Maximum response tokens
    #[arg(long, value_parser = validate_positive_u32)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,

    /// Random seed for reproducible sampling
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,

    /// Enable token validation
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    validate_tokens: Option<bool>,

    /// Model's context window limit
    #[arg(long, value_parser = validate_context_limit)]
    #[serde(skip_serializing_if = "Option::is_none")]
    context_limit: Option<usize>,

    /// Response format (json-object, json-schema, or text, OpenAI-compatible only)
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormatArg>,

    /// JSON Schema file path (required when --response-format=json-schema)
    #[arg(long, value_parser = validate_file_exists)]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format_schema: Option<PathBuf>,

    /// Use strict mode for JSON schema validation (default: true)
    #[arg(long, default_value_t = true)]
    #[serde(default = "default_response_format_schema_strict")]
    response_format_schema_strict: bool,

    /// API key for authentication (direct value)
    #[arg(long, conflicts_with = "api_key_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,

    /// Environment variable name containing the API key
    #[arg(long, conflicts_with = "api_key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key_name: Option<String>,

    /// Request timeout in seconds (must be > 0)
    #[arg(long = "timeout", value_parser = validate_positive_u64)]
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_secs: Option<u64>,

    /// Enable verbose logging (DEBUG level)
    #[arg(long, short = 'v', conflicts_with = "quiet")]
    #[serde(skip, default)]
    verbose: bool,

    /// Suppress all logging output
    #[arg(long, short = 'q', conflicts_with = "verbose")]
    #[serde(skip, default)]
    quiet: bool,

    /// Write output to file instead of stdout
    /// Uses atomic writes (temp file + rename) and creates parent directories
    #[arg(long, short = 'o')]
    #[serde(skip)]
    output: Option<PathBuf>,

    // Input Validation (regex-based pattern matching via CLI)
    // Note: For LLM-based guardrails (Llama Guard, GPT-OSS Safeguard, hybrid strategies),
    //       use config files with the [guardrails] section
    /// Enable regex-based input validation using default patterns (PII, prompt injection, etc.)
    /// For custom patterns or fine-grained control, use config files
    #[arg(long)]
    #[serde(skip, default)]
    enable_input_validation: bool,

    /// Maximum input length (default: 1MB when validation enabled)
    /// Accepts human-readable sizes: 100MB, 1.5GB, 500KB, or plain bytes
    #[arg(long, requires = "enable_input_validation", value_parser = validate_byte_size)]
    #[serde(skip)]
    max_input_length: Option<usize>,

    /// Maximum estimated input tokens (default: 200K when validation enabled)
    #[arg(long, requires = "enable_input_validation", value_parser = validate_positive_usize)]
    #[serde(skip)]
    max_input_tokens: Option<usize>,
}

fn default_response_format_schema_strict() -> bool {
    true
}

impl Default for Args {
    fn default() -> Self {
        Self {
            config_file: None,
            api_url: None,
            model: None,
            provider: None,
            system_file: None,
            system_text: None,
            user_file: None,
            user_text: None,
            pdf_file: None,
            temperature: None,
            max_tokens: None,
            seed: None,
            validate_tokens: None,
            context_limit: None,
            response_format: None,
            response_format_schema: None,
            response_format_schema_strict: true,
            api_key: None,
            api_key_name: None,
            timeout_secs: None,
            verbose: false,
            quiet: false,
            output: None,
            enable_input_validation: false,
            max_input_length: None,
            max_input_tokens: None,
        }
    }
}

/// Merge config file and CLI args using figment
/// Priority: CLI args > Config file
fn merge_config(args: &Args) -> Result<Args, CliError> {
    // If no config file specified, just return CLI args
    let Some(config_path) = &args.config_file else {
        return Ok(args.clone());
    };

    // Merge: config file < CLI args (CLI has highest priority)
    // Detect format by extension (matches config.rs pattern)
    let file_provider = match config_path.extension().and_then(|s| s.to_str()) {
        Some("json") => Figment::from(Json::file(config_path)),
        Some("toml") => Figment::from(Toml::file(config_path)),
        _ => {
            return Err(CliError::InvalidArguments(
                "Config file must have .json or .toml extension".to_string(),
            ));
        }
    };

    let merged: Args = file_provider
        .merge(Serialized::defaults(args))
        .extract()
        .map_err(|e| CliError::InvalidArguments(format!("Failed to merge config: {e}")))?;

    // Restore CLI-only fields that shouldn't be in config files
    //
    // ⚠️ CRITICAL CHECKLIST: When adding new #[serde(skip)] fields to Args,
    // you MUST add them to this restoration list below.
    //
    // Current CLI-only fields (7 total):
    // 1. config_file - Path to config file itself
    // 2. verbose - CLI logging flag
    // 3. quiet - CLI logging flag
    // 4. output - Output file path
    // 5. enable_input_validation - Input guardrails flag
    // 6. max_input_length - Input size limit
    // 7. max_input_tokens - Input token limit
    Ok(Args {
        config_file: args.config_file.clone(),
        verbose: args.verbose,
        quiet: args.quiet,
        output: args.output.clone(),
        enable_input_validation: args.enable_input_validation,
        max_input_length: args.max_input_length,
        max_input_tokens: args.max_input_tokens,
        ..merged
    })
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProviderArg {
    Ollama,
    #[value(name = "openai")]
    #[serde(rename = "openai")]
    OpenAI,
}

impl From<ProviderArg> for Provider {
    fn from(arg: ProviderArg) -> Self {
        match arg {
            ProviderArg::Ollama => Provider::Ollama,
            ProviderArg::OpenAI => Provider::OpenAI,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ResponseFormatArg {
    #[value(name = "json-object")]
    JsonObject,
    #[value(name = "json-schema")]
    JsonSchema,
    Text,
}

/// Configure input guardrails from CLI args or config file

#[tokio::main]
async fn main() {
    // Load .env file if it exists (silently ignore if not found)
    dotenvy::dotenv().ok();

    // If no arguments provided, print help and exit
    if std::env::args().len() == 1 {
        let _ = Args::command().print_help(); // Ignore broken pipe errors
        println!(); // Add newline after help
        process::exit(0);
    }

    let args = Args::parse();

    // Initialize logger with appropriate level
    // quiet: no logs, verbose: DEBUG+, default: INFO+
    let log_level = if args.quiet {
        log::LevelFilter::Off // Suppress all logs
    } else if args.verbose {
        log::LevelFilter::Debug // Show DEBUG, INFO, WARN, ERROR
    } else {
        log::LevelFilter::Info // Show INFO, WARN, ERROR (default)
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                record.level(),
                record.args()
            )
        })
        .init();

    // Save output path before consuming args
    let output_path = args.output.clone();

    // Run the main logic and handle errors
    match run(args).await {
        Ok(output) => {
            // Write output (to file or stdout)
            if let Err(e) = write_output(&output, output_path.as_ref()) {
                eprintln!("Error writing output: {e}");
                process::exit(1);
            }
            process::exit(0);
        }
        Err(e) => {
            // Create minimal error metadata (no config available)
            let metadata = Metadata {
                model: "unknown".to_string(),
                tokens_estimated: 0,
                latency_ms: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                api_url: "unknown".to_string(),
                provider: None,
                temperature: 0.0,
                max_tokens: None,
                seed: None,
                timeout_secs: 0,
                context_limit: None,
                response_format: None,
                validate_tokens: false,
                system_prompt_text: None,
                system_prompt_file: None,
                user_prompt_text: None,
                user_prompt_file: None,
                pdf_input: None,
                input_guardrails_enabled: None,
                output_guardrails_enabled: None,
            };

            // Create error output
            let output = CliOutput::error(e.code().to_string(), e.to_string(), metadata);

            // Write error output (to file or stdout)
            if let Err(io_err) = write_output(&output, output_path.as_ref()) {
                eprintln!("Error writing output: {io_err}");
                process::exit(1);
            }

            // Exit with appropriate error code
            process::exit(e.exit_code());
        }
    }
}

async fn run(args: Args) -> Result<CliOutput, CliError> {
    // Merge config file and CLI args using figment (CLI args override config file)
    let merged_args = merge_config(&args)?;

    // Load config file for guardrails configuration
    //
    // WHY DUAL LOADING: Figment (above) handles scalar fields (api_url, model, etc.),
    // but guardrail configuration is complex nested structures not in Args struct.
    // The [guardrails] section in config files requires ConfigFileRequest parsing.
    //
    // FUTURE: Could unify by adding guardrails field to Args, but would require
    // making GuardrailConfig implement clap::Args (significant refactor).
    let file_config = if let Some(config_path) = &merged_args.config_file {
        Some(fortified_llm_client::load_config_file(config_path)?)
    } else {
        None
    };

    // Start building config from merged args
    let mut builder = ConfigBuilder::new();

    // Set values from merged args (config file + CLI args, with CLI taking priority)
    if let Some(ref api_url) = merged_args.api_url {
        builder = builder.api_url(api_url.clone());
    }
    if let Some(ref model) = merged_args.model {
        builder = builder.model(model.clone());
    }
    if let Some(provider) = merged_args.provider {
        builder = builder.provider(provider.into());
    }
    if let Some(temperature) = merged_args.temperature {
        builder = builder.temperature(temperature);
    }
    if let Some(max_tokens) = merged_args.max_tokens {
        builder = builder.max_tokens(max_tokens);
    }
    if let Some(seed) = merged_args.seed {
        builder = builder.seed(seed);
    }
    if let Some(timeout_secs) = merged_args.timeout_secs {
        builder = builder.timeout_secs(timeout_secs);
    }
    if let Some(validate_tokens) = merged_args.validate_tokens {
        builder = builder.validate_tokens(validate_tokens);
    }
    if let Some(context_limit) = merged_args.context_limit {
        builder = builder.context_limit(context_limit);
    }

    // Handle input validation and guardrails (merged args already include config file values)
    // Must be called before load_prompt to avoid partial move of merged_args
    if let Some(guardrail_config) = configure_guardrails(
        merged_args.enable_input_validation,
        merged_args.max_input_length,
        file_config.as_ref(),
    ) {
        builder = builder.input_guardrails(guardrail_config);
    }

    // Handle output guardrails from config file
    if let Some(guardrail_config) = file_config
        .as_ref()
        .and_then(|fc| fc.guardrails.as_ref())
        .and_then(|g| {
            // Prefer explicit output field, fallback to flattened provider field
            g.output.clone().or_else(|| g.provider.clone())
        })
    {
        builder = builder.output_guardrails(guardrail_config);
    }

    // Handle system prompt (file > text > config file)
    // Validation: Warn if config file has conflicting fields
    if merged_args.system_file.is_some() && merged_args.system_text.is_some() {
        log::warn!(
            "Config file contains both system_file and system_text. \
             Using system_file (priority: file > text)."
        );
    }

    if let Some(file_path) = merged_args.system_file {
        let prompt = load_prompt(Some(file_path.clone()), None)?;
        builder = builder.system_prompt(prompt).system_prompt_file(file_path);
    } else if let Some(text) = merged_args.system_text {
        builder = builder.system_prompt(text);
        // No file path set - metadata will show text content
    }

    // Handle user prompt (file > text > PDF > config file)
    // Validation: Warn if config file has multiple user prompt sources
    if [
        merged_args.user_file.as_ref().map(|_| 1).unwrap_or(0),
        merged_args.user_text.as_ref().map(|_| 1).unwrap_or(0),
        merged_args.pdf_file.as_ref().map(|_| 1).unwrap_or(0),
    ]
    .iter()
    .sum::<i32>()
        > 1
    {
        log::warn!(
            "Config file contains multiple user prompt sources. \
             Using first available (priority: user_file > user_text > pdf_file)."
        );
    }

    if let Some(file_path) = merged_args.user_file {
        let prompt = load_prompt(Some(file_path.clone()), None)?;
        builder = builder.user_prompt(prompt).user_prompt_file(file_path);
    } else if let Some(text) = merged_args.user_text {
        builder = builder.user_prompt(text);
        // No file path set - metadata will show text content
    } else if let Some(pdf_path) = merged_args.pdf_file {
        builder = builder.pdf_input(pdf_path);
    }

    // Handle API key (CLI direct > CLI env var > config file env var > config file direct)
    if let Some(ref key) = merged_args.api_key {
        builder = builder.api_key(key.clone());
    } else if let Some(ref env_var_name) = merged_args.api_key_name {
        // CLI --api-key-name takes priority over config file
        match std::env::var(env_var_name) {
            Ok(key) => {
                log::debug!("API key loaded from environment variable (CLI): {env_var_name}");
                builder = builder.api_key(key);
            }
            Err(_) => {
                return Err(CliError::InvalidArguments(format!(
                    "Environment variable '{env_var_name}' specified by --api-key-name does not exist"
                )));
            }
        }
    } else if let Some(file_cfg) = file_config.as_ref() {
        // Check config file for api_key_name (env var name)
        if let Some(ref env_var_name) = file_cfg.api_key_name {
            match std::env::var(env_var_name) {
                Ok(key) => {
                    log::debug!(
                        "API key loaded from environment variable (config file): {env_var_name}"
                    );
                    builder = builder.api_key(key);
                }
                Err(_) => {
                    return Err(CliError::InvalidArguments(format!(
                        "Environment variable '{env_var_name}' specified by config file 'api_key_name' does not exist"
                    )));
                }
            }
        }
        // Note: Direct api_key from config file is handled by merge_file_config() below
    }

    // Handle response format with schema validation
    if merged_args.response_format_schema.is_some()
        && merged_args.response_format != Some(ResponseFormatArg::JsonSchema)
    {
        return Err(CliError::InvalidArguments(format!(
            "--response-format-schema can only be used with --response-format=json-schema\n\
            You provided --response-format-schema but --response-format is {:?}\n\
            Either:\n\
            - Add --response-format json-schema to use structured output\n\
            - Remove --response-format-schema if you don't need schema validation",
            merged_args
                .response_format
                .map(|f| match f {
                    ResponseFormatArg::Text => "text",
                    ResponseFormatArg::JsonObject => "json-object",
                    ResponseFormatArg::JsonSchema => "json-schema",
                })
                .unwrap_or("not set")
        )));
    }

    match merged_args.response_format {
        Some(ResponseFormatArg::Text) => {
            builder = builder.response_format(fortified_llm_client::ResponseFormat::text());
        }
        Some(ResponseFormatArg::JsonObject) => {
            builder = builder.response_format(fortified_llm_client::ResponseFormat::json());
        }
        Some(ResponseFormatArg::JsonSchema) => {
            let schema_path = merged_args.response_format_schema.as_ref().ok_or_else(|| {
                CliError::InvalidArguments(
                    "--response-format-schema is required when using --response-format=json-schema\n\
                    Example: --response-format json-schema --response-format-schema path/to/schema.json"
                        .to_string(),
                )
            })?;

            let response_format = config_builder::load_json_schema(
                schema_path,
                merged_args.response_format_schema_strict,
            )?;
            builder = builder.response_format(response_format);
        }
        None => {}
    }

    // Merge config file values (lower priority than CLI args)
    if let Some(file_cfg) = file_config.as_ref() {
        builder = builder.merge_file_config(file_cfg);
    }

    // Build final config (applies defaults and validation)
    let config = builder.build()?;

    // Debug log: Input parameters (excluding API key for security)
    log::debug!("=== Evaluation Parameters ===");
    log::debug!("API URL: {}", config.api_url);
    log::debug!("Model: {}", config.model);
    log::debug!("Provider: {:?}", config.provider);
    log::debug!("Temperature: {}", config.temperature);
    log::debug!(
        "Max tokens: {}",
        config
            .max_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "unlimited (model's maximum)".to_string())
    );
    log::debug!("Timeout: {}s", config.timeout_secs);
    log::debug!("Validate tokens: {}", config.validate_tokens);
    log::debug!("Context limit: {:?}", config.context_limit);
    log::debug!(
        "Response format: {}",
        config
            .response_format
            .as_ref()
            .map(|f| f.to_string())
            .unwrap_or_else(|| "not set".to_string())
    );
    // Log API key source (not the actual key value)
    let api_key_source = match (
        args.api_key.as_ref(),
        args.api_key_name.as_ref(),
        file_config.as_ref().and_then(|c| c.api_key_name.as_ref()),
        file_config.as_ref().and_then(|c| c.api_key.as_ref()),
    ) {
        (Some(_), _, _, _) => "CLI argument (--api-key)".to_string(),
        (None, Some(env_name), _, _) => {
            format!("environment variable: {env_name} (--api-key-name)")
        }
        (None, None, Some(env_name), _) => {
            format!("environment variable: {env_name} (config file api_key_name)")
        }
        (None, None, None, Some(_)) => "config file (api_key)".to_string(),
        (None, None, None, None) => "not set".to_string(),
    };

    log::debug!(
        "API key: {} (source: {})",
        if config.api_key.is_some() {
            "[REDACTED]"
        } else {
            "[NOT SET]"
        },
        api_key_source
    );
    log::debug!(
        "PDF input: {:?}",
        config.pdf_input.as_ref().map(|p| p.display())
    );
    log::debug!(
        "System prompt length: {} chars",
        config.system_prompt.chars().count()
    );
    log::debug!(
        "User prompt length: {} chars",
        config.user_prompt.chars().count()
    );
    log::debug!(
        "Input validation/guardrails: {}",
        if config.input_guardrails.is_some() {
            "enabled"
        } else {
            "disabled"
        }
    );
    log::debug!("=============================");

    // Call library function
    evaluate(config).await
}
