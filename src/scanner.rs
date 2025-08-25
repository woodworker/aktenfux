use crate::frontmatter::{parse_frontmatter_from_file, Note, ParseResult};
use crate::logger::Logger;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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

    pub fn scan_vault(&self, verbose: bool, lenient: bool) -> Result<Vec<Note>> {
        let mut logger = Logger::new(verbose);
        
        logger.log_info(
            format!("Scanning vault: {}", self.vault_path.display()),
            None::<&Path>,
        );
        
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

        logger.log_info(
            format!("Found {} markdown files", markdown_files.len()),
            None::<&Path>,
        );

        // Use Arc<Mutex<Logger>> for thread-safe logging
        let logger = Arc::new(Mutex::new(logger));

        // Process files in parallel
        let notes: Vec<Note> = markdown_files
            .par_iter()
            .filter_map(|path| {
                match parse_frontmatter_from_file(path, verbose, lenient) {
                    Ok(ParseResult { note, frontmatter_warning }) => {
                        // Log frontmatter warnings if present
                        if let Some(warning) = frontmatter_warning {
                            if let Ok(mut logger) = logger.lock() {
                                logger.log_warning(warning, Some(path));
                            }
                        }
                        note
                    }
                    Err(e) => {
                        if let Ok(mut logger) = logger.lock() {
                            logger.log_critical(
                                format!("Failed to parse file: {}", e),
                                Some(path),
                            );
                        }
                        None
                    }
                }
            })
            .collect();

        // Extract logger from Arc<Mutex<>> for final summary
        let logger = Arc::try_unwrap(logger)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap logger"))?
            .into_inner()
            .map_err(|_| anyhow::anyhow!("Failed to extract logger from mutex"))?;

        logger.print_summary(markdown_files.len(), notes.len());
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
        let notes = scanner.scan_vault(false, true).unwrap();
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
        let notes = scanner.scan_vault(false, true).unwrap();
        
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, Some("Test Note".to_string()));
    }
}