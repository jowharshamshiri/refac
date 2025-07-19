---
layout: default
title: Usage Guide
---

# Usage Guide

This comprehensive guide covers all aspects of using the Refac Tools suite for file operations, string replacement, and workspace management.

## Tools Overview

### Refac - String Replacement Tool
```bash
refac <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

### Scrap - File Management Tool
```bash
scrap [FILE/DIR] [SUBCOMMAND] [OPTIONS]
```

### Unscrap - File Restoration Tool
```bash
unscrap [FILE/DIR] [OPTIONS]
```

### Verbump - Version Management Tool
```bash
verbump [SUBCOMMAND] [OPTIONS]
```

## Refac - String Replacement

### Command Syntax

```bash
refac <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

- `ROOT_DIR`: Directory to search in (use `.` for current directory)
- `OLD_STRING`: String to find and replace
- `NEW_STRING`: Replacement string

### Simple Examples

```bash
# Replace in current directory
refac . "oldname" "newname"

# Process specific directory
refac ./src "OldClass" "NewClass"

# Preview changes first (recommended)
refac . "oldname" "newname" --dry-run
```

## Operation Modes

Refac operates on two levels by default:

1. **Name Replacement**: Renames files and directories
2. **Content Replacement**: Replaces strings inside text files

### Mode Flags

Use these flags to limit the operation scope:

```bash
# Only rename files/directories (skip content)
refac . "oldname" "newname" --names-only

# Only replace content (skip renaming)
refac . "oldname" "newname" --content-only

# Only process files (skip directories)
refac . "oldname" "newname" --files-only

# Only process directories (skip files)
refac . "oldname" "newname" --dirs-only
```

## Safety Features

### Dry Run Mode

Always preview changes before applying them:

```bash
# See what would be changed
refac . "oldname" "newname" --dry-run

# Dry run with verbose output
refac . "oldname" "newname" --dry-run --verbose
```

### Backup Files

Create backups before modifying files:

```bash
# Create .bak files before modification
refac . "oldname" "newname" --backup
```

### Force Mode

Skip confirmation prompts:

```bash
# Apply changes without confirmation
refac . "oldname" "newname" --force
```

## Filtering Options

### Include Patterns

Process only files matching specific patterns:

```bash
# Only Rust files
refac . "oldname" "newname" --include "*.rs"

# Multiple patterns
refac . "oldname" "newname" --include "*.rs" --include "*.toml"

# Include hidden files
refac . "oldname" "newname" --include ".*"
```

### Exclude Patterns

Skip files matching specific patterns:

```bash
# Exclude log files
refac . "oldname" "newname" --exclude "*.log"

# Exclude multiple patterns
refac . "oldname" "newname" --exclude "target/*" --exclude "*.log"

# Exclude build directories
refac . "oldname" "newname" --exclude "node_modules/*" --exclude "dist/*"
```

### Combining Filters

```bash
# Include source files but exclude tests
refac ./src "oldname" "newname" \
  --include "*.rs" \
  --include "*.toml" \
  --exclude "*test*"
```

## Advanced Options

### Depth Control

Limit how deep the tool searches:

```bash
# Search only current directory (depth 1)
refac . "oldname" "newname" --max-depth 1

# Search up to 3 levels deep
refac . "oldname" "newname" --max-depth 3
```

### Threading

Control parallel processing:

```bash
# Use 8 threads for faster processing
refac . "oldname" "newname" --threads 8

# Auto-detect optimal thread count (default)
refac . "oldname" "newname" --threads 0
```

### Case Sensitivity

```bash
# Case-insensitive matching
refac . "oldname" "newname" --ignore-case
```

### Regex Patterns

Use regular expressions for complex patterns:

```bash
# Use regex patterns
refac . "old_\w+" "new_name" --regex

# Case-insensitive regex
refac . "old.*name" "newname" --regex --ignore-case
```

## Output Formats

### Human-Readable (Default)

```bash
refac . "oldname" "newname"
```

Shows colored output with progress bars and detailed information.

### JSON Output

```bash
refac . "oldname" "newname" --format json
```

Useful for scripting and automation:

```json
{
  "summary": {
    "content_changes": 15,
    "file_renames": 8,
    "directory_renames": 3,
    "total_changes": 26
  },
  "result": "success",
  "dry_run": false
}
```

### Plain Text

```bash
refac . "oldname" "newname" --format plain
```

Simple text output without colors or special formatting.

## Real-World Examples

