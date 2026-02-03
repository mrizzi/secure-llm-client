use crate::{constants::token_estimation, model_registry};

/// Token estimator for calculating context requirements
pub struct TokenEstimator {
    system_tokens: usize,
    user_tokens: usize,
    response_buffer: usize,
}

impl TokenEstimator {
    /// Create a new token estimator using generic estimation (4 chars/token)
    pub fn new(system_prompt: &str, user_prompt: &str, response_buffer: u32) -> Self {
        Self {
            system_tokens: estimate_tokens(system_prompt),
            user_tokens: estimate_tokens(user_prompt),
            response_buffer: response_buffer as usize,
        }
    }

    /// Create a new token estimator using model-specific tokenizer characteristics
    ///
    /// Falls back to generic estimation if model is not recognized.
    pub fn new_for_model(
        system_prompt: &str,
        user_prompt: &str,
        response_buffer: u32,
        model_name: &str,
    ) -> Self {
        if let Some(model_info) = model_registry::lookup_model(model_name) {
            log::debug!(
                "Using model-specific token estimation for {} ({} chars/token)",
                model_name,
                model_info.tokenizer.chars_per_token()
            );
            Self {
                system_tokens: model_info.estimate_tokens(system_prompt),
                user_tokens: model_info.estimate_tokens(user_prompt),
                response_buffer: response_buffer as usize,
            }
        } else {
            log::debug!("Model '{model_name}' not in registry, using generic token estimation");
            Self::new(system_prompt, user_prompt, response_buffer)
        }
    }

    pub fn total_tokens_required(&self) -> usize {
        let base_tokens = self.system_tokens + self.user_tokens + self.response_buffer;
        let safety_tokens =
            (base_tokens as f64 * (token_estimation::SAFETY_MARGIN - 1.0)).ceil() as usize;
        base_tokens + safety_tokens
    }

    /// Get breakdown of token usage
    pub fn breakdown(&self) -> TokenBreakdown {
        TokenBreakdown {
            system_tokens: self.system_tokens,
            user_tokens: self.user_tokens,
            response_buffer: self.response_buffer,
            total_required: self.total_tokens_required(),
        }
    }
}

/// Detailed breakdown of token usage
#[derive(Debug, Clone)]
pub struct TokenBreakdown {
    pub system_tokens: usize,
    pub user_tokens: usize,
    pub response_buffer: usize,
    pub total_required: usize,
}

/// Estimate token count from text using char/4 approximation
/// This matches the estimation logic in run_evaluation.sh
///
/// NOTE: This is model-agnostic. Different tokenizers may vary:
/// - GPT tokenizer: ~4.0 chars/token (English)
/// - Llama tokenizer: ~3.5 chars/token
/// - Code: ~2.5 chars/token
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() as f64 / token_estimation::CHARS_PER_TOKEN).ceil() as usize
}
