use crate::{
    error::CliError,
    provider::{InvokeParams, LlmProvider},
    providers::create_provider,
};

pub use crate::provider::ProviderType as Provider;

pub struct LlmClient {
    provider: Box<dyn LlmProvider>,
}

impl LlmClient {
    pub fn new(api_url: String, provider: Option<Provider>) -> Self {
        Self {
            provider: create_provider(api_url, provider),
        }
    }

    /// Invoke the LLM with consolidated parameters
    ///
    /// # Arguments
    ///
    /// * `params` - All invocation parameters in a single struct
    ///
    /// # Returns
    ///
    /// The LLM's response as a string, or an error if invocation failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use secure_llm_client::{LlmClient, InvokeParams};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::new("http://localhost:11434/v1/chat/completions".to_string(), None);
    ///
    /// let params = InvokeParams {
    ///     model: "llama3",
    ///     system_prompt: "You are helpful.",
    ///     user_prompt: "Say hello",
    ///     temperature: 0.7,
    ///     max_tokens: Some(100),
    ///     seed: None,
    ///     api_key: None,
    ///     timeout_secs: 30,
    ///     response_format: None,
    /// };
    ///
    /// let response = client.invoke(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        self.provider.invoke(params).await
    }
}
