use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum ErrorLevel {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    #[allow(dead_code)]
    pub level: ErrorLevel,
    #[allow(dead_code)]
    pub message: String,
    #[allow(dead_code)]
    pub file_path: Option<String>,
}

#[derive(Debug)]
pub struct Logger {
    verbose: bool,
    entries: Vec<LogEntry>,
    error_counts: HashMap<String, usize>,
    lenient_parsing_count: usize,
}

impl Logger {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            entries: Vec::new(),
            error_counts: HashMap::new(),
            lenient_parsing_count: 0,
        }
    }

    pub fn log_critical<P: AsRef<Path>>(&mut self, message: String, file_path: Option<P>) {
        let file_path_str = file_path.map(|p| p.as_ref().to_string_lossy().to_string());
        let entry = LogEntry {
            level: ErrorLevel::Critical,
            message: message.clone(),
            file_path: file_path_str.clone(),
        };
        
        // Critical errors are always shown
        if let Some(path) = &file_path_str {
            eprintln!("Error: {} ({})", message, path);
        } else {
            eprintln!("Error: {}", message);
        }
        
        self.entries.push(entry);
    }

    pub fn log_warning<P: AsRef<Path>>(&mut self, message: String, file_path: Option<P>) {
        let file_path_str = file_path.map(|p| p.as_ref().to_string_lossy().to_string());
        let entry = LogEntry {
            level: ErrorLevel::Warning,
            message: message.clone(),
            file_path: file_path_str.clone(),
        };

        // Count warnings by type, but handle lenient parsing separately
        if message.contains("Used lenient parsing") {
            self.lenient_parsing_count += 1;
        } else {
            let warning_type = extract_warning_type(&message);
            *self.error_counts.entry(warning_type).or_insert(0) += 1;
        }

        // Show warnings only in verbose mode
        if self.verbose {
            if let Some(path) = &file_path_str {
                eprintln!("Warning: {} ({})", message, path);
            } else {
                eprintln!("Warning: {}", message);
            }
        }
        
        self.entries.push(entry);
    }

    pub fn log_info<P: AsRef<Path>>(&mut self, message: String, file_path: Option<P>) {
        let file_path_str = file_path.map(|p| p.as_ref().to_string_lossy().to_string());
        let entry = LogEntry {
            level: ErrorLevel::Info,
            message: message.clone(),
            file_path: file_path_str,
        };

        // Show info only in verbose mode
        if self.verbose {
            println!("{}", message);
        }
        
        self.entries.push(entry);
    }

    pub fn print_summary(&self, _total_files: usize, successful_files: usize) {
        println!("Successfully parsed {} notes", successful_files);
        
        // Show lenient parsing info if any files were fixed
        if self.lenient_parsing_count > 0 {
            println!("Fixed {} files with lenient parsing (frontmatter with colons in values)", self.lenient_parsing_count);
        }
        
        // Show actual parsing errors (files that were skipped)
        if !self.error_counts.is_empty() {
            let total_errors: usize = self.error_counts.values().sum();
            if total_errors > 0 {
                println!("Skipped {} files due to parsing errors:", total_errors);
                for (error_type, count) in &self.error_counts {
                    println!("  - {}: {} files", error_type, count);
                }
                if !self.verbose {
                    println!("Use --verbose/-v to see detailed error messages");
                }
            }
        }
    }

    #[cfg(test)]
    pub fn get_warning_count(&self) -> usize {
        self.entries.iter()
            .filter(|entry| matches!(entry.level, ErrorLevel::Warning))
            .count()
    }

    #[cfg(test)]
    pub fn get_critical_count(&self) -> usize {
        self.entries.iter()
            .filter(|entry| matches!(entry.level, ErrorLevel::Critical))
            .count()
    }
}

