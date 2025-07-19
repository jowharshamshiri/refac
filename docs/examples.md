---
layout: default
title: Examples
---

# Real-World Examples

Learn how to use the Refac Tools suite through practical examples for common development workflows, combining refactoring, project cleanup, and version control.

## Development Workflows

### Project Refactoring with Version Tracking

**Scenario**: Major refactoring of a class while maintaining version history and safely managing temporary files.

```bash
# 1. Set up version tracking
verbump install
git tag v1.0  # Mark current state

# 2. Move unwanted files to trash
scrap temp_files/ debug_logs/ old_tests/

# 3. Preview and apply refactoring
refac ./src "UserManager" "AccountManager" --dry-run
refac ./src "UserManager" "AccountManager" --include "*.rs" --include "*.toml"

# 4. Commit changes (auto-bumps version)
git add .
git commit -m "Refactor: UserManager -> AccountManager"

# 5. Check new version
verbump show
cat version.txt  # Shows new version: v1.0.1.X
```

### Safe Experimental Development

**Scenario**: Trying experimental changes with easy rollback.

```bash
# 1. Backup current implementation
scrap src/experimental.rs src/core.rs

# 2. Note current version
CURRENT_VERSION=$(verbump show | grep "Full Version" | cut -d: -f2 | xargs)
echo "Starting experiment from version: $CURRENT_VERSION"

# 3. Make experimental changes
refac ./src "old_algorithm" "new_algorithm" --content-only

# 4. If experiment fails, restore easily
unscrap experimental.rs
unscrap core.rs

# 5. If experiment succeeds, commit
git add . && git commit -m "Implement new algorithm"
```

### CI/CD Preparation Workflow

**Scenario**: Preparing a release with proper cleanup and versioning.

```bash
# 1. Archive old artifacts
scrap find "*.tmp" "*.log" "target/debug/*"
scrap archive pre-release-cleanup.tar.gz --remove

# 2. Update configuration for production
refac ./config "dev.api.com" "prod.api.com" --content-only
refac ./config "debug=true" "debug=false" --content-only

# 3. Ensure version tracking is set up
verbump status || verbump install

# 4. Create release commit
git add .
git commit -m "Prepare for production release"

# 5. Tag the release
NEW_VERSION=$(cat version.txt)
git tag "v$NEW_VERSION"
echo "Tagged release: v$NEW_VERSION"
```

---

## Individual Tool Examples

### Scrap - Local Trash Folder

#### Project Cleanup Instead of Deletion

**Scenario**: Cleaning up files you don't want but might need later.

```bash
# Move unwanted files to .scrap instead of deleting
scrap *.tmp *.log build/ target/debug/

# List what you've scrapped
scrap list --sort size

# Find specific items you remember scrapping
scrap find "*.log"
scrap find "test" --content  # Search in file contents

# Clean up old items (older than 7 days) permanently
scrap clean --days 7 --dry-run
scrap clean --days 7
```

#### Safe Code Cleanup

**Scenario**: Removing old implementations and experimental code safely.

```bash
# Scrap old code instead of deleting (in case you need it)
scrap old_implementation/ legacy_tests/

# Archive before permanent removal
scrap archive "old-code-$(date +%Y%m%d).tar.gz"

# Scrap experimental features that didn't work out
scrap experimental_feature/ prototype/
scrap archive --output experiments-archive.tar.gz --remove
```

### Unscrap - File Restoration

#### Undo Recent Changes

**Scenario**: Need to restore files after a mistake.

```bash
# Restore the last scrapped item
unscrap

# Check what's available to restore
scrap list

# Restore specific file to original location
unscrap important_config.json

# Restore to a different location
unscrap data.sql --to backup/
```

#### Selective File Recovery

**Scenario**: Restore only specific files from a cleanup.

```bash
# After cleaning workspace, need one file back
scrap temp_files/ logs/ build/
# ... realize you need a log file
unscrap server.log

# Restore to custom location without overwriting
unscrap config.json --to ./backup/ --force
```

### Verbump - Version Management

#### Project Setup and Release Management

**Scenario**: Setting up automatic versioning for a new project.

```bash
# Initialize project with versioning
git init
echo "Initial code" > main.rs
git add .
git commit -m "Initial commit"

# Set base version and install hook
git tag v0.1.0
verbump install

# Normal development with automatic versioning
echo "new feature" >> main.rs
git add .
git commit -m "Add feature"  # Version auto-incremented

# Check version progression
verbump show
cat version.txt  # 0.1.1.X (1 commit since tag, X changes)
```

