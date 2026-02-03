use fortified_llm_client::constants::llm_defaults;
use std::path::PathBuf;

// Validation constants
const MIN_TOKENS: u32 = 1;
const MIN_TIMEOUT: u64 = 1;
const MIN_CONTEXT_LIMIT: usize = 1;

/// Validate temperature value (must be within LLM defaults range)
pub fn validate_temperature(s: &str) -> Result<f32, String> {
    let temp: f32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    (llm_defaults::MIN_TEMPERATURE..=llm_defaults::MAX_TEMPERATURE)
        .contains(&temp)
        .then_some(temp)
        .ok_or_else(|| {
            format!(
                "Temperature must be between {} and {}, got {temp}",
                llm_defaults::MIN_TEMPERATURE,
                llm_defaults::MAX_TEMPERATURE
            )
        })
}

/// Validate positive u32 value (must be >= MIN_TOKENS)
pub fn validate_positive_u32(s: &str) -> Result<u32, String> {
    let val: u32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    (val >= MIN_TOKENS)
        .then_some(val)
        .ok_or_else(|| format!("Value must be >= {MIN_TOKENS}"))
}

/// Validate positive u64 value (must be >= MIN_TIMEOUT)
pub fn validate_positive_u64(s: &str) -> Result<u64, String> {
    let val: u64 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    (val >= MIN_TIMEOUT)
        .then_some(val)
        .ok_or_else(|| format!("Value must be >= {MIN_TIMEOUT}"))
}

/// Validate context limit value (must be >= MIN_CONTEXT_LIMIT)
pub fn validate_context_limit(s: &str) -> Result<usize, String> {
    let val: usize = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    (val >= MIN_CONTEXT_LIMIT)
        .then_some(val)
        .ok_or_else(|| format!("Context limit must be >= {MIN_CONTEXT_LIMIT}"))
}

/// Validate positive usize value (must be > 0)
pub fn validate_positive_usize(s: &str) -> Result<usize, String> {
    let val: usize = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    (val > 0)
        .then_some(val)
        .ok_or_else(|| "Value must be > 0".to_string())
}

/// Validate byte size value (supports human-readable formats like "100MB", "1.5GB")
///
/// Supported formats:
/// - Plain numbers (bytes): "1048576"
/// - Human-readable sizes: "100MB", "1.5GB", "500KB"
/// - Units: B, KB, MB, GB, TB (or K, M, G, T)
/// - SI units: KIB, MIB, GIB, TIB
pub fn validate_byte_size(s: &str) -> Result<usize, String> {
    let s = s.trim();

    // Try to parse as plain number first (bytes)
    if let Ok(val) = s.parse::<usize>() {
        return if val > 0 {
            Ok(val)
        } else {
            Err("Size must be > 0".to_string())
        };
    }

    // Parse human-readable size (e.g., "100MB", "1.5GB")
    let s_upper = s.to_uppercase();

    // Extract number and unit
    let (num_str, unit) = if let Some(pos) = s_upper.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], &s_upper[pos..])
    } else {
        return Err(format!(
            "Invalid size format: '{s}'. Use formats like '100MB', '1.5GB', or plain bytes"
        ));
    };

    // Parse the number (supports decimals like "1.5GB")
    let number: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("Invalid number: '{num_str}'"))?;

    if number <= 0.0 {
        return Err("Size must be > 0".to_string());
    }

    // Determine multiplier based on unit
    let multiplier: usize = match unit {
        "B" => 1,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024 * 1024 * 1024 * 1024,
        // Also support SI units (powers of 1000)
        "KIB" => 1024,
        "MIB" => 1024 * 1024,
        "GIB" => 1024 * 1024 * 1024,
        "TIB" => 1024 * 1024 * 1024 * 1024,
        _ => {
            return Err(format!(
                "Unknown unit: '{unit}'. Supported: B, KB, MB, GB, TB (or K, M, G, T)"
            ))
        }
    };

    // Calculate total bytes
    let bytes = (number * multiplier as f64) as usize;

    if bytes == 0 {
        return Err("Resulting size is too small (rounds to 0 bytes)".to_string());
    }

    Ok(bytes)
}

/// Validate file exists at the given path
pub fn validate_file_exists(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("File does not exist: '{s}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_temperature_valid() {
        assert_eq!(validate_temperature("0.7").unwrap(), 0.7);
        assert_eq!(validate_temperature("0.0").unwrap(), 0.0);
        assert_eq!(validate_temperature("2.0").unwrap(), 2.0);
    }

    #[test]
    fn test_validate_temperature_out_of_range() {
        assert!(validate_temperature("2.5").is_err());
        assert!(validate_temperature("-0.1").is_err());
    }

    #[test]
    fn test_validate_temperature_invalid() {
        assert!(validate_temperature("abc").is_err());
    }

    #[test]
    fn test_validate_positive_u32_valid() {
        assert_eq!(validate_positive_u32("1").unwrap(), 1);
        assert_eq!(validate_positive_u32("100").unwrap(), 100);
    }

    #[test]
    fn test_validate_positive_u32_zero() {
        assert!(validate_positive_u32("0").is_err());
    }

    #[test]
    fn test_validate_byte_size_plain_bytes() {
        assert_eq!(validate_byte_size("1048576").unwrap(), 1048576);
    }

    #[test]
    fn test_validate_byte_size_mb() {
        assert_eq!(validate_byte_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(validate_byte_size("100MB").unwrap(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_validate_byte_size_gb() {
        assert_eq!(
            validate_byte_size("1.5GB").unwrap(),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as usize
        );
    }

    #[test]
    fn test_validate_byte_size_invalid_unit() {
        assert!(validate_byte_size("100XB").is_err());
    }

    #[test]
    fn test_validate_byte_size_zero() {
        assert!(validate_byte_size("0").is_err());
        assert!(validate_byte_size("0MB").is_err());
    }
}
