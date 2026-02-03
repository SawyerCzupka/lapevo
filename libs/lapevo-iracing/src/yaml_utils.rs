use crate::error::{IracingError, Result};

/// Preprocess iRacing YAML to fix non-standard format issues.
pub fn preprocess_iracing_yaml(yaml: &str) -> Result<String> {
    let mut result = String::with_capacity(yaml.len());
    let mut prev_char = ' ';

    for ch in yaml.chars() {
        match ch {
            // Remove control characters except newline, carriage return, tab
            '\x00'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' => continue,
            _ => result.push(ch),
        }
        prev_char = ch;
    }
    let _ = prev_char;

    if result.trim().is_empty() {
        return Err(IracingError::Yaml("YAML is empty after preprocessing".to_string()));
    }

    Ok(result)
}

/// Extract YAML from a memory buffer at the given offset/length.
pub fn extract_yaml_from_memory(data: &[u8], offset: i32, length: i32) -> Result<String> {
    if offset < 0 {
        return Err(IracingError::Parse {
            context: "YAML extraction".to_string(),
            details: format!("Invalid offset: {}", offset),
        });
    }
    if length <= 0 {
        return Ok(String::new());
    }

    let offset = offset as usize;
    let length = length as usize;

    if offset + length > data.len() {
        return Err(IracingError::Parse {
            context: "YAML extraction".to_string(),
            details: format!(
                "YAML extends beyond buffer: offset={}, len={}, buf_size={}",
                offset, length, data.len()
            ),
        });
    }

    let yaml_data = &data[offset..offset + length];
    let yaml_len = yaml_data.iter().position(|&b| b == 0).unwrap_or(length);

    std::str::from_utf8(&yaml_data[..yaml_len])
        .map(|s| s.to_string())
        .map_err(|e| IracingError::Parse {
            context: "YAML UTF-8 conversion".to_string(),
            details: e.to_string(),
        })
}
