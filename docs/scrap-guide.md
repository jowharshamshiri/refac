---
layout: default
title: Scrap Tool Guide
---

# Scrap Tool Guide

The `scrap` tool provides file and directory management using a `.scrap` folder for temporary storage. It's designed to help you organize your workspace by temporarily moving files you don't need immediately, with the ability to easily restore them later.

## Core Concepts

### The .scrap Folder
- Automatically created in your current directory
- Stores temporarily moved files and directories
- Maintains metadata about original file locations
- Automatically added to `.gitignore` if it exists

### Metadata Tracking
- Records original file paths for easy restoration
- Timestamps when files were scrapped
- Enables intelligent restoration and history tracking

## Basic Operations

### Moving Files to .scrap

```bash
# Move a single file
scrap temp.log

# Move multiple files (run multiple commands)
scrap file1.txt file2.txt old_directory/

# Move with absolute paths
scrap /path/to/file.txt

# Automatic conflict resolution
scrap file.txt  # Creates .scrap/file.txt
scrap file.txt  # Creates .scrap/file_1.txt (automatic rename)
```

### Listing Contents

```bash
# List all items (default behavior)
scrap

# Explicit list command
scrap list

# Sort by different criteria
scrap list --sort name    # Alphabetical
scrap list --sort date    # Most recent first
scrap list --sort size    # Largest first
```

Example output:
```
Contents of .scrap folder:
--------------------------------------------------------------------------------
📄 temp.log               1.2 KB  2 hours ago     from: /home/user/project/temp.log
📁 old_code              15.3 MB  1 day ago       from: /home/user/project/old_code
📄 debug.txt              524 B   3 hours ago     from: /home/user/debug.txt
```

## Features

### Search and Find

```bash
# Search by filename (supports regex)
scrap find ".*\.log"           # Find all .log files
scrap find "test.*"            # Find files starting with "test"
scrap find "backup"            # Simple string search

# Search in file contents too
scrap find "TODO" --content    # Search for "TODO" in filenames and content
scrap find "bug.*fix" --content # Regex search in content
```

### Cleaning and Maintenance

```bash
# Remove items older than 30 days (default)
scrap clean

# Remove items older than specific number of days
scrap clean --days 7          # Remove items older than 1 week
scrap clean --days 1          # Remove items older than 1 day

# Preview what would be removed (dry run)
scrap clean --days 30 --dry-run

# Remove ALL items from .scrap folder
scrap purge

# Skip confirmation prompt
scrap purge --force
```

### Archive and Backup

```bash
# Archive .scrap contents to compressed file
scrap archive

# Archive with custom filename
scrap archive --output backup-2024.tar.gz
scrap archive --output "backup-$(date +%Y%m%d).tar.gz"

# Archive and remove original files from .scrap
scrap archive --remove

# Archive with custom name and remove
scrap archive --output monthly-backup.tar.gz --remove
```

## Workflow Examples

### Daily Workspace Cleanup

```bash
# Move temporary files to .scrap
scrap *.tmp *.log debug_output/

# List what's in .scrap
scrap

# Clean items older than a week
scrap clean --days 7

# Archive old items monthly
scrap archive --output "archive-$(date +%Y-%m).tar.gz" --remove
```

### Project Development

```bash
# Scrap old experimental code
scrap experimental_feature/ old_tests/

# Search for specific files later
scrap find "experimental.*"

# Restore if needed (using unscrap)
unscrap experimental_feature/

# Or clean up completely
scrap purge --force
```

### Code Refactoring

```bash
# Before major refactoring, scrap old implementations
scrap old_implementation.rs legacy_tests/

# Check what you've scrapped
scrap list --sort date

# If refactoring fails, restore old code
unscrap old_implementation.rs

# If successful, clean up
scrap clean --days 0  # Remove everything
```

## Safety Features

### Conflict Resolution
- **Automatic renaming**: If a file with the same name exists in `.scrap`, it's automatically renamed (e.g., `file_1.txt`, `file_2.txt`)
- **No overwrites**: Never overwrites existing files
- **Atomic operations**: File moves are atomic to prevent corruption

### Confirmation Prompts
- **Destructive operations**: `scrap purge` asks for confirmation unless `--force` is used
- **Preview mode**: `scrap clean --dry-run` shows what would be removed
- **Clear feedback**: Always shows what actions were taken

### Git Integration
- **Automatic .gitignore**: Adds `.scrap/` to `.gitignore` if the file exists
- **Repository safety**: Never commits temporary files to version control
- **Seamless workflow**: Works transparently in Git repositories

## Metadata and History

The scrap tool maintains detailed metadata about all operations:

### What's Tracked
- **Original paths**: Full path to where files came from
- **Timestamps**: When files were moved to .scrap
- **Restore information**: Data needed for intelligent restoration

### Metadata File
- Stored as `.scrap/.metadata.json`
- JSON format for easy parsing
- Automatically managed (no manual editing needed)
- Used by `unscrap` tool for restoration

## Tips and Best Practices

### Organization
```bash
# Use consistent naming for temporary files
scrap temp_* debug_* test_*

# Regular cleanup schedule
scrap clean --days 14  # Weekly cleanup of old items
```

### Backup Strategy
```bash
# Monthly archives
scrap archive --output "archive-$(date +%Y-%m).tar.gz" --remove

# Project-specific backups
scrap archive --output "project-backup-v1.0.tar.gz"
```

### Performance
- The `.scrap` folder is excluded from most file operations
- Metadata file is small and efficiently parsed
- Large directories are moved atomically (not copied)

### Integration with Other Tools
```bash
# Combine with find
find . -name "*.tmp" -exec scrap {} \;

# Use in scripts
if [ -f "old_config.conf" ]; then
    scrap old_config.conf
fi
```

## Error Handling

### Common Errors
- **File not found**: Clear error message if trying to scrap non-existent files
- **Permission denied**: Helpful error if lacking permissions
- **Disk space**: Warning if `.scrap` folder becomes very large

### Recovery
- **Metadata corruption**: Tool regenerates metadata from existing files
- **Partial operations**: Atomic moves prevent partial corruption
- **Emergency restore**: Files can be manually moved out of `.scrap` if needed

## See Also

- [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}) - Restoring files from .scrap
- [Installation Guide]({{ '/installation/' | relative_url }}) - Setting up the tools
- [API Reference]({{ '/api-reference/' | relative_url }}) - command reference