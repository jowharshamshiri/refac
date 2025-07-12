# Refac Tool

A robust, cross-platform command-line tool for recursive string replacement in file/folder names and contents. This tool is designed for safety, reliability, and performance, making it suitable for mission-critical operations like large-scale project refactoring.

## Features

### Core Functionality

- **Recursive processing**: Traverses directory trees with configurable depth limits
- **Dual operation modes**: Replaces strings in both file/directory names and file contents
- **Case-sensitive matching**: Ensures precise control over replacements
- **Cross-platform compatibility**: Works on Windows, macOS, and Linux

### Safety Features

- **Collision detection**: Prevents overwriting existing files/directories
- **Dry-run mode**: Preview changes before applying them
- **Binary file detection**: Automatically skips binary files for content replacement
- **Backup support**: Optional file backups before modification
- **Confirmation prompts**: Interactive confirmation unless forced

### Performance Features

- **Parallel processing**: Multi-threaded content replacement for large datasets
- **Streaming file processing**: Handles large files efficiently
- **Progress tracking**: Visual progress bars with detailed information
- **Smart filtering**: Include/exclude patterns with glob and regex support

### Advanced Options

- **Multiple operation modes**: Files-only, directories-only, names-only, content-only
- **Output formats**: Human-readable, JSON, and plain text
- **Verbose logging**: Detailed operation information
- **Symlink handling**: Configurable symlink following
- **Hidden file support**: Process hidden files and directories

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/refac
cd refac-tool/refac_rs

# Build and install
cargo build --release
cargo install --path .
```

## Usage

### Basic Syntax

```bash
refac <root_dir> <old_string> <new_string> [options]
```

### Examples

#### Basic Replacement

```bash
# Replace "oldname" with "newname" in current directory
refac . "oldname" "newname"

# Process specific directory
refac /path/to/project "OldClass" "NewClass"
```

#### Dry Run (Preview Changes)

```bash
# See what would be changed without making modifications
refac . "oldname" "newname" --dry-run
```

#### Operation Modes

```bash
# Only rename files and directories (skip content)
refac . "oldname" "newname" --names-only

# Only replace content (skip renaming)
refac . "oldname" "newname" --content-only

# Only process files (skip directories)
refac . "oldname" "newname" --files-only

# Only process directories (skip files)
refac . "oldname" "newname" --dirs-only
```

#### Advanced Features

```bash
# Force operation without confirmation
refac . "oldname" "newname" --force

# Create backups before modifying files
refac . "oldname" "newname" --backup

# Verbose output with detailed information
refac . "oldname" "newname" --verbose

# Limit directory traversal depth
refac . "oldname" "newname" --max-depth 3

# Use multiple threads for faster processing
refac . "oldname" "newname" --threads 8
```

#### Pattern Filtering

```bash
# Include only specific file types
refac . "oldname" "newname" --include "*.rs" --include "*.toml"

# Exclude specific patterns
refac . "oldname" "newname" --exclude "*.log" --exclude "target/*"

# Include hidden files
refac . "oldname" "newname" --include ".*"
```

#### Output Formats

```bash
# JSON output for scripting
refac . "oldname" "newname" --format json

# Plain text output
refac . "oldname" "newname" --format plain

# Human-readable output (default)
refac . "oldname" "newname" --format human
```

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--dry-run` | `-d` | Show what would be changed without making changes |
| `--force` | `-f` | Skip confirmation prompt |
| `--verbose` | `-v` | Show detailed output |
| `--backup` | `-b` | Create backup files before modifying content |
| `--files-only` | | Only process files (skip directories) |
| `--dirs-only` | | Only process directories (skip files) |
| `--names-only` | | Skip content replacement, only rename files/directories |
| `--content-only` | | Skip file/directory renaming, only replace content |
| `--follow-symlinks` | | Follow symbolic links |
| `--max-depth <N>` | | Maximum depth to search (0 = unlimited) |
| `--threads <N>` | `-j` | Number of threads to use (0 = auto) |
| `--include <PATTERN>` | | Include only files matching pattern |
| `--exclude <PATTERN>` | | Exclude files matching pattern |
| `--format <FORMAT>` | | Output format: human, json, plain |
| `--progress <MODE>` | | Progress display: auto, always, never |
| `--ignore-case` | `-i` | Ignore case when matching patterns |
| `--regex` | `-r` | Use regex patterns instead of literal strings |
| `--help` | `-h` | Show help information |
| `--version` | `-V` | Show version information |

## Safety Considerations

### What Gets Modified

- **File contents**: Text files only (binary files are automatically skipped)
- **File names**: Any file containing the target string
- **Directory names**: Any directory containing the target string

### What Doesn't Get Modified

- **Binary files**: Automatically detected and skipped for content replacement
- **The tool itself**: Self-modification is prevented
- **Symlink targets**: Unless `--follow-symlinks` is specified

### Collision Prevention

The tool checks for potential naming conflicts before making changes:

- Files/directories that would overwrite existing items
- Multiple sources trying to rename to the same target
- Case-only differences on case-insensitive filesystems

### Best Practices

1. **Always use dry-run first**: `--dry-run` to preview changes
2. **Use backups for important files**: `--backup` option
3. **Test on a copy**: Work on a backup of important directories
4. **Use version control**: Ensure your files are committed before running
5. **Be specific with patterns**: Use include/exclude patterns to limit scope

## Error Handling

The tool provides comprehensive error handling and reporting:

- **Input validation**: Checks for invalid arguments and paths
- **Permission errors**: Clear messages for insufficient permissions
- **File system errors**: Handles locked files, missing directories, etc.
- **Collision detection**: Prevents data loss from naming conflicts
- **Graceful degradation**: Continues processing after non-critical errors

## Performance

### Benchmarks

On typical hardware (modern SSD, multi-core CPU):

- **Small projects** (< 1000 files): < 1 second
- **Medium projects** (1000-10000 files): 1-10 seconds
- **Large projects** (> 10000 files): Scales linearly with thread count

### Optimization Tips

- Use `--threads` to increase parallelism for large datasets
- Use `--files-only` or `--dirs-only` when appropriate
- Use include/exclude patterns to limit processing scope
- Consider `--max-depth` for deep directory structures

## Output Examples

### Human-Readable Output

```
=== REFAC TOOL ===
Root directory: /path/to/project
Old string: 'oldname'
New string: 'newname'
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

### JSON Output

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

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Permission denied |
| 4 | File not found |
| 5 | Naming collision detected |

## Troubleshooting

### Common Issues

**"Permission denied" errors**

- Run with appropriate permissions
- Check file/directory ownership
- Ensure files are not locked by other processes

**"No changes found" when changes expected**

- Verify the search string is correct (case-sensitive)
- Check include/exclude patterns
- Use `--verbose` to see what's being processed

**"Naming collision detected"**

- Review the collision report
- Rename conflicting files manually
- Use different target names

**Binary files not being processed**

- This is by design for safety
- Use `--verbose` to see which files are skipped
- Manually verify file types if needed

### Debug Mode

For detailed debugging information:

```bash
refac . "old" "new" --verbose --dry-run
```

## Contributing

Contributions are welcome! Please read the contributing guidelines and submit pull requests for any improvements.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/refac
cd refac-tool/refac_rs

# Run tests
cargo test

# Run with test coverage
cargo test --all-features

# Build for release
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### Version 0.1.0

- Initial release
- Basic rename functionality
- Safety features and collision detection
- Multi-threading support
- Comprehensive test suite
