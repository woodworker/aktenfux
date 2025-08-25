use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ParseResult {
    pub note: Option<Note>,
    pub frontmatter_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub path: String,
    pub frontmatter: HashMap<String, Value>,
    pub title: Option<String>,
}

impl Note {
    pub fn new(path: String, frontmatter: HashMap<String, Value>) -> Self {
        let title = frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Extract title from filename if not in frontmatter
                Path::new(&path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            });

        Self {
            path,
            frontmatter,
            title,
        }
    }

    pub fn get_frontmatter_value(&self, key: &str) -> Option<&Value> {
        self.frontmatter.get(key)
    }

    pub fn matches_filter(&self, key: &str, value: &str) -> bool {
        if let Some(fm_value) = self.get_frontmatter_value(key) {
            match fm_value {
                Value::String(s) => s.contains(value),
                Value::Sequence(seq) => {
                    seq.iter().any(|v| {
                        if let Value::String(s) = v {
                            s.contains(value)
                        } else {
                            false
                        }
                    })
                }
                Value::Number(n) => n.to_string().contains(value),
                Value::Bool(b) => b.to_string().contains(value),
                _ => false,
            }
        } else {
            false
        }
    }
}

pub fn parse_frontmatter_from_file<P: AsRef<Path>>(path: P, verbose: bool, lenient: bool) -> Result<ParseResult> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;

    let path_str = path.as_ref().to_string_lossy().to_string();
    
    let (frontmatter_opt, warning) = extract_frontmatter_with_options(&content, &path_str, verbose, lenient)?;
    
    let note = if let Some(frontmatter) = frontmatter_opt {
        Some(Note::new(path_str.clone(), frontmatter))
    } else {
        // Create note with empty frontmatter if no frontmatter found
        Some(Note::new(path_str, HashMap::new()))
    };
    
    Ok(ParseResult {
        note,
        frontmatter_warning: warning,
    })
}

fn extract_frontmatter(content: &str, file_path: &str, _verbose: bool) -> Result<(Option<HashMap<String, Value>>, Option<String>)> {
    extract_frontmatter_with_options(content, file_path, _verbose, true)
}

fn extract_frontmatter_with_options(content: &str, file_path: &str, _verbose: bool, lenient: bool) -> Result<(Option<HashMap<String, Value>>, Option<String>)> {
    let content = content.trim();
    
    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---") {
        return Ok((None, None));
    }

    // Find the end of frontmatter
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, None));
    }

    let mut end_index = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(i);
            break;
        }
    }

    let end_index = match end_index {
        Some(idx) => idx,
        None => return Ok((None, None)),
    };

    // Extract frontmatter content
    let frontmatter_lines = &lines[1..end_index];
    let frontmatter_content = frontmatter_lines.join("\n");

    if frontmatter_content.trim().is_empty() {
        return Ok((Some(HashMap::new()), None));
    }

    // Parse YAML frontmatter
    match serde_yaml::from_str::<HashMap<String, Value>>(&frontmatter_content) {
        Ok(parsed) => Ok((Some(parsed), None)),
        Err(e) => {
            if lenient {
                // Try lenient parsing by fixing common YAML issues
                match try_lenient_parse(&frontmatter_content) {
                    Ok(parsed) => {
                        let warning = format!("Used lenient parsing for frontmatter in file {} due to: {}", file_path, e);
                        Ok((Some(parsed), Some(warning)))
                    },
                    Err(_) => {
                        // If lenient parsing also fails, return warning message and empty frontmatter
                        let warning = format!("Failed to parse frontmatter in file {} even with lenient parsing: {}", file_path, e);
                        Ok((Some(HashMap::new()), Some(warning)))
                    }
                }
            } else {
                // If YAML parsing fails, return warning message and empty frontmatter
                let warning = format!("Failed to parse frontmatter in file {}: {}", file_path, e);
                Ok((Some(HashMap::new()), Some(warning)))
            }
        }
    }
}

fn try_lenient_parse(frontmatter_content: &str) -> Result<HashMap<String, Value>, serde_yaml::Error> {
    // Fix common YAML issues by preprocessing the content
    let fixed_content = fix_yaml_issues(frontmatter_content);
    serde_yaml::from_str::<HashMap<String, Value>>(&fixed_content)
}