### Code Refactoring

```bash
# Rename a class throughout a project
refac ./src "UserManager" "AccountManager" --dry-run

# Update variable names in specific files
refac ./src "oldVar" "newVar" --include "*.js" --include "*.ts"

# Rename function across codebase
refac . "calculateTotal" "computeSum" --content-only
```

### File Organization

```bash
# Rename draft files to final
refac ./docs "draft" "final" --names-only

# Update configuration URLs
refac ./config "old.example.com" "new.example.com" --content-only

# Rename project files
refac . "myproject" "newproject" --exclude "node_modules/*"
```

### Database Migration

```bash
# Update table names in SQL files
refac ./sql "old_table" "new_table" --include "*.sql"

# Update column references
refac ./src "old_column" "new_column" --include "*.py" --include "*.sql"
```

## Best Practices

### 1. Always Test First

```bash
# Preview changes with dry-run
refac . "oldname" "newname" --dry-run --verbose

# Check the output carefully before proceeding
refac . "oldname" "newname"
```

### 2. Use Version Control

```bash
# Commit before refactoring
git add .
git commit -m "Before refactoring: rename oldname to newname"

# Run refac
refac . "oldname" "newname"

# Review changes
git diff
```

### 3. Be Specific with Patterns

```bash
# Good: Specific file types
refac ./src "oldname" "newname" --include "*.rs" --include "*.toml"

# Better: Also exclude unwanted directories
refac ./src "oldname" "newname" \
  --include "*.rs" \
  --exclude "target/*" \
  --exclude "*.log"
```

### 4. Use Appropriate Modes

```bash
# When renaming files only
refac . "old_prefix" "new_prefix" --names-only

# When updating configuration values
refac ./config "old.server.com" "new.server.com" --content-only
```

### 5. Handle Large Projects

```bash
# Use more threads for large codebases
refac . "oldname" "newname" --threads 8

# Limit scope with patterns
refac . "oldname" "newname" --include "src/**" --exclude "tests/**"
```

## Troubleshooting

### No Changes Found

```bash
# Use verbose mode to see what's being processed
refac . "oldname" "newname" --dry-run --verbose

# Check if the string exists
grep -r "oldname" . --include="*.rs"

# Verify include/exclude patterns
refac . "oldname" "newname" --include "*" --dry-run
```

### Permission Errors

```bash
# Check file permissions
ls -la problematic_file

# Use sudo if necessary (be careful!)
sudo refac . "oldname" "newname"
```

### Binary Files

Refac automatically skips binary files for content replacement:

```bash
# Use verbose to see skipped files
refac . "oldname" "newname" --verbose
```

### Large Files

For very large files, increase available memory:

```bash
# Process in smaller batches
refac . "oldname" "newname" --max-depth 2
```

## Integration with Other Tools

### Git Hooks

Create a pre-commit hook to validate changes:

```bash
#!/bin/bash
# .git/hooks/pre-commit
refac . "debug_print" "logger.debug" --dry-run --format json | \
  jq '.summary.total_changes > 0' && \
  echo "Warning: debug prints found"
```

### CI/CD Pipelines

```bash
# Check for outdated patterns
refac . "old_api_url" "new_api_url" --dry-run --format json | \
  jq '.summary.total_changes > 0' && exit 1
```

### Scripts

```bash
#!/bin/bash
# bulk-rename.sh
PATTERNS=(
  "old_function_1:new_function_1"
  "old_function_2:new_function_2"
  "old_variable:new_variable"
)

for pattern in "${PATTERNS[@]}"; do
  IFS=':' read -r old new <<< "$pattern"
  echo "Replacing $old with $new..."
  refac . "$old" "$new" --force
done
```

## Scrap - File Management

The scrap tool provides temporary file storage and organization using a `.scrap` folder.

### Basic Operations

```bash
# Move files to .scrap folder
scrap temp_file.txt old_directory/

# List contents (default behavior)
scrap

# Search for files
scrap find "*.log"

# Clean old items
scrap clean --days 30

# Archive everything
scrap archive --remove
```

### Workflow Examples

```bash
# Development workflow
scrap debug.log temp_output/ experimental_code/
# Continue working...
scrap clean --days 7  # Weekly cleanup

# Project organization
scrap old_version/ deprecated_files/
scrap archive --output "project-backup-v1.0.tar.gz"
```

For detailed information, see the [Scrap Tool Guide]({{ '/scrap-guide/' | relative_url }}).