#### Release Workflow Integration

**Scenario**: Integrating versioning with release process.

```bash
# Create feature branch with versioning
git checkout -b feature-auth
verbump install --force  # Ensure hook is active

# Development commits auto-increment version
git commit -m "Add auth module"     # v0.1.2.Y
git commit -m "Add user validation" # v0.1.3.Z

# Prepare for release
git checkout main
git merge feature-auth
git tag v0.2.0  # New major/minor version

# Future commits will be v0.2.X.Y
```

#### CI/CD Integration

**Scenario**: Using versions in build scripts.

```bash
# In build script
VERSION=$(cat version.txt)
echo "Building version: $VERSION"

# Use in Docker builds
docker build -t myapp:$VERSION .
docker build -t myapp:latest .

# Use in artifact naming
cargo build --release
cp target/release/myapp "releases/myapp-$VERSION"
```

### Refac - String Replacement

#### Rename Variables Throughout Project

**Scenario**: You need to rename a variable across your entire codebase.

```bash
# Preview the changes first
refac ./src "oldVariableName" "newVariableName" --dry-run

# Apply the changes
refac ./src "oldVariableName" "newVariableName"
```

### Update API Endpoints

**Scenario**: Your API URL changed and you need to update all references.

```bash
# Update only file contents, don't rename files
refac . "api.old-service.com" "api.new-service.com" --content-only

# Include only relevant file types
refac . "api.old-service.com" "api.new-service.com" \
  --content-only \
  --include "*.js" \
  --include "*.py" \
  --include "*.json"
```

## File and Directory Organization

### Rename Project Files

**Scenario**: You're renaming your project from "MyApp" to "AwesomeApp".

```bash
# Rename both files and their contents
refac . "MyApp" "AwesomeApp" --dry-run

# Exclude certain directories
refac . "MyApp" "AwesomeApp" \
  --exclude "node_modules/*" \
  --exclude ".git/*" \
  --exclude "target/*"
```

### Reorganize File Naming Convention

**Scenario**: Change file naming from `camelCase` to `snake_case`.

```bash
# Rename files only, don't change content
refac ./src "camelCase" "snake_case" --names-only

# Or use regex for more complex patterns
refac ./src "([a-z])([A-Z])" "\$1_\$2" --names-only --regex
```

## Language-Specific Refactoring

### Rust Project Refactoring

**Scenario**: Rename a struct and update all references.

```bash
# Target only Rust files
refac ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --include "*.toml"

# With backup for safety
refac ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --backup
```

### JavaScript/TypeScript Project

**Scenario**: Update function names across JS/TS files.

```bash
# Target JS and TS files
refac ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --include "*.jsx" \
  --include "*.tsx"

# Exclude test files
refac ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --exclude "*test*" \
  --exclude "*spec*"
```

### Python Project

**Scenario**: Rename a class and update imports.

```bash
# Python files only
refac ./project "OldClass" "NewClass" \
  --include "*.py" \
  --exclude "__pycache__/*"

# Include requirements files too
refac ./project "old-package" "new-package" \
  --include "*.py" \
  --include "requirements*.txt" \
  --include "setup.py"
```

## Configuration and Deployment

### Update Environment Variables

**Scenario**: Change environment variable names in configuration files.

```bash
# Target configuration files
refac ./config "OLD_ENV_VAR" "NEW_ENV_VAR" \
  --include "*.env" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "*.json"

# Content only (don't rename config files)
refac ./config "staging.server.com" "production.server.com" \
  --content-only \
  --include "*.env" \
  --include "*.config"
```

### Docker and Deployment Scripts

**Scenario**: Update service names in deployment configurations.

```bash
# Update container names
refac ./deployment "old-service" "new-service" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "Dockerfile*" \
  --include "*.sh"

# Update image names
refac ./k8s "myregistry/old-app" "myregistry/new-app" \
  --include "*.yaml" \
  --content-only
```

## Database and Schema Changes

### Update Table Names

**Scenario**: Rename database tables in SQL files and application code.

```bash
# SQL files only
refac ./sql "old_table" "new_table" \
  --include "*.sql" \
  --include "*.migration"

# Application code
refac ./src "old_table" "new_table" \
  --include "*.py" \
  --include "*.js" \
  --include "*.rb"
```

