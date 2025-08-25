# Aktenfux

A fast CLI tool for indexing and filtering Obsidian vault notes by their frontmatter metadata.

## Features

- 🚀 **Fast parallel processing** - Scans large vaults quickly using all CPU cores
- 🔍 **Flexible filtering** - Filter notes by any frontmatter field and value
- 📊 **Field analysis** - List all available frontmatter fields across your vault
- 📈 **Value statistics** - See all values for specific fields with usage counts
- 🎯 **Multiple output formats** - Table, paths-only, or JSON output
- 🏃 **No indexing required** - Scans vault on each run (perfect for dynamic vaults)
- 🔧 **Single binary** - Easy deployment and distribution

## Installation

### Using Nix (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd aktenfux

# Enter development environment
nix develop

# Build the project
cargo build --release

# The binary will be available at target/release/aktenfux
```

### Using Cargo

```bash
# Clone the repository
git clone <repository-url>
cd aktenfux

# Build the project
cargo build --release

# The binary will be available at target/release/aktenfux
```

## Usage

### Basic Commands

```bash
# List all available frontmatter fields in your vault
aktenfux fields [vault_path] [--filter=<field>=<value>] [--verbose] [--strict]

# List all values for a specific field
aktenfux values [vault_path] --field=<field_name> [--filter=<field>=<value>] [--verbose] [--strict]

# Filter notes by frontmatter
aktenfux filter [vault_path] --filter=<field>=<value> [--verbose] [--strict]
```

If no `vault_path` is provided, the current directory is used.

### Verbose Output

All commands support a `--verbose` (or `-v`) flag for detailed output:

- **Default mode**: Shows a summary of parsing errors (e.g., "Skipped 5 files due to frontmatter parsing errors")
- **Verbose mode**: Shows detailed error messages with specific file paths and error descriptions

```bash
# Show detailed error information
aktenfux filter --verbose
aktenfux fields --verbose
aktenfux values --field=tags --verbose
```

This is particularly useful when troubleshooting frontmatter parsing issues in large vaults.

### Examples

#### List all frontmatter fields
```bash
aktenfux fields ~/Documents/ObsidianVault
```

Output:
```
Available frontmatter fields:

Field         Notes   Values
----------------------------
author            15        3
status            42        5
tags              38       12
title             45        45
priority           8        3

Total: 5 unique fields across 45 notes
```

#### List fields from filtered notes
```bash
# Show only fields from notes tagged as "work"
aktenfux fields ~/Documents/ObsidianVault --filter=tags=work
```

Output:
```
Available frontmatter fields:

Field         Notes   Values
----------------------------
author             8        2
status            12        3
tags              12        8
title             12       12
priority           5        3

Total: 5 unique fields across 12 notes
```

```bash
# Show fields from D&D monster notes
aktenfux fields ~/Documents/DnDVault --filter=type=Monster
```

#### List all values for the "tags" field
```bash
aktenfux values ~/Documents/ObsidianVault --field=tags
```

Output:
```
Values for field 'tags':

Value         Count
--------------------
work             12
personal          8
project           6
meeting           4
idea              3

Total: 5 unique values, 33 total occurrences
```

#### List values from filtered notes
```bash
# Show status values only from work-tagged notes
aktenfux values ~/Documents/ObsidianVault --field=status --filter=tags=work
```

Output:
```
Values for field 'status':

Value         Count
--------------------
active            8
completed         4

Total: 2 unique values, 12 total occurrences
```

```bash
# Show tag values only from notes by specific author
aktenfux values ~/Documents/DnDVault --field=tags --filter=author=DM
```

#### Filter notes by tag
```bash
aktenfux filter ~/Documents/ObsidianVault --filter=tags=work
```

Output:
```
Found 12 matching notes:

Path                           Title                    Frontmatter
------------------------------------------------------------------------
notes/project-alpha.md        Project Alpha Planning   title, tags, status, priority
notes/meeting-notes-2024.md   Weekly Team Meeting      title, tags, author, date
...
```

#### Multiple filters (AND logic)
```bash
aktenfux filter ~/Documents/ObsidianVault --filter=tags=work --filter=status=active
```

#### Different output formats
```bash
# Paths only (great for piping to other tools)
aktenfux filter ~/Documents/ObsidianVault --filter=tags=work --format=paths

# JSON output (for programmatic processing)
aktenfux filter ~/Documents/ObsidianVault --filter=tags=work --format=json
```

### Output Formats

- **table** (default): Human-readable table with path, title, and frontmatter summary
- **paths**: File paths only, one per line
- **json**: Complete JSON output with all frontmatter data

## Frontmatter Support

Aktenfux supports YAML frontmatter in the standard format:

```markdown
---
title: My Note
tags: [work, important]
status: active
author: John Doe
priority: high
due_date: 2024-12-31
---

