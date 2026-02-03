use fortified_llm_client::CliError;
use std::path::PathBuf;

/// Load prompt from file or text string
///
/// Exactly one of `file` or `text` must be provided.
///
/// # Arguments
///
/// * `file` - Optional path to prompt file
/// * `text` - Optional inline prompt text
///
/// # Returns
///
/// - `Ok(String)` - The prompt content
/// - `Err(CliError)` - If both or neither provided, or file read fails
///
/// # Example
///
/// ```ignore
/// use std::path::PathBuf;
///
/// // Load from file
/// let prompt = load_prompt(Some(PathBuf::from("prompt.txt")), None)?;
///
/// // Load from inline text
/// let prompt = load_prompt(None, Some("You are helpful".to_string()))?;
/// ```
pub fn load_prompt(file: Option<PathBuf>, text: Option<String>) -> Result<String, CliError> {
    match (file, text) {
        (Some(path), None) => std::fs::read_to_string(&path).map_err(|e| {
            CliError::FileNotFound(format!("Failed to read file '{}': {e}", path.display()))
        }),
        (None, Some(content)) => Ok(content),
        (Some(_), Some(_)) => Err(CliError::InvalidArguments(
            "Cannot provide both file and text for the same prompt".to_string(),
        )),
        (None, None) => Err(CliError::InvalidArguments(
            "Must provide either file or text for prompt".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_prompt_from_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("txt");
        let content = "Test prompt content";
        fs::write(&path, content).unwrap();

        let result = load_prompt(Some(path.clone()), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);

        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_prompt_from_text() {
        let text = "Inline prompt text".to_string();
        let result = load_prompt(None, Some(text.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), text);
    }

    #[test]
    fn test_load_prompt_both_provided() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("txt");
        fs::write(&path, "file content").unwrap();

        let result = load_prompt(Some(path.clone()), Some("text content".to_string()));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot provide both"));

        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_prompt_neither_provided() {
        let result = load_prompt(None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Must provide either"));
    }

    #[test]
    fn test_load_prompt_file_not_found() {
        let result = load_prompt(Some(PathBuf::from("/nonexistent/file.txt")), None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file"));
    }
}