### Update Column References

**Scenario**: Rename a database column across your application.

```bash
# Preview changes across multiple file types
refac ./project "old_column_name" "new_column_name" \
  --dry-run \
  --include "*.sql" \
  --include "*.py" \
  --include "*.js"

# Apply with verbose output
refac ./project "old_column_name" "new_column_name" \
  --verbose \
  --include "*.sql" \
  --include "*.py" \
  --include "*.js"
```

## Patterns

### Using Regular Expressions

**Scenario**: Update version strings with regex patterns.

```bash
# Match version patterns like "v1.2.3"
refac ./docs "v1\\.\\d+\\.\\d+" "v2.0.0" \
  --regex \
  --include "*.md" \
  --include "*.txt"

# Case-insensitive function name updates
refac ./src "oldfunction" "newFunction" \
  --regex \
  --ignore-case \
  --include "*.js"
```

### Batch Operations with Scripts

**Scenario**: Multiple related replacements in sequence.

```bash
#!/bin/bash
# bulk-refactor.sh

# Array of old:new pairs
REPLACEMENTS=(
  "OldClass1:NewClass1"
  "OldClass2:NewClass2"
  "old_function:new_function"
  "OLD_CONSTANT:NEW_CONSTANT"
)

# Process each replacement
for replacement in "${REPLACEMENTS[@]}"; do
  IFS=':' read -r old new <<< "$replacement"
  echo "Replacing '$old' with '$new'..."
  
  refac ./src "$old" "$new" \
    --include "*.rs" \
    --include "*.toml" \
    --force
    
  if [ $? -ne 0 ]; then
    echo "Error processing $old -> $new"
    exit 1
  fi
done

echo "All replacements completed successfully!"
```

### Conditional Replacements

**Scenario**: Different replacements for different environments.

```bash
#!/bin/bash
# environment-update.sh

ENVIRONMENT=${1:-staging}

case $ENVIRONMENT in
  "staging")
    refac ./config "production.db.com" "staging.db.com" \
      --content-only \
      --include "*.env"
    ;;
  "production")
    refac ./config "staging.db.com" "production.db.com" \
      --content-only \
      --include "*.env"
    ;;
  *)
    echo "Usage: $0 [staging|production]"
    exit 1
    ;;
esac
```

## Safety and Testing

### Safe Refactoring Workflow

**Scenario**: A safe, step-by-step refactoring process.

```bash
#!/bin/bash
# safe-refactor.sh

OLD_NAME="$1"
NEW_NAME="$2"
PROJECT_DIR="$3"

if [ $# -ne 3 ]; then
  echo "Usage: $0 <old_name> <new_name> <project_dir>"
  exit 1
fi

# Step 1: Backup
echo "Creating backup..."
cp -r "$PROJECT_DIR" "${PROJECT_DIR}.backup"

# Step 2: Dry run
echo "Previewing changes..."
refac "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --dry-run --verbose

read -p "Continue with these changes? (y/N): " confirm
if [ "$confirm" != "y" ]; then
  echo "Aborted"
  exit 0
fi

# Step 3: Apply changes with backup
echo "Applying changes..."
refac "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --backup

# Step 4: Run tests (if available)
if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
  echo "Running Rust tests..."
  cd "$PROJECT_DIR" && cargo test
elif [ -f "$PROJECT_DIR/package.json" ]; then
  echo "Running Node.js tests..."
  cd "$PROJECT_DIR" && npm test
elif [ -f "$PROJECT_DIR/setup.py" ]; then
  echo "Running Python tests..."
  cd "$PROJECT_DIR" && python -m pytest
else
  echo "No test framework detected. Please run tests manually."
fi

echo "Refactoring completed!"
```

### Testing Changes

**Scenario**: Verify refactoring didn't break anything.

```bash
# Before refactoring
git add .
git commit -m "Before refactoring: rename oldname to newname"

# Apply refactoring
refac . "oldname" "newname" --backup --verbose

# Check what changed
git diff --name-only
git diff --stat

# Run your tests
cargo test  # Rust
npm test    # Node.js
pytest      # Python
make test   # Make-based projects

# If tests pass, commit
git add .
git commit -m "Refactor: rename oldname to newname"

# If tests fail, you can restore
git checkout .
# Or restore from backup files (*.bak)
```

## Performance Optimization

### Large Codebase Handling

**Scenario**: Refactoring a very large project efficiently.

