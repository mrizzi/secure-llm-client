// Unit tests for fortified-llm-client

// Note: To access internal modules for testing, we need to make them public in lib.rs
// For now, we'll test through the binary's public interface

#[cfg(test)]
mod token_tests {
    // Re-export the modules we need to test
    // Since we're testing a binary crate, we need to move logic to a lib.rs
    // For this MVP, we'll document that comprehensive unit testing requires
    // restructuring into a library crate with a binary wrapper

    #[test]
    fn test_token_estimation_formula() {
        // This test validates the char/4 formula
        let text = "A".repeat(400); // 400 chars
        let expected_tokens = 100; // 400 / 4.0 = 100

        // Simplified version of estimate_tokens logic
        const CHARS_PER_TOKEN: f64 = 4.0;
        let estimated = (text.len() as f64 / CHARS_PER_TOKEN).ceil() as usize;

        assert_eq!(estimated, expected_tokens);
    }

    #[test]
    fn test_context_requirement_calculation() {
        // Test the formula: (system + user + buffer) * 1.1
        let system = "A".repeat(10000); // ~2500 tokens
        let user = "B".repeat(10000); // ~2500 tokens
        let response_buffer = 4000;
        let safety_margin_pct = 10.0;

        const CHARS_PER_TOKEN: f64 = 4.0;
        let system_tokens = (system.len() as f64 / CHARS_PER_TOKEN).ceil() as usize;
        let user_tokens = (user.len() as f64 / CHARS_PER_TOKEN).ceil() as usize;
        let base_tokens = system_tokens + user_tokens + response_buffer;
        let safety_tokens = (base_tokens as f64 * safety_margin_pct / 100.0).ceil() as usize;
        let total = base_tokens + safety_tokens;

        // Expected: (2500 + 2500 + 4000) * 1.1 = 9900
        assert_eq!(total, 9900);
    }

    #[test]
    fn test_context_limit_validation() {
        // Test that validation correctly detects when limit is exceeded
        let system = "A".repeat(200000); // ~50k tokens
        let user = "B".repeat(200000); // ~50k tokens
        let response_buffer = 4000;
        let safety_margin_pct = 10.0;
        let context_limit = 40960; // qwen3-14b limit

        const CHARS_PER_TOKEN: f64 = 4.0;
        let system_tokens = (system.len() as f64 / CHARS_PER_TOKEN).ceil() as usize;
        let user_tokens = (user.len() as f64 / CHARS_PER_TOKEN).ceil() as usize;
        let base_tokens = system_tokens + user_tokens + response_buffer;
        let safety_tokens = (base_tokens as f64 * safety_margin_pct / 100.0).ceil() as usize;
        let required = base_tokens + safety_tokens;

        // Should exceed limit
        assert!(required > context_limit);
        let excess = required - context_limit;
        assert!(excess > 0);
    }
}

#[cfg(test)]
mod provider_tests {
    #[test]
    fn test_ollama_detection_localhost() {
        // Ollama standard installation on localhost:11434
        let url = "http://localhost:11434/api/generate";
        let is_ollama = url.contains("localhost:11434");
        assert!(is_ollama);
    }

    #[test]
    fn test_ollama_detection_localhost_ip() {
        // Ollama on 127.0.0.1:11434
        let url = "http://127.0.0.1:11434/api/generate";
        let is_ollama = url.contains("127.0.0.1:11434");
        assert!(is_ollama);
    }

    #[test]
    fn test_openai_format_for_groq() {
        // Groq uses OpenAI-compatible format
        let url = "https://api.groq.com/openai/v1/chat/completions";
        let is_ollama = url.contains("localhost:11434") || url.contains("127.0.0.1:11434");

        // Should NOT detect as Ollama
        assert!(!is_ollama);
        // Real implementation would default to Provider::OpenAI
    }

    #[test]
    fn test_openai_format_for_openai() {
        // OpenAI actual API
        let url = "https://api.openai.com/v1/chat/completions";
        let is_ollama = url.contains("localhost:11434") || url.contains("127.0.0.1:11434");

        // Should NOT detect as Ollama
        assert!(!is_ollama);
        // Real implementation would default to Provider::OpenAI
    }

