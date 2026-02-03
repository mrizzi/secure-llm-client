use crate::{error::CliError, guardrails::provider::Severity};
use regex::Regex;
use std::{fs, path::Path};

/// Pattern scope (where the pattern applies)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternScope {
    /// Pattern applies to input validation only
    Input,
    /// Pattern applies to output validation only
    Output,
    /// Pattern applies to both input and output validation
    Both,
}

/// A pattern definition from external file
#[derive(Debug, Clone)]
pub struct PatternDefinition {
    pub scope: PatternScope,
    pub regex: Regex,
    pub description: String,
    pub severity: Severity,
}

impl PatternDefinition {
    /// Check if this pattern applies to input validation
    pub fn applies_to_input(&self) -> bool {
        matches!(self.scope, PatternScope::Input | PatternScope::Both)
    }

    /// Check if this pattern applies to output validation
    pub fn applies_to_output(&self) -> bool {
        matches!(self.scope, PatternScope::Output | PatternScope::Both)
    }
}

/// Load patterns from a file
///
/// File format (tab-delimited):
/// ```text
/// # Lines starting with # are comments
/// # Empty lines are ignored
/// # Format: scope<TAB>pattern<TAB>description<TAB>severity
/// # Scope: input, output, or both
/// # Severity: low, medium, high, critical (case-insensitive)
///
/// input    \b[0-9]{3}-[0-9]{2}-[0-9]{4}\b    Custom SSN pattern    critical
/// output    (?i)confidential data leaked    Data leakage check    high
/// both    (?i)(api[_-]?key|secret[_-]?token)    API credentials    critical
/// ```
pub fn load_patterns_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<PatternDefinition>, CliError> {
    let content = fs::read_to_string(path.as_ref()).map_err(|e| {
        let path_display = path.as_ref().display();
        CliError::InvalidResponse(format!("Failed to read pattern file {path_display}: {e}"))
    })?;

    parse_patterns(&content)
}

/// Parse patterns from string content
pub fn parse_patterns(content: &str) -> Result<Vec<PatternDefinition>, CliError> {
    let mut patterns = Vec::new();
    let mut line_number = 0;

    for line in content.lines() {
        line_number += 1;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse tab-delimited format: scope\tpattern\tdescription\tseverity
        let parts: Vec<&str> = line.split('\t').map(|s| s.trim()).collect();

        if parts.len() != 4 {
            return Err(CliError::InvalidResponse(format!(
                "Invalid pattern format at line {}: expected 4 tab-separated fields (scope<TAB>pattern<TAB>description<TAB>severity), got {}",
                line_number,
                parts.len()
            )));
        }

        let scope = parse_scope(parts[0])
            .map_err(|e| CliError::InvalidResponse(format!("Line {line_number}: {e}")))?;

        let regex = Regex::new(parts[1]).map_err(|e| {
            let pattern = parts[1];
            CliError::InvalidResponse(format!(
                "Line {line_number}: Invalid regex pattern '{pattern}': {e}"
            ))
        })?;

        let description = parts[2].to_string();

        let severity = parse_severity(parts[3])
            .map_err(|e| CliError::InvalidResponse(format!("Line {line_number}: {e}")))?;

        patterns.push(PatternDefinition {
            scope,
            regex,
            description,
            severity,
        });
    }

    Ok(patterns)
}

/// Parse scope from string
fn parse_scope(s: &str) -> Result<PatternScope, String> {
    match s.to_lowercase().as_str() {
        "input" => Ok(PatternScope::Input),
        "output" => Ok(PatternScope::Output),
        "both" => Ok(PatternScope::Both),
        _ => Err(format!(
            "Invalid scope '{s}'. Must be 'input', 'output', or 'both'"
        )),
    }
}

