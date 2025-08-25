use crate::frontmatter::{parse_frontmatter_from_file, Note};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct VaultScanner {
    vault_path: PathBuf,
}

impl VaultScanner {
    pub fn new<P: AsRef<Path>>(vault_path: P) -> Result<Self> {
        let vault_path = vault_path.as_ref().to_path_buf();
        
        if !vault_path.exists() {
            return Err(anyhow::anyhow!(
                "Vault path does not exist: {}",
                vault_path.display()
            ));
        }

        if !vault_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Vault path is not a directory: {}",
                vault_path.display()
            ));
        }

        Ok(Self { vault_path })
    }

    pub fn scan_vault(&self) -> Result<Vec<Note>> {
        println!("Scanning vault: {}", self.vault_path.display());
        
        // Find all markdown files
        let markdown_files: Vec<PathBuf> = WalkDir::new(&self.vault_path)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                
                // Skip hidden files and directories
                if path.file_name()?.to_str()?.starts_with('.') {
                    return None;
                }
                
                // Only process markdown files
                if path.extension()?.to_str()? == "md" {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
            .collect();

        println!("Found {} markdown files", markdown_files.len());

        // Process files in parallel
        let notes: Vec<Note> = markdown_files
            .par_iter()
            .filter_map(|path| {
                match parse_frontmatter_from_file(path) {
                    Ok(Some(note)) => Some(note),
                    Ok(None) => None,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        println!("Successfully parsed {} notes", notes.len());
        Ok(notes)
    }

    pub fn get_vault_path(&self) -> &Path {
        &self.vault_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_creation() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = VaultScanner::new(temp_dir.path()).unwrap();
        assert_eq!(scanner.get_vault_path(), temp_dir.path());
    }

    #[test]
    fn test_scanner_nonexistent_path() {
        let result = VaultScanner::new("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = VaultScanner::new(temp_dir.path()).unwrap();
        let notes = scanner.scan_vault().unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn test_scan_vault_with_markdown() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test markdown file
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, r#"---
title: Test Note
tags: [test]
---

# Test Content
"#).unwrap();

        let scanner = VaultScanner::new(temp_dir.path()).unwrap();
        let notes = scanner.scan_vault().unwrap();
        
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, Some("Test Note".to_string()));
    }
}