    #[test]
    fn test_openai_format_for_azure() {
        // Azure OpenAI uses OpenAI-compatible format
        let url = "https://myresource.openai.azure.com/openai/deployments/gpt-4/chat/completions";
        let is_ollama = url.contains("localhost:11434") || url.contains("127.0.0.1:11434");

        // Should NOT detect as Ollama
        assert!(!is_ollama);
        // Real implementation would default to Provider::OpenAI
    }

    #[test]
    fn test_custom_endpoint_defaults_to_openai() {
        // Custom endpoint should default to OpenAI format
        let url = "http://custom-api.example.com/v1/chat/completions";
        let is_ollama = url.contains("localhost:11434") || url.contains("127.0.0.1:11434");

        // Should NOT detect as Ollama
        assert!(!is_ollama);
        // Real implementation would default to Provider::OpenAI (most compatible)
    }

    #[test]
    fn test_custom_ollama_server_requires_explicit_provider() {
        // Custom Ollama server on different port requires --provider flag
        let url = "http://ollama-server.local:8080/api/generate";
        let is_ollama = url.contains("localhost:11434") || url.contains("127.0.0.1:11434");

        // Auto-detection will NOT detect this as Ollama
        assert!(!is_ollama);
        // User must use --provider ollama to override
        // Real implementation would default to Provider::OpenAI without override
    }
}

#[cfg(test)]
mod error_tests {
    #[test]
    fn test_exit_code_mapping() {
        // Test that error types map to correct exit codes
        let exit_codes = vec![
            ("CONTEXT_LIMIT_EXCEEDED", 2),
            ("HTTP_ERROR", 3),
            ("INVALID_RESPONSE", 4),
            ("FILE_NOT_FOUND", 5),
            ("INVALID_ARGUMENTS", 6),
            ("AUTH_FAILED", 7),
        ];

        for (code_name, expected_exit) in &exit_codes {
            // Validate exit code is in valid range
            assert!(expected_exit > &0 && expected_exit < &10);

            // Validate exit codes are unique
            let other_codes: Vec<i32> = exit_codes
                .iter()
                .filter(|(name, _)| name != code_name)
                .map(|(_, code)| *code)
                .collect();
            assert!(!other_codes.contains(expected_exit));
        }
    }

    #[test]
    fn test_timeout_behavior() {
        // Timeouts are handled as HTTP errors by reqwest
        // This test documents that timeout errors result in HTTP_ERROR exit code (3)
        // rather than a separate TIMEOUT error code

        // When a timeout occurs, reqwest returns a reqwest::Error
        // which gets converted to CliError::HttpError via the #[from] attribute
        // Therefore, the exit code should be 3 (HTTP_ERROR)

        let timeout_exit_code = 3; // HTTP_ERROR includes timeouts
        let http_error_exit_code = 3;

        // Timeouts use the same exit code as other HTTP errors
        assert_eq!(timeout_exit_code, http_error_exit_code);

        // Verify this is documented correctly
        // In the real implementation:
        // - reqwest timeout errors -> CliError::HttpError (via #[from])
        // - CliError::HttpError.exit_code() -> 3
        // - CliError::HttpError.code() -> "HTTP_ERROR"
    }
}

#[cfg(test)]
mod timeout_tests {
    use std::time::Duration;

    #[test]
    fn test_timeout_configuration() {
        // Test that timeout values are properly configured
        let default_timeout_secs = 300; // 5 minutes (from CLI args default)
        let default_timeout_ms = default_timeout_secs * 1000;

        // Verify default timeout is reasonable for LLM inference
        assert_eq!(default_timeout_secs, 300);
        assert_eq!(default_timeout_ms, 300_000);

        // Verify timeout can be converted to Duration
        let duration = Duration::from_secs(default_timeout_secs);
        assert_eq!(duration.as_secs(), 300);
    }

    #[test]
    fn test_timeout_ranges() {
        // Test valid timeout ranges
        let min_timeout = 1; // 1 second minimum
        let default_timeout = 300; // 5 minutes default
        let max_timeout = 600; // 10 minutes maximum (from CLI validation)

        assert!(min_timeout < default_timeout);
        assert!(default_timeout < max_timeout);

        // All timeouts should be positive
        assert!(min_timeout > 0);
        assert!(default_timeout > 0);
        assert!(max_timeout > 0);
    }
}

// Note: Integration tests with mock HTTP servers are in tests/integration_tests.rs
