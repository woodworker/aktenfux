use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod filter;
mod frontmatter;
mod logger;
mod output;
mod scanner;
mod yaml_compat;

use crate::filter::FilterCriteria;
use crate::frontmatter::Note;
use crate::scanner::VaultScanner;

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
        /// Enable case-insensitive matching for filters
        #[arg(short = 'i', long)]
        ignore_case: bool,
        /// Output format: table, paths, json
        #[arg(short, long, default_value = "table")]
        format: String,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Suppress all non-essential output (summary and info messages)
        #[arg(short, long)]
        silent: bool,
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
        /// Enable case-insensitive matching for filters
        #[arg(short = 'i', long)]
        ignore_case: bool,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Suppress all non-essential output (summary and info messages)
        #[arg(short, long)]
        silent: bool,
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
        /// Enable case-insensitive matching for field names and filters
        #[arg(short = 'i', long)]
        ignore_case: bool,
        /// Filter by field=value pairs (can be used multiple times)
        #[arg(long, value_parser = parse_filter)]
        filter: Vec<(String, String)>,
        /// Enable verbose output with detailed error messages
        #[arg(short, long)]
        verbose: bool,
        /// Suppress all non-essential output (summary and info messages)
        #[arg(short, long)]
        silent: bool,
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
        Commands::Filter {
            vault_path,
            filter,
            ignore_case,
            format,
            verbose,
            silent,
            strict,
        } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, silent, !strict, Some(&format))?;

            let criteria = if ignore_case {
                FilterCriteria::new_case_insensitive(filter)
            } else {
                FilterCriteria::new(filter)
            };
            let filtered_notes = criteria.apply_filters(&notes);

            output::display_filtered_results(&filtered_notes, &format, silent)?;
        }
        Commands::Fields {
            vault_path,
            filter,
            ignore_case,
            verbose,
            silent,
            strict,
        } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, silent, !strict, None)?;

            let criteria = if ignore_case {
                FilterCriteria::new_case_insensitive(filter)
            } else {
                FilterCriteria::new(filter)
            };
            let filtered_notes = criteria.apply_filters(&notes);

            // Convert Vec<&Note> back to Vec<Note> for display_all_fields
            let filtered_notes_owned: Vec<Note> = filtered_notes.into_iter().cloned().collect();

            output::display_all_fields(&filtered_notes_owned, silent)?;
        }
        Commands::Values {
            vault_path,
            field,
            ignore_case,
            filter,
            verbose,
            silent,
            strict,
        } => {
            let scanner = VaultScanner::new(vault_path)?;
            let notes = scanner.scan_vault(verbose, silent, !strict, None)?;

            let criteria = if ignore_case {
                FilterCriteria::new_case_insensitive(filter)
            } else {
                FilterCriteria::new(filter)
            };
            let filtered_notes = criteria.apply_filters(&notes);

            // Convert Vec<&Note> back to Vec<Note> for display_field_values
            let filtered_notes_owned: Vec<Note> = filtered_notes.into_iter().cloned().collect();

            output::display_field_values_with_options(
                &filtered_notes_owned,
                &field,
                !ignore_case,
                silent,
            )?;
        }
    }

    Ok(())
}
