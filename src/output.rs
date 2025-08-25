use crate::frontmatter::Note;
use crate::filter::{collect_all_fields, collect_field_values, get_field_statistics};
use anyhow::Result;
use colored::*;
use serde_json;

pub fn display_filtered_results(notes: &[&Note], format: &str) -> Result<()> {
    match format.to_lowercase().as_str() {
        "table" => display_table_format(notes),
        "paths" => display_paths_format(notes),
        "json" => display_json_format(notes),
        _ => {
            eprintln!("Unknown format: {}. Using table format.", format);
            display_table_format(notes)
        }
    }
}

pub fn display_all_fields(notes: &[Note]) -> Result<()> {
    let fields = collect_all_fields(notes);
    let stats = get_field_statistics(notes);
    
    if fields.is_empty() {
        println!("{}", "No frontmatter fields found in any notes.".yellow());
        return Ok(());
    }
    
    println!("{}", "Available frontmatter fields:".bold().blue());
    println!();
    
    // Calculate column widths
    let max_field_width = fields.iter().map(|f| f.len()).max().unwrap_or(0);
    let field_width = std::cmp::max(max_field_width, 10);
    
    // Header
    println!(
        "{:<width$} {:>8} {:>8}",
        "Field".bold(),
        "Notes".bold(),
        "Values".bold(),
        width = field_width
    );
    println!("{}", "-".repeat(field_width + 18));
    
    // Field data
    for field in &fields {
        let field_stats = stats.get(field).unwrap();
        println!(
            "{:<width$} {:>8} {:>8}",
            field.green(),
            field_stats.total_count,
            field_stats.unique_values.len(),
            width = field_width
        );
    }
    
    println!();
    println!("Total: {} unique fields across {} notes", fields.len(), notes.len());
    
    Ok(())
}

pub fn display_field_values(notes: &[Note], field: &str) -> Result<()> {
    let values = collect_field_values(notes, field);
    let stats = get_field_statistics(notes);
    
    if values.is_empty() {
        println!("{}", format!("No values found for field '{}'.", field).yellow());
        return Ok(());
    }
    
    println!("{}", format!("Values for field '{}':", field).bold().blue());
    println!();
    
    if let Some(field_stats) = stats.get(field) {
        // Calculate column width
        let max_value_width = values.iter().map(|v| v.len()).max().unwrap_or(0);
        let value_width = std::cmp::max(max_value_width, 10);
        
        // Header
        println!(
            "{:<width$} {:>8}",
            "Value".bold(),
            "Count".bold(),
            width = value_width
        );
        println!("{}", "-".repeat(value_width + 10));
        
        // Sort values by count (descending)
        let mut value_counts: Vec<_> = field_stats.value_counts.iter().collect();
        value_counts.sort_by(|a, b| b.1.cmp(a.1));
        
        for (value, count) in value_counts {
            println!(
                "{:<width$} {:>8}",
                value.green(),
                count,
                width = value_width
            );
        }
        
        println!();
        println!(
            "Total: {} unique values, {} total occurrences",
            values.len(),
            field_stats.total_count
        );
    } else {
        // Fallback if stats are not available
        for value in &values {
            println!("  {}", value.green());
        }
        println!();
        println!("Total: {} unique values", values.len());
    }
    
    Ok(())
}

fn display_table_format(notes: &[&Note]) -> Result<()> {
    if notes.is_empty() {
        println!("{}", "No notes match the specified criteria.".yellow());
        return Ok(());
    }
    
    println!("{}", format!("Found {} matching notes:", notes.len()).bold().blue());
    println!();
    
    // Calculate column widths
    let max_path_width = notes.iter().map(|n| n.path.len()).max().unwrap_or(0);
    let max_title_width = notes.iter()
        .map(|n| n.title.as_ref().map(|t| t.len()).unwrap_or(0))
        .max()
        .unwrap_or(0);
    
    let path_width = std::cmp::min(max_path_width, 50);
    let title_width = std::cmp::min(max_title_width, 30);
    
    // Header
    println!(
        "{:<path_width$} {:<title_width$} {}",
        "Path".bold(),
        "Title".bold(),
        "Frontmatter".bold(),
        path_width = path_width,
        title_width = title_width
    );
    println!("{}", "-".repeat(path_width + title_width + 20));
    
    // Note data
    for note in notes {
        let path = if note.path.len() > path_width {
            format!("...{}", &note.path[note.path.len() - path_width + 3..])
        } else {
            note.path.clone()
        };
        
        let title = note.title.as_ref()
            .map(|t| if t.len() > title_width {
                format!("{}...", &t[..title_width - 3])
            } else {
                t.clone()
            })
            .unwrap_or_else(|| "-".to_string());
        
        let frontmatter_summary = if note.frontmatter.is_empty() {
            "-".to_string()
        } else {
            let keys: Vec<String> = note.frontmatter.keys().cloned().collect();
            if keys.len() <= 3 {
                keys.join(", ")
            } else {
                format!("{}, ... (+{})", keys[..3].join(", "), keys.len() - 3)
            }
        };
        
        println!(
            "{:<path_width$} {:<title_width$} {}",
            path.cyan(),
            title.green(),
            frontmatter_summary.dimmed(),
            path_width = path_width,
            title_width = title_width
        );
    }
    
    Ok(())
}

fn display_paths_format(notes: &[&Note]) -> Result<()> {
    if notes.is_empty() {
        println!("{}", "No notes match the specified criteria.".yellow());
        return Ok(());
    }
    
    for note in notes {
        println!("{}", note.path);
    }
    
    Ok(())
}

fn display_json_format(notes: &[&Note]) -> Result<()> {
    let json_output = serde_json::to_string_pretty(notes)?;
    println!("{}", json_output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use serde_yaml::Value;

    fn create_test_note(path: &str, title: Option<&str>, frontmatter: HashMap<String, Value>) -> Note {
        let mut note = Note::new(path.to_string(), frontmatter);
        if let Some(t) = title {
            note.title = Some(t.to_string());
        }
        note
    }

    #[test]
    fn test_display_paths_format() {
        let mut fm = HashMap::new();
        fm.insert("tag".to_string(), Value::String("test".to_string()));
        
        let notes = vec![
            create_test_note("note1.md", Some("Note 1"), fm.clone()),
            create_test_note("note2.md", Some("Note 2"), fm),
        ];
        
        let note_refs: Vec<&Note> = notes.iter().collect();
        
        // This would normally print to stdout, but we can't easily test that
        // Just ensure it doesn't panic
        assert!(display_paths_format(&note_refs).is_ok());
    }
}