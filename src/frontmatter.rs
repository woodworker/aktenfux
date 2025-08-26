use yaml_rust2::Yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use crate::yaml_compat::{parse_yaml_frontmatter, yaml_as_str, yaml_contains_str, yaml_contains_str_case_insensitive};

// Type alias for complex frontmatter extraction result
type FrontmatterResult = Result<(Option<HashMap<String, Yaml>>, Option<String>)>;

#[derive(Debug)]
pub struct ParseResult {
    pub note: Option<Note>,
    pub frontmatter_warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub path: String,
    pub frontmatter: HashMap<String, Yaml>,
    pub title: Option<String>,
}

impl Note {
    pub fn new(path: String, frontmatter: HashMap<String, Yaml>) -> Self {
        let title = frontmatter
            .get("title")
            .and_then(|v| yaml_as_str(v))
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

    pub fn get_frontmatter_value(&self, key: &str) -> Option<&Yaml> {
        self.frontmatter.get(key)
    }

    pub fn matches_filter(&self, key: &str, value: &str) -> bool {
        if let Some(fm_value) = self.get_frontmatter_value(key) {
            yaml_contains_str(fm_value, value)
        } else {
            false
        }
    }

    pub fn matches_filter_with_case_sensitivity(&self, key: &str, value: &str, case_sensitive: bool) -> bool {
        if case_sensitive {
            self.matches_filter(key, value)
        } else {
            // For case-insensitive matching, we need to check both field name and value
            let matching_key = if case_sensitive {
                self.get_frontmatter_value(key)
            } else {
                // Find field with case-insensitive key matching
                self.frontmatter.iter()
                    .find(|(k, _)| k.to_lowercase() == key.to_lowercase())
                    .map(|(_, v)| v)
            };

            if let Some(fm_value) = matching_key {
                yaml_contains_str_case_insensitive(fm_value, value)
            } else {
                false
            }
        }
    }

    pub fn get_frontmatter_value_case_insensitive(&self, key: &str) -> Option<&Yaml> {
        // First try exact match
        if let Some(value) = self.frontmatter.get(key) {
            return Some(value);
        }
        
        // Then try case-insensitive match
        let key_lower = key.to_lowercase();
        self.frontmatter.iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v)
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

#[cfg(test)]
fn extract_frontmatter(content: &str, file_path: &str, _verbose: bool) -> FrontmatterResult {
    extract_frontmatter_with_options(content, file_path, _verbose, true)
}

fn extract_frontmatter_with_options(content: &str, file_path: &str, _verbose: bool, lenient: bool) -> FrontmatterResult {
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
    match parse_yaml_frontmatter(&frontmatter_content) {
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

fn try_lenient_parse(frontmatter_content: &str) -> Result<HashMap<String, Yaml>> {
    // Fix common YAML issues by preprocessing the content
    let fixed_content = fix_yaml_issues(frontmatter_content);
    parse_yaml_frontmatter(&fixed_content)
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
        assert_eq!(yaml_as_str(result.get("title").unwrap()).unwrap(), "Test Note");
        assert_eq!(yaml_as_str(result.get("status").unwrap()).unwrap(), "active");
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
        
        assert_eq!(yaml_as_str(result.get("title").unwrap()).unwrap(), "Test Note");
        assert_eq!(yaml_as_str(result.get("source").unwrap()).unwrap(), "Eberron: Rising from the Last War p. 277");
        assert_eq!(yaml_as_str(result.get("book").unwrap()).unwrap(), "Player's Handbook: Chapter 3");
        assert_eq!(yaml_as_str(result.get("url").unwrap()).unwrap(), "https://example.com/path");
        
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
        assert_eq!(yaml_as_str(result_lenient.get("title").unwrap()).unwrap(), "Test Note");
        assert_eq!(yaml_as_str(result_lenient.get("source").unwrap()).unwrap(), "Eberron: Rising from the Last War p. 277");
        assert!(warning_lenient.is_some());
        assert!(warning_lenient.unwrap().contains("Used lenient parsing"));
    }

    #[test]
    fn test_case_insensitive_filtering() {
        let mut fm = HashMap::new();
        fm.insert("Tag".to_string(), Yaml::String("Work".to_string()));
        fm.insert("Status".to_string(), Yaml::String("Active".to_string()));
        fm.insert("Priority".to_string(), Yaml::Array(vec![
            Yaml::String("High".to_string()),
            Yaml::String("Urgent".to_string()),
        ]));
        
        let note = Note::new("test.md".to_string(), fm);
        
        // Test case-sensitive matching (should fail)
        assert!(!note.matches_filter("tag", "Work")); // field name case mismatch
        assert!(!note.matches_filter("Tag", "work")); // value case mismatch
        
        // Test case-insensitive matching (should succeed)
        assert!(note.matches_filter_with_case_sensitivity("tag", "work", false)); // both case mismatches
        assert!(note.matches_filter_with_case_sensitivity("TAG", "WORK", false)); // both uppercase
        assert!(note.matches_filter_with_case_sensitivity("Status", "active", false)); // value case mismatch
        assert!(note.matches_filter_with_case_sensitivity("priority", "high", false)); // array value case mismatch
        
        // Test that case-sensitive mode still works
        assert!(note.matches_filter_with_case_sensitivity("Tag", "Work", true)); // exact match
        assert!(!note.matches_filter_with_case_sensitivity("tag", "Work", true)); // field name case mismatch
    }

    #[test]
    fn test_case_insensitive_field_lookup() {
        let mut fm = HashMap::new();
        fm.insert("Title".to_string(), Yaml::String("Test Note".to_string()));
        fm.insert("TAG".to_string(), Yaml::String("work".to_string()));
        fm.insert("status".to_string(), Yaml::String("active".to_string()));
        
        let note = Note::new("test.md".to_string(), fm);
        
        // Test exact matches
        assert!(note.get_frontmatter_value("Title").is_some());
        assert!(note.get_frontmatter_value("TAG").is_some());
        assert!(note.get_frontmatter_value("status").is_some());
        
        // Test case-insensitive lookup
        assert!(note.get_frontmatter_value_case_insensitive("title").is_some());
        assert!(note.get_frontmatter_value_case_insensitive("TITLE").is_some());
        assert!(note.get_frontmatter_value_case_insensitive("tag").is_some());
        assert!(note.get_frontmatter_value_case_insensitive("Tag").is_some());
        assert!(note.get_frontmatter_value_case_insensitive("STATUS").is_some());
        assert!(note.get_frontmatter_value_case_insensitive("Status").is_some());
        
        // Test non-existent field
        assert!(note.get_frontmatter_value_case_insensitive("nonexistent").is_none());
        
        // Verify values are correct
        if let Some(Yaml::String(title)) = note.get_frontmatter_value_case_insensitive("title") {
            assert_eq!(title, "Test Note");
        } else {
            panic!("Expected string value for title");
        }
    }
}