```bash
# Use multiple threads for better performance
refac ./large-project "oldname" "newname" \
  --threads 8 \
  --progress always

# Limit scope to reduce processing time
refac ./large-project "oldname" "newname" \
  --max-depth 3 \
  --include "src/**" \
  --exclude "node_modules/**" \
  --exclude "target/**"

# Process in batches for very large projects
refac ./src "oldname" "newname" --threads 8
refac ./tests "oldname" "newname" --threads 8
refac ./docs "oldname" "newname" --threads 8
```

### Memory-Conscious Processing

**Scenario**: Handle large files without running out of memory.

```bash
# Process with limited depth
refac ./project "oldname" "newname" --max-depth 2

# Target specific file types to reduce scope
refac ./project "oldname" "newname" \
  --include "*.rs" \
  --exclude "*.log" \
  --exclude "*.tmp"
```

## Integration Examples

### CI/CD Pipeline Integration

**Scenario**: Automated refactoring checks in your pipeline.

```yaml
# .github/workflows/refactor-check.yml
name: Check for deprecated patterns

on: [push, pull_request]

jobs:
  check-deprecated:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Refac
        run: cargo install --git https://github.com/jowharshamshiri/refac
        
      - name: Check for deprecated patterns
        run: |
          # Check for deprecated function names
          if refac . "deprecated_function" "new_function" --dry-run --format json | jq -e '.summary.total_changes > 0'; then
            echo "Found deprecated patterns!"
            exit 1
          fi
```

### Git Hooks Integration

**Scenario**: Prevent commits with certain patterns.

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for debug statements
if refac . "console.log" "" --dry-run --format json | jq -e '.summary.total_changes > 0' >/dev/null; then
  echo "Error: Found console.log statements in code"
  refac . "console.log" "" --dry-run --include "*.js" --include "*.ts"
  echo "Please remove debug statements before committing"
  exit 1
fi

# Check for TODO comments (warning only)
if refac . "TODO" "" --dry-run --format json | jq -e '.summary.total_changes > 0' >/dev/null; then
  echo "Warning: Found TODO comments in code"
  refac . "TODO" "" --dry-run --include "*.rs" --include "*.js" --include "*.py"
fi

exit 0
```

## Troubleshooting Examples

### Debugging No Changes Found

**Scenario**: Refac reports no changes but you expect some.

```bash
# Use verbose mode to see what's happening
refac . "search_term" "replacement" --dry-run --verbose

# Check if the term exists
grep -r "search_term" . --include="*.rs"

# Verify include/exclude patterns
refac . "search_term" "replacement" \
  --dry-run \
  --verbose \
  --include "*" \
  --exclude "target/*"

# Test with broader patterns
refac . "search_term" "replacement" \
  --dry-run \
  --ignore-case \
  --include "*.rs"
```

### Handling Permission Issues

**Scenario**: Some files can't be modified due to permissions.

```bash
# Check file permissions
ls -la problematic_file

# Fix permissions if needed
chmod 644 *.rs

# Or run with appropriate permissions
sudo refac . "oldname" "newname" --backup

# Skip problematic files
refac . "oldname" "newname" \
  --exclude "readonly_files/*" \
  --verbose
```

---

## Tool Suite Workflows

### Full-Stack Application Refactoring

**Scenario**: Refactoring an entire application with proper backup, versioning, and cleanup.

```bash
#!/bin/bash
# complete-refactor.sh - Full application refactoring workflow

PROJECT_NAME="$1"
OLD_NAME="$2"
NEW_NAME="$3"

echo "=== Starting Refactoring Workflow ==="
echo "Project: $PROJECT_NAME"
echo "Refactor: $OLD_NAME -> $NEW_NAME"

# 1. Setup versioning if not already configured
echo "Setting up version tracking..."
if ! verbump status >/dev/null 2>&1; then
    verbump install
    echo "Verbump installed"
fi

# Record starting version
START_VERSION=$(verbump show 2>/dev/null | grep "Full Version" | cut -d: -f2 | xargs)
echo "Starting version: $START_VERSION"

# 2. Clean workspace
echo "Cleaning workspace..."
scrap target/ build/ *.log *.tmp node_modules/ .cache/
scrap archive "pre-refactor-backup-$(date +%s).tar.gz"
echo "Workspace cleaned and archived"

# 3. Preview changes
echo "Previewing changes..."
refac . "$OLD_NAME" "$NEW_NAME" --dry-run --verbose