fn fix_yaml_issues(content: &str) -> String {
    let mut fixed_lines = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            fixed_lines.push(line.to_string());
            continue;
        }
        
        // Check if this looks like a key-value pair
        if let Some(colon_pos) = trimmed.find(':') {
            let key_part = &trimmed[..colon_pos];
            let value_part = &trimmed[colon_pos + 1..].trim_start();
            
            // Skip if this is already a properly formatted YAML (like arrays, objects, etc.)
            if value_part.starts_with('[') || value_part.starts_with('{') ||
               value_part.starts_with('"') || value_part.starts_with('\'') ||
               value_part.is_empty() {
                fixed_lines.push(line.to_string());
                continue;
            }
            
            // Check if the value contains additional colons and isn't already quoted
            if value_part.contains(':') && !value_part.starts_with('"') && !value_part.starts_with('\'') {
                // Quote the value to make it valid YAML
                let leading_spaces = line.len() - line.trim_start().len();
                let spaces = " ".repeat(leading_spaces);
                fixed_lines.push(format!("{}{}: \"{}\"", spaces, key_part, value_part));
            } else {
                fixed_lines.push(line.to_string());
            }
        } else {
            fixed_lines.push(line.to_string());
        }
    }
    
    fixed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = r#"---
title: Test Note
tags: [work, important]
status: active
---

# Test Note

This is the content of the note."#;

        let (result, warning) = extract_frontmatter(content, "test.md", false).unwrap();
        let result = result.unwrap();
        assert_eq!(result.get("title").unwrap().as_str().unwrap(), "Test Note");
        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "active");
        assert!(warning.is_none());
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a regular markdown file\n\nWith some content.";
        let (result, warning) = extract_frontmatter(content, "test.md", false).unwrap();
        assert!(result.is_none());
        assert!(warning.is_none());
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n\n# Note with empty frontmatter";
        let (result, warning) = extract_frontmatter(content, "test.md", false).unwrap();
        let result = result.unwrap();
        assert!(result.is_empty());
        assert!(warning.is_none());
    }

    #[test]
    fn test_frontmatter_with_colons_in_values() {
        let content = r#"---
title: Test Note
source: Eberron: Rising from the Last War p. 277
book: Player's Handbook: Chapter 3
url: https://example.com/path
---

# Test Note

This note has colons in frontmatter values."#;

        let (result, warning) = extract_frontmatter(content, "test.md", false).unwrap();
        let result = result.unwrap();
        
        assert_eq!(result.get("title").unwrap().as_str().unwrap(), "Test Note");
        assert_eq!(result.get("source").unwrap().as_str().unwrap(), "Eberron: Rising from the Last War p. 277");
        assert_eq!(result.get("book").unwrap().as_str().unwrap(), "Player's Handbook: Chapter 3");
        assert_eq!(result.get("url").unwrap().as_str().unwrap(), "https://example.com/path");
        
        // Should have a warning about lenient parsing
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Used lenient parsing"));
    }

    #[test]
    fn test_fix_yaml_issues() {
        let problematic_yaml = r#"title: Test Note
source: Eberron: Rising from the Last War p. 277
book: Player's Handbook: Chapter 3
url: https://example.com/path
tags: [work, important]
quoted: "Already: quoted"
number: 42"#;

        let fixed = fix_yaml_issues(problematic_yaml);
        
        // Should quote values with colons but leave others alone
        assert!(fixed.contains("source: \"Eberron: Rising from the Last War p. 277\""));
        assert!(fixed.contains("book: \"Player's Handbook: Chapter 3\""));
        assert!(fixed.contains("url: \"https://example.com/path\""));
        assert!(fixed.contains("title: Test Note")); // No colon in value, shouldn't be quoted
        assert!(fixed.contains("tags: [work, important]")); // Array, shouldn't be quoted
        assert!(fixed.contains("quoted: \"Already: quoted\"")); // Already quoted, shouldn't be double-quoted
        assert!(fixed.contains("number: 42")); // Number, shouldn't be quoted
    }

    #[test]
    fn test_strict_vs_lenient_parsing() {
        let content = r#"---
title: Test Note
source: Eberron: Rising from the Last War p. 277
---

# Test Note"#;

        // Test strict parsing (should fail and return empty frontmatter)
        let (result_strict, warning_strict) = extract_frontmatter_with_options(content, "test.md", false, false).unwrap();
        let result_strict = result_strict.unwrap();
        assert!(result_strict.is_empty()); // Should be empty due to parsing failure
        assert!(warning_strict.is_some());
        assert!(warning_strict.unwrap().contains("Failed to parse frontmatter"));

        // Test lenient parsing (should succeed)
        let (result_lenient, warning_lenient) = extract_frontmatter_with_options(content, "test.md", false, true).unwrap();
        let result_lenient = result_lenient.unwrap();
        assert!(!result_lenient.is_empty()); // Should have parsed content
        assert_eq!(result_lenient.get("title").unwrap().as_str().unwrap(), "Test Note");
        assert_eq!(result_lenient.get("source").unwrap().as_str().unwrap(), "Eberron: Rising from the Last War p. 277");
        assert!(warning_lenient.is_some());
        assert!(warning_lenient.unwrap().contains("Used lenient parsing"));
    }
}