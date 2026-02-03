use crate::{error::CliError, models::ResponseFormat};
use async_trait::async_trait;

/// Parameters for LLM invocation
///
/// This struct consolidates all parameters for LLM invocation to avoid
/// parameter explosion in the `invoke()` method signature.
///
/// # Lifetimes
///
/// - `'a` - Lifetime of borrowed string parameters (model, prompts, api_key)
///
/// # Example
///
/// ```ignore
/// let params = InvokeParams {
///     model: "gpt-4",
///     system_prompt: "You are a helpful assistant.",
///     user_prompt: "What is 2+2?",
///     temperature: 0.7,
///     max_tokens: Some(100),
///     seed: Some(42),
///     api_key: Some("sk-..."),
///     timeout_secs: 30,
///     response_format: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct InvokeParams<'a> {
    /// Model name/identifier
    pub model: &'a str,

    /// System prompt (instructions for the LLM)
    pub system_prompt: &'a str,

    /// User prompt (the actual query/input)
    pub user_prompt: &'a str,

    /// Sampling temperature (0.0 = deterministic, 2.0 = maximum randomness)
    pub temperature: f32,

    /// Maximum tokens to generate in response (None = use model's maximum)
    pub max_tokens: Option<u32>,

    /// Random seed for reproducible sampling (None = non-deterministic)
    pub seed: Option<u64>,

    /// Optional API key for authentication
    pub api_key: Option<&'a str>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Optional response format constraint (OpenAI-compatible only)
    pub response_format: Option<&'a ResponseFormat>,
}

/// LLM provider trait for extensibility
///
/// This trait defines the interface for LLM provider implementations.
/// Different providers (OpenAI, Ollama, etc.) implement this trait to
/// provide a consistent interface for LLM invocation.
///
/// # Implementations
///
/// - `OpenAIProvider` - For OpenAI-compatible APIs
/// - `OllamaProvider` - For Ollama /api/generate format
///
/// # Example
///
/// ```ignore
/// use secure_llm_client::provider::{InvokeParams, LlmProvider};
/// use secure_llm_client::providers::OpenAIProvider;
///
/// let provider = OpenAIProvider::new("https://api.openai.com/v1/chat/completions".to_string());
///
/// let params = InvokeParams {
///     model: "gpt-4",
///     system_prompt: "You are helpful.",
///     user_prompt: "Say hello",
///     temperature: 0.7,
///     max_tokens: Some(100),
///     api_key: Some("sk-..."),
///     timeout_secs: 30,
///     response_format: None,
/// };
///
/// let response = provider.invoke(params).await?;
/// ```
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Invoke the LLM with given parameters
    ///
    /// # Arguments
    ///
    /// * `params` - All invocation parameters consolidated in a single struct
    ///
    /// # Returns
    ///
    /// The LLM's response as a string, or an error if the invocation failed
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError>;

    /// Get provider name for logging and debugging
    fn name(&self) -> &str;

    /// Check if provider supports streaming
    ///
    /// Default implementation returns false. Providers that support
    /// streaming should override this method.
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Provider types for LLM API formats
///
/// Different LLM providers use different API formats. This enum
/// identifies which format to use.
///
/// # Variants
///
/// - `Ollama` - For Ollama /api/generate format (local servers)
/// - `OpenAI` - For OpenAI-compatible /v1/chat/completions format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// Ollama /api/generate format (local servers)
    Ollama,
    /// OpenAI-compatible /v1/chat/completions format
    OpenAI,
}
