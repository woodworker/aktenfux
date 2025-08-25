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

pub fn parse_frontmatter_from_file<P: AsRef<Path>>(path: P, verbose: bool) -> Result<ParseResult> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;

    let path_str = path.as_ref().to_string_lossy().to_string();
    
    let (frontmatter_opt, warning) = extract_frontmatter(&content, &path_str, verbose)?;
    
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
            // If YAML parsing fails, return warning message and empty frontmatter
            let warning = format!("Failed to parse frontmatter in file {}: {}", file_path, e);
            Ok((Some(HashMap::new()), Some(warning)))
        }
    }
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
}