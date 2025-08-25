use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod frontmatter;
mod scanner;
mod filter;
mod output;
mod logger;

use crate::scanner::VaultScanner;
use crate::filter::FilterCriteria;
use crate::frontmatter::Note;

#[derive(Parser)]
#[command(name = "aktenfux")]
#[command(about = "A CLI tool for indexing and filtering Obsidian vault notes by frontmatter")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Filter notes by frontmatter fields
    Filter {
        /// Path to the Obsidian vault (defaults to current directory)
        #[arg(default_value = ".")]
        vault_path: PathBuf,
        /// Filter by field=value pairs (can be used multiple times)
        #[arg(long, value_parser = parse_filter)]
        filter: Vec<(String, String)>,
        /// Output format: table, paths, json
        #[arg(short, long, default_value = "table")]
        format: String,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Use strict YAML parsing (disable lenient parsing for frontmatter with colons)
        #[arg(long)]
        strict: bool,
    },
    /// List all available frontmatter fields in the vault
    Fields {
        /// Path to the Obsidian vault (defaults to current directory)
        #[arg(default_value = ".")]
        vault_path: PathBuf,
        /// Filter by field=value pairs (can be used multiple times)
        #[arg(long, value_parser = parse_filter)]
        filter: Vec<(String, String)>,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Use strict YAML parsing (disable lenient parsing for frontmatter with colons)
        #[arg(long)]
        strict: bool,
    },
    /// List all values for a specific frontmatter field
    Values {
        /// Path to the Obsidian vault (defaults to current directory)
        #[arg(default_value = ".")]
        vault_path: PathBuf,
        /// The field to list values for
        #[arg(short, long)]
        field: String,
        /// Filter by field=value pairs (can be used multiple times)
        #[arg(long, value_parser = parse_filter)]
        filter: Vec<(String, String)>,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Use strict YAML parsing (disable lenient parsing for frontmatter with colons)
        #[arg(long)]
        strict: bool,
    },
}

fn parse_filter(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid filter format: '{}'. Use field=value", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Filter { vault_path, filter, format, verbose, strict } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, !strict)?;
            
            let criteria = FilterCriteria::new(filter);
            let filtered_notes = criteria.apply_filters(&notes);
            
            output::display_filtered_results(&filtered_notes, &format)?;
        }
        Commands::Fields { vault_path, filter, verbose, strict } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, !strict)?;
            
            let criteria = FilterCriteria::new(filter);
            let filtered_notes = criteria.apply_filters(&notes);
            
            // Convert Vec<&Note> back to Vec<Note> for display_all_fields
            let filtered_notes_owned: Vec<Note> = filtered_notes.into_iter().cloned().collect();
            
            output::display_all_fields(&filtered_notes_owned)?;
        }
        Commands::Values { vault_path, field, filter, verbose, strict } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, !strict)?;
            
            let criteria = FilterCriteria::new(filter);
            let filtered_notes = criteria.apply_filters(&notes);
            
            // Convert Vec<&Note> back to Vec<Note> for display_field_values
            let filtered_notes_owned: Vec<Note> = filtered_notes.into_iter().cloned().collect();
            
            output::display_field_values(&filtered_notes_owned, &field)?;
        }
    }

    Ok(())
}