## Unscrap - File Restoration

The unscrap tool restores files from `.scrap` back to their original locations.

### Basic Operations

```bash
# Restore last scrapped item
unscrap

# Restore specific file
unscrap filename.txt

# Restore to custom location
unscrap file.txt --to /new/location/

# Force overwrite existing files
unscrap file.txt --force
```

### Workflow Examples

```bash
# Quick recovery
scrap important.txt  # Accidental scrap
unscrap              # Quick undo

# Selective restoration
scrap list           # See what's available
unscrap old_config.json --to ./backup/
```

For detailed information, see the [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}).

## Verbump - Version Management

### Installation and Setup

```bash
# Navigate to your git repository
cd your-project/

# Install git hook
verbump install

# Check status
verbump status
```

### Basic Operations

```bash
# Show current version information
verbump show

# Manual version update
verbump update

# Force update (outside git repo)
verbump update --force

# Check configuration and status
verbump status

# Uninstall hook
verbump uninstall
```

### Configuration

Create `.verbump.json` in your repository root:

```json
{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}
```

### Version Calculation

Verbump calculates versions using:
- **Major**: Latest git tag (e.g., `v1.2` → `1.2`)
- **Minor**: Commits since tag
- **Patch**: Total line changes

```bash
# Example with git tag v1.0 and 3 commits since tag:
# Result: 1.0.3.247 (where 247 is total changes)
```

### Workflow Examples

```bash
# Setup new project
git init
git add .
git commit -m "Initial commit"
git tag v1.0
verbump install

# Normal development (automatic)
echo "new feature" >> main.rs
git add .
git commit -m "Add feature"  # Version auto-updated

# Check current version
verbump show
cat version.txt

# Manual version check
verbump update --force
```

For detailed information, see the [Verbump Tool Guide]({{ '/verbump-guide/' | relative_url }}).

## Tool Integration

### Combined Workflows

```bash
# Complete development workflow
verbump install                               # Set up versioning
scrap temp_* *.log build/                    # Clear workspace
refac . "OldClass" "NewClass" --dry-run      # Preview changes
refac . "OldClass" "NewClass"                # Apply changes
git add . && git commit -m "Refactor class" # Auto-version bump

# Safe refactoring with backup
scrap old_implementation.rs                  # Backup current code
verbump show                                 # Note current version
refac . "OldClass" "NewClass" --dry-run      # Preview changes
refac . "OldClass" "NewClass"                # Apply changes
# If issues arise: unscrap old_implementation.rs

# Project cleanup and maintenance
scrap temp_* debug_* old_*/                  # Move temporary files
refac . "old_project_name" "new_project_name" # Rename project
scrap clean --days 14                       # Clean old scrapped files
verbump status                               # Check version tracking
```

### Automation Scripts

```bash
#!/bin/bash
# safe-refactor.sh - Complete refactoring workflow
OLD="$1"
NEW="$2"

# Check verbump is set up
if ! verbump status >/dev/null 2>&1; then
    echo "Setting up version tracking..."
    verbump install
fi

# Record current version
CURRENT_VERSION=$(verbump show 2>/dev/null | grep "Full Version" | cut -d: -f2 | xargs)
echo "Current version: $CURRENT_VERSION"

# Backup current state
echo "Creating backup..."
scrap archive --output "backup-$(date +%s).tar.gz"

# Preview changes
echo "Previewing changes..."
refac . "$OLD" "$NEW" --dry-run

# Ask for confirmation
read -p "Apply changes? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    refac . "$OLD" "$NEW"
    echo "Refactoring complete!"
    
    # Commit changes (triggers version bump)
    git add .
    git commit -m "Refactor: $OLD -> $NEW"
    
    # Show new version
    NEW_VERSION=$(cat version.txt 2>/dev/null || echo "unknown")
    echo "Version updated: $CURRENT_VERSION -> $NEW_VERSION"
else
    echo "Operation cancelled."
fi
```

## Next Steps

- Check the [Command Reference]({{ '/api-reference/' | relative_url }}) for complete option details
- See [Scrap Tool Guide]({{ '/scrap-guide/' | relative_url }}) for advanced file management
- See [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}) for restoration workflows
- See [Verbump Tool Guide]({{ '/verbump-guide/' | relative_url }}) for version management setup
- See [Examples]({{ '/examples/' | relative_url }}) for more real-world scenarios
- Report issues at [GitHub Issues](https://github.com/jowharshamshiri/refac/issues)