---
layout: default
title: Command Reference
---

# Command Reference

reference for all Refac Tools command-line options and usage patterns.

## Tools Overview

The Refac Tools suite includes four command-line utilities:

### Refac - String Replacement
```bash
refac <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

### Scrap - Local Trash
```bash
scrap [PATH...] [SUBCOMMAND] [OPTIONS]
```

### Unscrap - File Restoration  
```bash
unscrap [NAME] [OPTIONS]
```

### Verbump - Version Management
```bash
verbump [SUBCOMMAND] [OPTIONS]
```

---

## Refac Command Reference

## Arguments

### Required Arguments

| Argument | Description |
|----------|-------------|
| `ROOT_DIR` | Root directory to search in (use `.` for current directory) |
| `OLD_STRING` | String to find and replace |
| `NEW_STRING` | Replacement string |

### Argument Validation

- `ROOT_DIR` must exist and be a directory
- `OLD_STRING` cannot be empty
- `NEW_STRING` cannot be empty
- `NEW_STRING` cannot contain path separators (`/` or `\`)

## Options

### Basic Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--dry-run` | `-d` | Show what would be changed without making changes | `false` |
| `--force` | `-f` | Skip confirmation prompt | `false` |
| `--verbose` | `-v` | Show detailed output | `false` |
| `--help` | `-h` | Show help information | |
| `--version` | `-V` | Show version information | |

### Safety Options

| Option | Description | Default |
|--------|-------------|---------|
| `--backup` | `-b` | Create backup files before modifying content | `false` |
| `--follow-symlinks` | Follow symbolic links | `false` |

### Operation Mode Options

| Option | Description | Default |
|--------|-------------|---------|
| `--files-only` | Only process files (skip directories) | `false` |
| `--dirs-only` | Only process directories (skip files) | `false` |
| `--names-only` | Skip content replacement, only rename files/directories | `false` |
| `--content-only` | Skip file/directory renaming, only replace content | `false` |

**Note**: Only one mode flag can be specified at a time.

### Filtering Options

| Option | Description | Default |
|--------|-------------|---------|
| `--include <PATTERN>` | Include only files matching pattern (glob) | Include all |
| `--exclude <PATTERN>` | Exclude files matching pattern (glob) | Exclude none |
| `--max-depth <N>` | Maximum depth to search (0 = unlimited) | `0` |

**Note**: Multiple `--include` and `--exclude` patterns can be specified.

### Performance Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--threads <N>` | `-j` | Number of threads to use (0 = auto) | `0` |

### Pattern Matching Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--ignore-case` | `-i` | Ignore case when matching patterns | `false` |
| `--regex` | `-r` | Use regex patterns instead of literal strings | `false` |

### Output Options

| Option | Description | Values | Default |
|--------|-------------|--------|---------|
| `--format <FORMAT>` | Output format | `human`, `json`, `plain` | `human` |
| `--progress <MODE>` | Progress display mode | `auto`, `always`, `never` | `auto` |

## Operation Modes

### Full Mode (Default)

Processes both file/directory names and file contents.

```bash
refac . "oldname" "newname"
```

### Files Only Mode

Process only files, skip directories.

```bash
refac . "oldname" "newname" --files-only
```

### Directories Only Mode

Process only directories, skip files.

```bash
refac . "oldname" "newname" --dirs-only
```

### Names Only Mode

Only rename files and directories, skip content replacement.

```bash
refac . "oldname" "newname" --names-only
```

### Content Only Mode

Only replace content in files, skip renaming.

```bash
refac . "oldname" "newname" --content-only
```

## Pattern Matching

### Glob Patterns

Use glob patterns for include/exclude filters:

```bash
# Include specific file types
refac . "old" "new" --include "*.rs" --include "*.toml"

# Exclude directories
refac . "old" "new" --exclude "target/*" --exclude "node_modules/*"

# Include hidden files
refac . "old" "new" --include ".*"
```

#### Glob Pattern Syntax

| Pattern | Matches |
|---------|---------|
| `*` | Any sequence of characters |
| `?` | Any single character |
| `[abc]` | Any character in brackets |
| `[a-z]` | Any character in range |
| `**` | Recursive directory match |

### Regular Expressions

Use regex patterns with the `--regex` flag:

```bash
# Match word boundaries
refac . "\\bold\\b" "new" --regex

# Case-insensitive regex
refac . "old.*name" "newname" --regex --ignore-case

# Match numbers
refac . "version_\\d+" "version_2" --regex
```

## Output Formats

### Human Format (Default)

Colored, interactive output with progress bars:

```bash
refac . "old" "new"
```

Example output:
```
=== REFAC TOOL ===
Root directory: /project
Old string: 'old'
New string: 'new'
Mode: Full

Phase 1: Discovering files and directories...
Phase 2: Checking for naming collisions...

=== CHANGE SUMMARY ===
Content modifications: 15 file(s)
File renames:         8 file(s)
Directory renames:    3 directory(ies)
Total changes:        26

Do you want to proceed? (y/N) y

Replacing content in files...
Renaming files and directories...

=== OPERATION COMPLETE ===
Operation completed successfully!
Total changes applied: 26
```

### JSON Format

Machine-readable output for scripting:

```bash
refac . "old" "new" --format json
```

Example output:
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

### Plain Format

Simple text output without colors:

```bash
refac . "old" "new" --format plain
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments |
| `3` | Permission denied |
| `4` | File not found |
| `5` | Naming collision detected |

## Examples

### Basic Usage

```bash
# Simple replacement
refac . "oldname" "newname"

# Preview changes
refac . "oldname" "newname" --dry-run

