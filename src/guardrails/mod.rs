pub mod config;
pub mod gpt_oss_safeguard;
pub mod hybrid;
pub mod llama_guard;
pub mod llama_prompt_guard;
pub mod patterns;
pub mod provider;
pub mod regex;

// Re-export core trait types
pub use provider::{
    GptOssSafeguardResult, GuardrailProvider, GuardrailResult, LlamaGuardResult,
    ProviderSpecificResult, Severity, Violation,
};

// Re-export concrete implementations
pub use config::{
    create_guardrail_provider, AggregationMode, ExecutionMode, GuardrailConfig,
    GuardrailProviderConfig, RegexGuardrailConfig,
};
pub use gpt_oss_safeguard::{GptOssSafeguardConfig, GptOssSafeguardProvider};
pub use hybrid::HybridGuardrail;
pub use llama_guard::{LlamaGuardCategory, LlamaGuardConfig, LlamaGuardProvider};
pub use llama_prompt_guard::{
    LlamaPromptGuardConfig, LlamaPromptGuardProvider, LlamaPromptGuardResult,
};
pub use regex::RegexGuardrail;

// Type aliases
/// Type alias for RegexGuardrail used for input validation
pub type InputGuardrail = RegexGuardrail;

/// Type alias for RegexGuardrail used for output validation
pub type OutputGuardrail = RegexGuardrail;
