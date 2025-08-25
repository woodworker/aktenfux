use yaml_rust2::{YamlLoader, Yaml};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json;

/// Compatibility wrapper for yaml-rust2 to match serde_yaml behavior
pub fn parse_yaml_frontmatter(content: &str) -> Result<HashMap<String, Yaml>> {
    let docs = YamlLoader::load_from_str(content)
        .map_err(|e| anyhow!("YAML parsing error: {}", e))?;
    
    if docs.is_empty() {
        return Ok(HashMap::new());
    }
    
    // Take the first document (frontmatter is single document)
    let doc = &docs[0];
    
    // Convert to string-keyed HashMap
    yaml_to_string_map(doc)
}

/// Convert Yaml::Hash to HashMap<String, Yaml> for string keys only
fn yaml_to_string_map(yaml: &Yaml) -> Result<HashMap<String, Yaml>> {
    match yaml {
        Yaml::Hash(hash) => {
            let mut result = HashMap::new();
            for (key, value) in hash {
                if let Yaml::String(key_str) = key {
                    result.insert(key_str.clone(), value.clone());
                }
                // Skip non-string keys (shouldn't happen in frontmatter)
            }
            Ok(result)
        }
        Yaml::Null => Ok(HashMap::new()), // Empty document
        _ => Err(anyhow!("Expected hash or null at document root, got {:?}", yaml)),
    }
}

/// Helper function to get string value from Yaml (equivalent to serde_yaml::Value::as_str)
pub fn yaml_as_str(yaml: &Yaml) -> Option<&str> {
    match yaml {
        Yaml::String(s) => Some(s),
        _ => None,
    }
}

/// Helper function to check if Yaml contains a string value (replaces serde_yaml pattern matching)
pub fn yaml_contains_str(yaml: &Yaml, search: &str) -> bool {
    match yaml {
        Yaml::String(s) => s.contains(search),
        Yaml::Array(arr) => {
            arr.iter().any(|item| yaml_contains_str(item, search))
        }
        Yaml::Integer(n) => n.to_string().contains(search),
        Yaml::Real(f) => f.to_string().contains(search),
        Yaml::Boolean(b) => b.to_string().contains(search),
        _ => false,
    }
}

/// Convert Yaml to string representation for display/comparison
pub fn yaml_to_string(yaml: &Yaml) -> String {
    match yaml {
        Yaml::String(s) => s.clone(),
        Yaml::Integer(n) => n.to_string(),
        Yaml::Real(f) => f.to_string(),
        Yaml::Boolean(b) => b.to_string(),
        Yaml::Null => "null".to_string(),
        _ => format!("{:?}", yaml),
    }
}

/// Helper to collect string values from Yaml (for arrays and single values)
pub fn collect_yaml_strings(yaml: &Yaml) -> Vec<String> {
    match yaml {
        Yaml::String(s) => vec![s.clone()],
        Yaml::Array(arr) => {
            arr.iter()
                .filter_map(|item| {
                    if let Yaml::String(s) = item {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect()
        }
        Yaml::Integer(n) => vec![n.to_string()],
        Yaml::Real(f) => vec![f.to_string()],
        Yaml::Boolean(b) => vec![b.to_string()],
        _ => vec![],
    }
}

/// Convert Yaml to serde_json::Value for JSON serialization
pub fn yaml_to_json_value(yaml: &Yaml) -> serde_json::Value {
    match yaml {
        Yaml::String(s) => serde_json::Value::String(s.clone()),
        Yaml::Integer(n) => serde_json::Value::Number((*n).into()),
        Yaml::Real(f) => {
            // Parse the string representation of the float
            if let Ok(float_val) = f.parse::<f64>() {
                serde_json::Number::from_f64(float_val)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        },
        Yaml::Boolean(b) => serde_json::Value::Bool(*b),
        Yaml::Array(arr) => serde_json::Value::Array(
            arr.iter().map(yaml_to_json_value).collect()
        ),
        Yaml::Hash(hash) => {
            let mut map = serde_json::Map::new();
            for (k, v) in hash {
                if let Yaml::String(key) = k {
                    map.insert(key.clone(), yaml_to_json_value(v));
                }
            }
            serde_json::Value::Object(map)
        }
        Yaml::Null => serde_json::Value::Null,
        _ => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_frontmatter() {
        let content = r#"
title: Test Note
tags: [work, important]
status: active
"#;
        let result = parse_yaml_frontmatter(content).unwrap();
        assert_eq!(result.len(), 3);
        assert!(matches!(result.get("title"), Some(Yaml::String(_))));
        assert!(matches!(result.get("tags"), Some(Yaml::Array(_))));
        assert!(matches!(result.get("status"), Some(Yaml::String(_))));
    }

    #[test]
    fn test_yaml_contains_str() {
        let yaml_string = Yaml::String("test value".to_string());
        assert!(yaml_contains_str(&yaml_string, "test"));
        assert!(!yaml_contains_str(&yaml_string, "missing"));

        let yaml_array = Yaml::Array(vec![
            Yaml::String("first".to_string()),
            Yaml::String("second".to_string()),
        ]);
        assert!(yaml_contains_str(&yaml_array, "first"));
        assert!(!yaml_contains_str(&yaml_array, "missing"));
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "";
        let result = parse_yaml_frontmatter(content).unwrap();
        assert!(result.is_empty());
    }
}