fn extract_warning_type(message: &str) -> String {
    if message.contains("frontmatter") {
        "Frontmatter parsing errors".to_string()
    } else if message.contains("Failed to parse") {
        "File parsing errors".to_string()
    } else if message.contains("Failed to read") {
        "File read errors".to_string()
    } else {
        "Other errors".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_verbose_mode() {
        let mut logger = Logger::new(true);
        
        // Test warning logging
        logger.log_warning("Test warning".to_string(), Some("test.md"));
        
        assert_eq!(logger.get_warning_count(), 1);
        assert_eq!(logger.get_critical_count(), 0);
    }

    #[test]
    fn test_logger_quiet_mode() {
        let mut logger = Logger::new(false);
        
        // Test warning logging (should be counted but not displayed in verbose mode)
        logger.log_warning("Test warning".to_string(), Some("test.md"));
        
        assert_eq!(logger.get_warning_count(), 1);
        assert_eq!(logger.get_critical_count(), 0);
    }

    #[test]
    fn test_error_categorization() {
        let mut logger = Logger::new(false);
        
        logger.log_warning("Failed to parse frontmatter".to_string(), Some("test1.md"));
        logger.log_warning("Failed to parse file".to_string(), Some("test2.md"));
        logger.log_warning("Failed to read file".to_string(), Some("test3.md"));
        logger.log_warning("Some other error".to_string(), Some("test4.md"));
        
        // Check that error counts are tracked
        assert_eq!(logger.get_warning_count(), 4);
        
        // Check that error counts by type are tracked
        assert_eq!(logger.error_counts.len(), 4);
    }

    #[test]
    fn test_critical_errors_always_shown() {
        let mut logger = Logger::new(false); // Non-verbose mode
        
        // Critical errors should always be shown regardless of verbose setting
        logger.log_critical("Critical error occurred".to_string(), Some("test.md"));
        
        assert_eq!(logger.get_critical_count(), 1);
        assert_eq!(logger.get_warning_count(), 0);
    }

    #[test]
    fn test_extract_warning_type() {
        assert_eq!(extract_warning_type("Failed to parse frontmatter"), "Frontmatter parsing errors");
        assert_eq!(extract_warning_type("Failed to parse file"), "File parsing errors");
        assert_eq!(extract_warning_type("Failed to read file"), "File read errors");
        assert_eq!(extract_warning_type("Unknown error"), "Other errors");
    }

    #[test]
    fn test_logger_error_counts() {
        let mut logger = Logger::new(false);
        
        // Add multiple warnings of the same type
        logger.log_warning("Failed to parse frontmatter in file1".to_string(), Some("test1.md"));
        logger.log_warning("Failed to parse frontmatter in file2".to_string(), Some("test2.md"));
        
        // Should have 2 warnings total
        assert_eq!(logger.get_warning_count(), 2);
        
        // Should have 1 error type with count of 2
        assert_eq!(logger.error_counts.len(), 1);
        assert_eq!(logger.error_counts.get("Frontmatter parsing errors"), Some(&2));
    }

    #[test]
    fn test_lenient_parsing_tracking() {
        let mut logger = Logger::new(false);
        
        // Add lenient parsing warnings
        logger.log_warning("Used lenient parsing for frontmatter in file test.md due to: mapping values are not allowed".to_string(), Some("test.md"));
        logger.log_warning("Used lenient parsing for frontmatter in file test2.md due to: mapping values are not allowed".to_string(), Some("test2.md"));
        
        // Add regular parsing error
        logger.log_warning("Failed to parse frontmatter in file3".to_string(), Some("test3.md"));
        
        // Should have 3 warnings total
        assert_eq!(logger.get_warning_count(), 3);
        
        // Should have 2 lenient parsing fixes
        assert_eq!(logger.lenient_parsing_count, 2);
        
        // Should have 1 actual error
        assert_eq!(logger.error_counts.len(), 1);
        assert_eq!(logger.error_counts.get("Frontmatter parsing errors"), Some(&1));
    }
}