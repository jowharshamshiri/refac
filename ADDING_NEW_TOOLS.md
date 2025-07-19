# Adding New Tools to Refac Tools Suite

This document provides a guide for adding new tools to the Refac Tools suite. It captures all requirements, patterns, and integration steps to ensure consistency and quality across all tools.

## Table of Contents

1. [Tool Design Philosophy](#tool-design-philosophy)
2. [Project Structure Requirements](#project-structure-requirements)
3. [Implementation Requirements](#implementation-requirements)
4. [Testing Requirements](#testing-requirements)
5. [Documentation Requirements](#documentation-requirements)
6. [Integration Requirements](#integration-requirements)
7. [Step-by-Step Implementation Guide](#step-by-step-implementation-guide)
8. [Quality Checklist](#quality-checklist)

## Tool Design Philosophy

### Core Principles

All tools in the Refac Tools suite must adhere to these fundamental principles:

1. **Safety First**: Never destructive without explicit confirmation
2. **Atomic Operations**: All-or-nothing operations to prevent corruption
3. **Clear Error Messages**: Helpful, actionable error messages
4. **Predictable Behavior**: Consistent patterns across all tools
5. **Performance Optimized**: Efficient for both small and large operations
6. **Cross-Platform**: Works on Windows, macOS, and Linux
7. **Mission-Critical Ready**: Robust enough for production environments

### User Experience Standards

- **Intuitive CLI**: Self-explanatory command structure
- **Helpful Defaults**: Sensible default behavior
- **Progressive Disclosure**: Basic usage is simple, advanced features available
- **Confirmation Prompts**: Interactive confirmation for destructive operations
- **Dry-Run Mode**: Preview functionality where applicable
- **Verbose Mode**: Detailed output for debugging
- **Colored Output**: Clear visual feedback with colored status messages

## Project Structure Requirements

### Directory Organization

When adding a new tool called `newtool`:

```
src/
├── newtool/                    # Tool-specific modules
│   ├── mod.rs                  # Module exports and public API
│   ├── newtool_common.rs       # Core functionality and data structures
│   └── [additional_modules.rs] # Tool-specific modules as needed
├── bin/
│   └── newtool.rs             # Binary entry point
├── refac/                     # Existing refac tool modules
├── scrap/                     # Existing scrap tool modules
└── lib.rs                     # Root library with re-exports
```

### File Naming Conventions

- **Binary**: `src/bin/toolname.rs`
- **Module directory**: `src/toolname/`
- **Core module**: `src/toolname/toolname_common.rs`
- **Module file**: `src/toolname/mod.rs`
- **Tests**: `tests/toolname_integration_tests.rs`
- **Advanced tests**: `tests/toolname_advanced_integration_tests.rs`

## Implementation Requirements

### Cargo.toml Configuration

Add binary configuration:

```toml
[[bin]]
name = "newtool"
path = "src/bin/newtool.rs"
```

### Required Dependencies

Ensure these dependencies are available:

```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "color"] }
anyhow = "1.0"
colored = "2.0"
# Add tool-specific dependencies as needed
```

### Command-Line Interface Structure

Use consistent CLI patterns:

```rust
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(name = "newtool")]
#[command(about = "Brief description of what the tool does")]
#[command(version = "0.1.0")]
struct Args {
    /// Main argument (if applicable)
    target: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List/show information
    List {
        /// Sort criteria
        #[arg(short, long, default_value = "name")]
        sort: String,
    },
    
    /// Perform main operation
    Process {
        /// Force without confirmation
        #[arg(short, long)]
        force: bool,
        
        /// Show what would be done
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    
    // Add other subcommands as needed
}
```

### Error Handling Pattern

Use consistent error handling:

```rust
use anyhow::{Context, Result};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {:#}", "Error".red(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Main implementation
    Ok(())
}

// Use .context() for error messages
fs::read_to_string(&path)
    .with_context(|| format!("Failed to read file: {}", path.display()))?;
```

### Output and Feedback Patterns

Use consistent output patterns:

```rust
// Success messages
println!("{} Operation completed successfully", "Success".green());

// Info messages  
println!("{} Processing files...", "Info".blue());

// Warnings
println!("{} This is a warning", "Warning".yellow());

// Errors (use error! macro or eprintln!)
eprintln!("{} Something went wrong", "Error".red());
```

### Metadata and State Management

If the tool manages state or metadata:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub version: u32,
    pub entries: HashMap<String, ToolEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolEntry {
    pub created_at: DateTime<Utc>,
    pub metadata_field: String,
    // Add tool-specific fields
}

impl ToolMetadata {
    pub fn load(dir: &Path) -> Result<Self> { /* ... */ }
    pub fn save(&self, dir: &Path) -> Result<()> { /* ... */ }
}
```

## Testing Requirements

### Test Coverage

Every new tool must have:

1. **Unit Tests** (in the binary file):
   - Core functionality tests
   - Edge case handling
   - Error condition tests
   - Input validation tests

2. **Integration Tests** (separate test file):
   - End-to-end functionality
   - CLI argument parsing
   - File system operations
   - Cross-platform compatibility

3. **Advanced Integration Tests** (if complex):
   - Complex workflow scenarios
   - Performance tests
   - Interaction with other tools
   - Stress tests

### Test Structure Pattern

```rust
// In src/bin/newtool.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let work_dir = temp_dir.path().to_path_buf();
        (temp_dir, work_dir)
    }
    
    #[test]
    fn test_basic_functionality() {
        let (temp_dir, work_dir) = setup_test_env();
        // Test implementation
    }
    
    #[test]
    fn test_error_conditions() {
        // Test error handling
    }
    
    // Add 5-10 unit tests covering all major functions
}
```

### Integration Test Pattern

```rust
// In tests/newtool_integration_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_command() {
    let temp_dir = TempDir::new().unwrap();
    
    Command::cargo_bin("newtool")
        .unwrap()
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}

#[test] 
fn test_help_output() {
    Command::cargo_bin("newtool")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Brief description"));
}

// Add 15-20 integration tests covering:
// - All CLI commands and options
// - Error conditions
// - File operations
// - Cross-platform scenarios
```

### Test Quality Requirements

- **Minimum 20 tests total** (unit + integration)
- **All error paths tested**
- **Cross-platform compatibility verified**
- **Temporary directories for file operations**
- **No test dependencies on external state**
- **Tests run in parallel safely**

## Documentation Requirements

### README.md Updates

Add tool to the main README.md:

1. **Tools Overview Section**: Add brief description
2. **Installation Section**: Update with new binary name
3. **Quick Start Section**: Add basic usage examples
4. **New Tool Section**: documentation

### Dedicated Tool Guide

Create `docs/newtool-guide.md`:

```markdown
---
layout: default
title: NewTool Guide
---

# NewTool Guide

Brief description of what the tool does and why it's useful.

## Core Concepts

### Key Concept 1
Explanation of main concepts users need to understand.

### Key Concept 2
Additional important concepts.

## Basic Operations

### Primary Use Case
```bash
# Common usage examples
newtool basic-command
```

### Secondary Use Cases  

```bash
# Additional examples
newtool advanced-command --options
```

## Advanced Features

### Feature 1

Detailed explanation with examples.

### Feature 2

More advanced functionality.

## Workflow Examples

### Typical Workflow

Step-by-step workflow examples.

### Integration with Other Tools

How it works with refac, scrap, unscrap.

## Safety Features

### What It Protects Against

Explanation of safety mechanisms.

### Best Practices

Recommended usage patterns.

## See Also

Links to related documentation.

```

### GitHub Pages Updates

Update the following documentation files:

1. **`docs/index.md`**: Add tool to overview
2. **`docs/usage.md`**: Add usage section
3. **`docs/installation.md`**: Update installation instructions
4. **Navigation**: Ensure tool guide is linked

### Help and Version Output

Ensure consistent help output:

```rust
#[command(about = "Brief, clear description of tool purpose")]
#[command(long_about = "Longer description with key features and use cases")]
#[command(version = "0.1.0")]
```

## Integration Requirements

### Installation Script Updates

Update `install.sh` and `uninstall.sh`:

```bash
# In install.sh, add to binaries array:
local binaries=("refac" "scrap" "unscrap" "newtool")

# In uninstall.sh, add to binaries array:  
local binaries=("refac" "scrap" "unscrap" "newtool")
```

### Library Integration

Update `src/lib.rs`:

```rust
pub mod refac;
pub mod scrap;
pub mod newtool;  // Add new module

// Re-export public types if needed
pub use newtool::newtool_common::{NewToolMetadata, NewToolEntry};
```

### Module Structure

Create `src/newtool/mod.rs`:

```rust
pub mod newtool_common;

pub use newtool_common::{NewToolMetadata, NewToolEntry};
```

### Cross-Tool Integration

If the tool integrates with others:

1. **Shared metadata formats** for interoperability
2. **Common directory patterns** (like `.scrap` folder)
3. **Consistent CLI patterns** for user familiarity
4. **Compatible file operations** for workflow integration

## Step-by-Step Implementation Guide

### Phase 1: Setup and Structure

1. **Add binary configuration** to `Cargo.toml`
2. **Create directory structure** in `src/newtool/`
3. **Create basic binary** in `src/bin/newtool.rs`
4. **Set up CLI structure** with clap
5. **Add to lib.rs** re-exports
6. **Verify compilation** with `cargo build`

### Phase 2: Core Implementation

1. **Implement core data structures** in `newtool_common.rs`
2. **Add main functionality** with proper error handling
3. **Implement CLI command handling**
4. **Add file operations** with safety checks
5. **Add confirmation prompts** for destructive operations
6. **Test basic functionality manually**

### Phase 3: Advanced Features

1. **Add subcommands** as needed
2. **Implement dry-run mode** if applicable
3. **Add verbose output** options
4. **Optimize performance** for large operations
5. **Add progress indicators** for long operations
6. **Handle edge cases** and error conditions

### Phase 4: Testing

1. **Write unit tests** (minimum 10 tests)
2. **Create integration tests** (minimum 15 tests)
3. **Test error conditions** thoroughly
4. **Verify cross-platform compatibility**
5. **Run full test suite**: `cargo test`
6. **Test manual installation** and usage

### Phase 5: Documentation

1. **Update README.md** with tool info
2. **Create tool guide** in docs/
3. **Update installation guide**
4. **Update GitHub Pages** documentation
5. **Add usage examples** to docs/usage.md
6. **Test documentation** for clarity

### Phase 6: Integration

1. **Update installation scripts**
2. **Test installation process**
3. **Verify uninstallation process**
4. **Test tool interactions** (if applicable)
5. **Run full test suite** one final time
6. **Manual end-to-end testing**

## Quality Checklist

Before considering a new tool complete, verify all items:

### ✅ **Implementation**

- [ ] Binary compiles without warnings
- [ ] All CLI commands work as expected
- [ ] Error handling is comprehensive
- [ ] Output follows project patterns
- [ ] Performance is acceptable for intended use
- [ ] Cross-platform compatibility verified

### ✅ **Safety**

- [ ] No destructive operations without confirmation
- [ ] Atomic operations where applicable
- [ ] Clear error messages for all failure modes
- [ ] Input validation prevents crashes
- [ ] File operations are safe (no data loss)
- [ ] Handles edge cases gracefully

### ✅ **Testing**

- [ ] Minimum 20 total tests (unit + integration)
- [ ] All major functions have unit tests
- [ ] All CLI options have integration tests
- [ ] Error conditions are tested
- [ ] Cross-platform scenarios tested
- [ ] Tests run cleanly in parallel
- [ ] All tests pass: `cargo test`

### ✅ **Documentation**

- [ ] README.md updated with tool info
- [ ] Dedicated tool guide created
- [ ] Installation instructions updated
- [ ] GitHub Pages documentation updated
- [ ] Help output is clear and complete
- [ ] Usage examples are comprehensive

### ✅ **Integration**

- [ ] Installation script updated
- [ ] Uninstallation script updated
- [ ] Library exports updated if needed
- [ ] Tool works with existing suite
- [ ] Installation process tested
- [ ] Uninstallation process tested

### ✅ **User Experience**

- [ ] CLI is intuitive and consistent
- [ ] Output is clear and actionable
- [ ] Error messages are helpful
- [ ] Confirmation prompts where appropriate
- [ ] Dry-run mode if applicable
- [ ] Verbose mode available
- [ ] Performance is responsive

## Examples and References

### Example: Adding a "Backup" Tool

Hypothetical `backup` tool that creates timestamped backups:

**Structure:**

```
src/backup/
├── mod.rs
└── backup_common.rs
src/bin/backup.rs
tests/backup_integration_tests.rs
docs/backup-guide.md
```

**CLI Design:**

```bash
backup file.txt                    # Create backup
backup restore file.txt.backup     # Restore backup
backup list                        # List all backups
backup clean --days 30             # Clean old backups
```

**Integration:**

- Works with scrap: `backup . && scrap old_version/`
- Works with refac: `backup . && refac . "old" "new"`

### Reference Implementation

Study existing tools for patterns:

- **`scrap`**: Complex tool with subcommands, metadata, file operations
- **`unscrap`**: Simpler tool with focused functionality
- **`refac`**: Performance-critical tool with parallel processing

Each demonstrates different aspects of the project patterns and can serve as implementation references.

---

## Final Notes

This guide ensures consistency, quality, and maintainability across all tools in the Refac Tools suite. Following these patterns makes the tools feel like a cohesive suite rather than separate utilities, providing users with a smooth, predictable experience.

When in doubt, look at existing tools for patterns and maintain consistency with established conventions. The goal is to create tools that are powerful, safe, and delightful to use.
