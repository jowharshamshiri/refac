---
layout: default
title: Refac - String Replacement Tool
toc: false
---

# Refac

A robust, cross-platform command-line tool for recursive string replacement in file/folder names and contents. Designed for safety, reliability, and performance, making it suitable for mission-critical operations like large-scale project refactoring.

## Key Features

- **Dual Operation**: Replace strings in both file/directory names AND file contents
- **Safety First**: Collision detection, dry-run mode, and binary file protection
- **High Performance**: Multi-threaded processing with progress tracking
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Flexible Filtering**: Include/exclude patterns with glob and regex support
- **Multiple Modes**: Files-only, directories-only, names-only, or content-only processing

## Quick Start

```bash
# Basic usage
refac . "oldname" "newname"

# Preview changes first (recommended)
refac . "oldname" "newname" --dry-run

# Only rename files/directories (skip content)
refac . "oldname" "newname" --names-only

# Only replace content (skip renaming)
refac . "oldname" "newname" --content-only

# Create backups before changes
refac . "oldname" "newname" --backup
```

## Installation

### From Source

```bash
git clone https://github.com/jowharshamshiri/refac
cd refac
cargo build --release
cargo install --path .
```

## How It Works

Refac performs two types of operations:

1. **Name Replacement**: Renames files and directories containing the target string
2. **Content Replacement**: Replaces strings inside text files (automatically skips binary files)

By default, both operations are performed. Use mode flags to limit the scope:

- `--names-only`: Only rename files/directories
- `--content-only`: Only replace file contents
- `--files-only`: Process files but not directories
- `--dirs-only`: Process directories but not files

## Safety Features

- **Collision Detection**: Prevents overwriting existing files
- **Binary File Detection**: Automatically skips binary files for content replacement
- **Dry Run Mode**: Preview all changes before applying them
- **Backup Support**: Create backups of modified files
- **Confirmation Prompts**: Interactive confirmation (unless `--force` is used)

## Performance

- **Multi-threaded**: Parallel content processing for large codebases
- **Streaming**: Efficient handling of large files
- **Progress Tracking**: Visual progress bars with detailed information
- **Smart Filtering**: Process only relevant files with include/exclude patterns

## Common Use Cases

### Project Refactoring

```bash
# Rename a class throughout a codebase
refac ./src "OldClassName" "NewClassName"

# Rename variables (case-sensitive)
refac ./project "old_variable" "new_variable"
```

### File Organization

```bash
# Rename files only, skip content
refac ./docs "draft" "final" --names-only

# Update file contents only, keep names
refac ./config "old.example.com" "new.example.com" --content-only
```

### Bulk Operations

```bash
# Process specific file types
refac ./src "oldname" "newname" --include "*.rs" --include "*.toml"

# Exclude certain directories
refac ./project "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

## Best Practices

1. **Always test first**: Use `--dry-run` to preview changes
2. **Use version control**: Commit your code before running refac
3. **Create backups**: Use `--backup` for important changes
4. **Be specific**: Use include/exclude patterns to limit scope
5. **Test after changes**: Run your tests after refactoring

## Getting Help

- View all options: `refac --help`
- Check version: `refac --version`
- Report issues: [GitHub Issues](https://github.com/jowharshamshiri/refac/issues)

## Documentation

- [Installation Guide]({{ '/installation/' | relative_url }}) - Detailed installation instructions
- [Usage Guide]({{ '/usage/' | relative_url }}) - Comprehensive usage examples
- [Command Reference]({{ '/api-reference/' | relative_url }}) - Complete command-line reference

## License

MIT License - see the [LICENSE](https://github.com/jowharshamshiri/refac/blob/main/LICENSE) file for details.