/// Parse severity from string
fn parse_severity(s: &str) -> Result<Severity, String> {
    match s.to_lowercase().as_str() {
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        "critical" => Ok(Severity::Critical),
        _ => Err(format!(
            "Invalid severity '{s}'. Must be 'low', 'medium', 'high', or 'critical'"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_patterns_basic() {
        let content = "# This is a comment
input	test_pattern	Test description	high
output	another_pattern	Another test	medium
both	shared_pattern	Shared check	critical
";

        let patterns = parse_patterns(content).unwrap();
        assert_eq!(patterns.len(), 3);

        assert_eq!(patterns[0].scope, PatternScope::Input);
        assert_eq!(patterns[0].description, "Test description");
        assert_eq!(patterns[0].severity, Severity::High);

        assert_eq!(patterns[1].scope, PatternScope::Output);
        assert_eq!(patterns[1].description, "Another test");
        assert_eq!(patterns[1].severity, Severity::Medium);

        assert_eq!(patterns[2].scope, PatternScope::Both);
        assert_eq!(patterns[2].description, "Shared check");
        assert_eq!(patterns[2].severity, Severity::Critical);
    }

    #[test]
    fn test_parse_patterns_empty_lines() {
        let content = "
input	pattern1	Description	low

# Comment in the middle

output	pattern2	Description	high

";

        let patterns = parse_patterns(content).unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_parse_patterns_regex() {
        let content = "
input	\\b[0-9]{3}-[0-9]{2}-[0-9]{4}\\b	SSN pattern	critical
output	(?i)password\\s+\\w+	Credentials	high
both	[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}	Email	medium
";

        let patterns = parse_patterns(content).unwrap();
        assert_eq!(patterns.len(), 3);

        // Test that regexes actually work
        assert!(patterns[0].regex.is_match("123-45-6789"));
        assert!(patterns[1].regex.is_match("my password is secret"));
        assert!(patterns[2].regex.is_match("user@example.com"));
    }

    #[test]
    fn test_parse_patterns_invalid_scope() {
        let content = "invalid_scope	pattern	description	high";
        let result = parse_patterns(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid scope"));
    }

    #[test]
    fn test_parse_patterns_invalid_severity() {
        let content = "input	pattern	description	invalid_severity";
        let result = parse_patterns(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid severity"));
    }

    #[test]
    fn test_parse_patterns_invalid_regex() {
        let content = "input	[unclosed	description	high";
        let result = parse_patterns(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex"));
    }

    #[test]
    fn test_parse_patterns_wrong_field_count() {
        let content = "input	pattern	description"; // Missing severity
        let result = parse_patterns(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected 4"));
    }

    #[test]
    fn test_pattern_applies_to_input() {
        let input_pattern = PatternDefinition {
            scope: PatternScope::Input,
            regex: Regex::new("test").unwrap(),
            description: "Test".to_string(),
            severity: Severity::Low,
        };
        assert!(input_pattern.applies_to_input());
        assert!(!input_pattern.applies_to_output());

        let both_pattern = PatternDefinition {
            scope: PatternScope::Both,
            regex: Regex::new("test").unwrap(),
            description: "Test".to_string(),
            severity: Severity::Low,
        };
        assert!(both_pattern.applies_to_input());
        assert!(both_pattern.applies_to_output());
    }

    #[test]
    fn test_pattern_applies_to_output() {
        let output_pattern = PatternDefinition {
            scope: PatternScope::Output,
            regex: Regex::new("test").unwrap(),
            description: "Test".to_string(),
            severity: Severity::Low,
        };
        assert!(!output_pattern.applies_to_input());
        assert!(output_pattern.applies_to_output());
    }

    #[test]
    fn test_parse_patterns_case_insensitive() {
        let content = "
INPUT	pattern1	Description	LOW
OUTPUT	pattern2	Description	HIGH
BOTH	pattern3	Description	CRITICAL
";

        let patterns = parse_patterns(content).unwrap();
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].scope, PatternScope::Input);
        assert_eq!(patterns[0].severity, Severity::Low);
        assert_eq!(patterns[1].scope, PatternScope::Output);
        assert_eq!(patterns[1].severity, Severity::High);
        assert_eq!(patterns[2].scope, PatternScope::Both);
        assert_eq!(patterns[2].severity, Severity::Critical);
    }
}