# Force without confirmation
refac . "oldname" "newname" --force
```

### File Type Filtering

```bash
# Only Rust files
refac . "old_function" "new_function" --include "*.rs"

# Multiple file types
refac . "oldname" "newname" --include "*.js" --include "*.ts" --include "*.json"

# Exclude build artifacts
refac . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

### Mode Examples

```bash
# Only rename files, don't change content
refac ./docs "draft" "final" --names-only

# Only change content, don't rename files
refac ./config "old.server.com" "new.server.com" --content-only

# Only process files, skip directories
refac . "oldname" "newname" --files-only
```

### Options

```bash
# Limit search depth
refac . "oldname" "newname" --max-depth 3

# Use more threads
refac . "oldname" "newname" --threads 8

# Case-insensitive matching
refac . "oldname" "newname" --ignore-case

# Regex patterns
refac . "old_\\w+" "new_name" --regex
```

### Safety Features

```bash
# Create backups
refac . "oldname" "newname" --backup

# Verbose output for debugging
refac . "oldname" "newname" --dry-run --verbose
```

### Output Formats

```bash
# JSON output for scripts
refac . "oldname" "newname" --format json

# Plain text output
refac . "oldname" "newname" --format plain

# Disable progress bars
refac . "oldname" "newname" --progress never
```

## Limitations

### String Restrictions

- Old string cannot be empty
- New string cannot be empty
- New string cannot contain path separators (`/` or `\`)

### Performance Limits

- Maximum thread count: 1000
- Maximum search depth: 1000

### File Handling

- Binary files are automatically skipped for content replacement
- Symlinks are not followed unless `--follow-symlinks` is specified
- Very large files may require additional memory

## Error Handling

### Common Error Messages

**"Root directory does not exist"**
- Check that the specified directory path is correct

**"Cannot specify more than one mode flag"**
- Use only one of: `--files-only`, `--dirs-only`, `--names-only`, `--content-only`

**"New string cannot contain path separators"**
- Remove `/` or `\` characters from the new string

**"Naming collision detected"**
- Review the collision report and resolve conflicts manually

### Debugging

Use verbose mode to see detailed operation information:

```bash
refac . "oldname" "newname" --dry-run --verbose
```

This will show:
- Which files are being processed
- Which files are being skipped
- Why files are being skipped
- Detailed pattern matching information

---

## Scrap Command Reference

### Synopsis
```bash
scrap [PATH...] [SUBCOMMAND] [OPTIONS]
```

### Basic Operations
```bash
# Move unwanted files to .scrap trash folder
scrap file.txt directory/

# List .scrap contents (default when no args)
scrap
scrap list [--sort name|date|size]
```

### Subcommands

| Subcommand | Description | Options |
|------------|-------------|---------|
| `list` | List .scrap contents | `--sort name\|date\|size` |
| `clean` | Remove old items | `--days N`, `--dry-run` |
| `purge` | Remove all items | `--force` |
| `find` | Search for patterns | `--content` |
| `archive` | Create archive | `--output FILE`, `--remove` |

### Examples
```bash
scrap temp.txt logs/                    # Move to trash
scrap list --sort size                  # List trash contents
scrap find "*.log"                      # Find log files in trash
scrap clean --days 30                   # Permanently remove old items
scrap archive backup.tar.gz --remove   # Archive and remove
scrap purge --force                     # Empty trash completely
```

---

## Unscrap Command Reference

### Synopsis
```bash
unscrap [NAME] [OPTIONS]
```

### Operations

| Command | Description |
|---------|-------------|
| `unscrap` | Restore last scrapped item |
| `unscrap NAME` | Restore specific item |
| `unscrap NAME --to PATH` | Restore to custom location |
| `unscrap NAME --force` | Overwrite existing files |

### Options

| Option | Description |
|--------|-------------|
| `--to PATH` | Custom restoration path |
| `--force` | Overwrite existing files |
| `--help` | Show help |
| `--version` | Show version |

### Examples
```bash
unscrap                           # Restore last item
unscrap important_file.txt        # Restore specific file
unscrap config.json --to backup/  # Restore to directory
unscrap data.txt --force          # Overwrite existing
```

---

## Verbump Command Reference

### Synopsis
```bash
verbump [SUBCOMMAND] [OPTIONS]
```

### Subcommands

| Subcommand | Description | Options |
|------------|-------------|---------|
| `install` | Install git pre-commit hook | `--force` |
| `uninstall` | Remove git hook | |
| `show` | Display version information | |
| `update` | Manually update version | `--force` |
| `status` | Show configuration status | |

### Configuration

Create `.verbump.json` in repository root:
```json
{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}
```

### Version Format

`{major}.{minor}.{patch}`

- **Major**: From git tags (e.g., `v1.2` → `1.2`)
- **Minor**: Commits since tag
- **Patch**: Total line changes

### Examples
```bash
verbump install                 # Install git hook
verbump install --force         # Force reinstall
verbump show                    # Show version info
verbump update                  # Manual update
verbump status                  # Check status
verbump uninstall              # Remove hook
```

---

## Getting Help

### Tool-Specific Help
```bash
# Show help for each tool
refac --help
scrap --help
unscrap --help
verbump --help

# Show versions
refac --version
scrap --version
unscrap --version
verbump --version
```

### Resources

For more information:
- [Getting Started]({{ '/getting-started/' | relative_url }}) - Quick start guide
- [Usage Guide]({{ '/usage/' | relative_url }}) - usage examples
- [Tool-Specific Guides]({{ '/scrap-guide/' | relative_url }}) - Individual tool documentation
- [GitHub Issues](https://github.com/jowharshamshiri/refac/issues) - Report bugs or request features