# 4. Confirm with user
read -p "Apply these changes? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Refactoring cancelled"
    exit 0
fi

# 5. Apply refactoring in stages
echo "Applying refactoring..."

# Code files first
refac ./src "$OLD_NAME" "$NEW_NAME" --include "*.rs" --include "*.py" --include "*.js"

# Configuration files
refac ./config "$OLD_NAME" "$NEW_NAME" --content-only

# Documentation
refac ./docs "$OLD_NAME" "$NEW_NAME"

# Build files
refac . "$OLD_NAME" "$NEW_NAME" --include "*.toml" --include "*.json" --include "*.yaml"

# 6. Commit changes (triggers version bump)
echo "Committing changes..."
git add .
git commit -m "Refactor: $OLD_NAME -> $NEW_NAME

- Updated all source files
- Updated configuration
- Updated documentation
- Updated build files"

# 7. Show results
END_VERSION=$(cat version.txt 2>/dev/null || echo "unknown")
echo "=== Refactoring Complete ==="
echo "Version: $START_VERSION -> $END_VERSION"
echo "Files changed: $(git diff HEAD~1 --name-only | wc -l)"

# 8. Cleanup
echo "Final cleanup..."
scrap *.orig *.bak  # Remove any backup files created during refactoring
echo "Refactoring workflow complete!"
```

### Team Development Workflow

**Scenario**: Establishing a consistent workflow for a development team.

```bash
# team-setup.sh - Setup Refac Tools for team development

# 1. Install all tools for the team
./install.sh

# 2. Setup project-wide versioning
verbump install

# 3. Create team workspace management script
cat > team-cleanup.sh << 'EOF'
#!/bin/bash
# Team workspace cleanup

echo "Cleaning workspace..."
scrap find "*.tmp" "*.log" "*~" ".DS_Store"
scrap clean --days 3

echo "Archiving old experiments..."
scrap find "experiment_*" "test_*" "old_*"
scrap archive "team-cleanup-$(date +%Y%m%d).tar.gz" --remove

echo "Workspace cleaned!"
EOF

chmod +x team-cleanup.sh

# 4. Create refactoring safety script
cat > safe-refactor.sh << 'EOF'
#!/bin/bash
OLD="$1"
NEW="$2"

if [ -z "$OLD" ] || [ -z "$NEW" ]; then
    echo "Usage: $0 <old_string> <new_string>"
    exit 1
fi

# Backup current state
scrap archive "backup-$(date +%s).tar.gz"

# Preview changes
echo "Previewing changes..."
refac . "$OLD" "$NEW" --dry-run

# Apply with confirmation
read -p "Apply changes? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    refac . "$OLD" "$NEW"
    git add . && git commit -m "Refactor: $OLD -> $NEW"
    echo "Refactoring complete!"
else
    echo "Cancelled"
fi
EOF

chmod +x safe-refactor.sh

echo "Team development tools configured!"
echo "Usage:"
echo "  ./team-cleanup.sh     - Clean workspace"
echo "  ./safe-refactor.sh old new - Safe refactoring"
```

### Release Pipeline Integration

**Scenario**: Integrating Refac Tools into a CI/CD pipeline.

```bash
# release-pipeline.sh - Automated release preparation

# 1. Clean build artifacts
scrap target/ build/ *.log
scrap purge --force  # Clear all previous scrap items

# 2. Update environment configurations
refac ./config "development" "production" --content-only
refac ./config "debug=true" "debug=false" --content-only
refac ./config "localhost" "$PROD_HOST" --content-only

# 3. Update version and create release
if verbump status; then
    # Version automatically updated on commit
    git add .
    git commit -m "Prepare production release"
    
    # Tag the release
    RELEASE_VERSION=$(cat version.txt)
    git tag "v$RELEASE_VERSION"
    
    echo "Release v$RELEASE_VERSION prepared"
else
    echo "Warning: Verbump not configured"
fi

# 4. Generate release artifacts
mkdir -p releases/
tar -czf "releases/app-v$RELEASE_VERSION.tar.gz" \
    --exclude=target --exclude=.git --exclude=.scrap .

echo "Release pipeline complete!"
echo "Artifact: releases/app-v$RELEASE_VERSION.tar.gz"
```

These examples demonstrate how the Refac Tools suite provides a solution for development workflow management, combining safe refactoring, intelligent file management, and automatic version tracking.