# My Note Content

Your note content here...
```

### Supported Value Types

- **Strings**: `title: My Note`
- **Arrays**: `tags: [work, personal, project]`
- **Numbers**: `priority: 1`
- **Booleans**: `published: true`
- **Dates**: `due_date: 2024-12-31`

### Lenient Frontmatter Parsing

Aktenfux includes **lenient parsing** to handle common YAML frontmatter issues:

- **Values with colons**: Automatically handles values like `source: Eberron: Rising from the Last War p. 277`
- **URLs**: Works with `url: https://example.com/path` without requiring quotes
- **Book references**: Handles `book: Player's Handbook: Chapter 3` correctly

By default, Aktenfux uses lenient parsing which automatically quotes problematic values. If you need strict YAML compliance, use the `--strict` flag:

```bash
# Default lenient parsing (recommended)
aktenfux filter --filter=source=Eberron

# Strict YAML parsing (may fail on unquoted values with colons)
aktenfux filter --filter=source=Eberron --strict
```

**Example problematic frontmatter that works with lenient parsing:**
```markdown
---
title: D&D Reference Note
source: Eberron: Rising from the Last War p. 277
book: Player's Handbook: Chapter 3
url: https://example.com/path
tags: [dnd, reference]
---
```

With lenient parsing, this will be automatically converted to valid YAML internally and parsed successfully.

## Performance

Aktenfux is designed for speed:

- **Parallel processing**: Uses all available CPU cores
- **Memory efficient**: Streams file processing
- **Fast filtering**: Hash-based lookups
- **No persistent index**: Always up-to-date with your vault

Typical performance on a modern laptop:
- ~1000 notes: < 1 second
- ~10000 notes: < 5 seconds

## Use Cases

### Daily Workflow
```bash
# Quick overview of your vault structure
aktenfux fields

# See what fields are available in your work notes
aktenfux fields --filter=tags=work

# Find all active work items
aktenfux filter --filter=tags=work --filter=status=active --format=paths

# See all your project statuses
aktenfux values --field=status

# See status values only from work notes
aktenfux values --field=status --filter=tags=work
```

### Integration with Other Tools
```bash
# Open all work notes in your editor
aktenfux filter --filter=tags=work --format=paths | xargs code

# Count notes by status
aktenfux filter --filter=status=completed --format=paths | wc -l

# Export work notes metadata
aktenfux filter --filter=tags=work --format=json > work-notes.json
```

### Vault Analysis
```bash
# Analyze your tagging system
aktenfux values --field=tags

# Find notes missing certain metadata
aktenfux filter --filter=status= --format=paths  # Notes without status

# Get overview of all metadata fields
aktenfux fields

# Analyze metadata structure for specific note types
aktenfux fields --filter=type=Monster  # D&D monster notes
aktenfux fields --filter=tags=project  # Project notes only

# Analyze value distributions within filtered subsets
aktenfux values --field=status --filter=tags=work    # Work note statuses
aktenfux values --field=cr --filter=type=Monster     # D&D monster challenge ratings
```

## Development

### Requirements

- Rust 1.70+
- Nix (optional, for development environment)

### Building from Source

```bash
git clone <repository-url>
cd aktenfux

# Using Nix
nix develop
cargo build --release

# Or using Cargo directly
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Development Environment

The project includes a Nix flake for easy development:

```bash
nix develop
# This provides Rust toolchain, cargo-watch, rust-analyzer, etc.
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Changelog

### v0.1.3
- **Lenient Frontmatter Parsing**: Added automatic handling of YAML frontmatter with colons in values
- **Strict Mode Option**: Added `--strict` flag to disable lenient parsing when needed
- **Better YAML Compatibility**: Automatically quotes problematic values like `source: Eberron: Rising from the Last War p. 277`
- **Enhanced Error Messages**: Distinguishes between lenient parsing warnings and actual parsing failures

### v0.1.2
- **Improved Error Handling**: Added `--verbose`/`-v` flag to all commands
- **Better Error Messages**: Frontmatter parsing errors now include specific file paths
- **Summary Output**: Default mode shows error counts by category instead of individual warnings
- **Cleaner Output**: Reduced noise for large vaults while maintaining debugging capability
- **Enhanced Logging**: Centralized error reporting system with categorized error types

### v0.1.1
- Added filtering capability to `fields` command
- Added filtering capability to `values` command
- Can now filter field analysis by frontmatter criteria (e.g., `aktenfux fields --filter=type=Monster`)
- Can now filter value analysis by frontmatter criteria (e.g., `aktenfux values --field=status --filter=tags=work`)
- Supports multiple filters with AND logic for both commands
- Maintains same filtering syntax as existing `filter` command

### v0.1.0
- Initial release
- Basic frontmatter parsing and filtering
- Parallel processing support
- Multiple output formats
- Field and value analysis commands