---
layout: default
title: Verbump Guide - Automatic Version Management
---

# Verbump Guide

Verbump is an automatic version management tool that integrates with Git to provide semantic versioning based on your repository's commit history. It calculates version numbers using git tags, commit counts, and change statistics, making it perfect for continuous integration workflows.

## How It Works

Verbump uses a three-part versioning scheme based on your Git repository:

- **Major Version**: Extracted from the most recent Git tag (e.g., `v1.0` → `1.0`)
- **Minor Version**: Number of commits since the last tag
- **Patch Version**: Total number of changes (line additions + deletions) across all commits

**Final Version Format**: `{major}.{minor}.{patch}`

### Example Version Calculation

```bash
# Repository state:
# - Latest tag: v2.1
# - Commits since tag: 5
# - Total changes: 247

# Result: 2.1.5.247
```

## Installation and Setup

### 1. Install Verbump

First, ensure verbump is installed as part of the Refac Tools suite:

```bash
# Install all tools including verbump
./install.sh

# Or install just verbump
cargo install --path . --bin verbump
```

### 2. Install Git Hook

Navigate to your Git repository and install the pre-commit hook:

```bash
cd your-git-repo
verbump install
```

This creates a pre-commit hook that automatically updates your version file before each commit.

### 3. Configure (Optional)

Create a `.verbump.json` configuration file in your repository root:

```json
{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}
```

**Configuration Options:**
- `enabled`: Enable/disable automatic version updates
- `version_file`: Path to the file where version should be written (default: `version.txt`)

## Basic Usage

### Install Hook
```bash
# Install pre-commit hook
verbump install

# Force reinstall (if already installed)
verbump install --force
```

### Show Version Information
```bash
# Display current version breakdown
verbump show
```

Output example:
```
Current Version Information:
  Major (tag): v1.2
  Minor (commits since tag): 3
  Patch (total changes): 156
  Full Version: 1.2.3.156
```

### Manual Version Update
```bash
# Update version file manually
verbump update

# Force update (even outside git repo)
verbump update --force
```

### Check Status
```bash
# Show verbump status and configuration
verbump status
```

Output example:
```
Verbump Status:
  Git Repository: ✓
  Hook Installed: ✓
  Enabled: ✓
  Version File: version.txt
  Current Version: 1.2.3.156
  Version File Exists: ✓
```

### Uninstall Hook
```bash
# Remove verbump from pre-commit hooks
verbump uninstall
```

## Workflow Integration

### Automatic Mode (Recommended)

1. Install the git hook once: `verbump install`
2. Work normally - commit as usual
3. Version file is automatically updated before each commit
4. Version file changes are automatically staged

### Manual Mode

If you prefer manual control:

1. Configure: Set `"enabled": false` in `.verbump.json`
2. Update manually: Run `verbump update` when needed
3. Commit the version file changes manually

### CI/CD Integration

Include version information in your build scripts:

```bash
#!/bin/bash
# Get current version
VERSION=$(cat version.txt)
echo "Building version: $VERSION"

# Use in build process
docker build -t myapp:$VERSION .
```

## Advanced Features

### Custom Version Files

Configure different version file paths:

```json
{
  "version_file": "src/version.rs"
}
```

### Version File Formats

Verbump writes just the version number to the file:

```
1.2.3.156
```

You can incorporate this into different file formats using scripts:

**Rust example:**
```bash
# Update Rust version constant
echo "pub const VERSION: &str = \"$(cat version.txt)\";" > src/version.rs
```

**JavaScript example:**
```bash
# Update package.json version
jq --arg version "$(cat version.txt)" '.version = $version' package.json > package.json.tmp
mv package.json.tmp package.json
```

### Multiple Repositories

Each repository can have its own verbump configuration:

```bash
# Project A
cd project-a
verbump install
echo '{"version_file": "VERSION"}' > .verbump.json

# Project B  
cd project-b
verbump install
echo '{"version_file": "src/version.txt"}' > .verbump.json
```

## Troubleshooting

### Hook Not Running

If the version isn't updating automatically:

1. Check if hook is installed: `verbump status`
2. Verify hook file exists: `ls -la .git/hooks/pre-commit`
3. Ensure hook is executable: `chmod +x .git/hooks/pre-commit`
4. Check if verbump is in PATH: `which verbump`

### Version Not Updating

If version calculations seem wrong:

1. Check git repository status: `git status`
2. Verify tags exist: `git tag -l`
3. Check commit history: `git log --oneline`
4. Test manually: `verbump show`

### Configuration Issues

If configuration isn't working:

1. Validate JSON syntax: `cat .verbump.json | jq .`
2. Check file permissions: `ls -la .verbump.json`
3. Verify configuration is in repository root

### Removing Verbump

To completely remove verbump from a repository:

```bash
# Remove git hook
verbump uninstall

# Remove configuration (optional)
rm .verbump.json

# Remove version file (optional)
rm version.txt
```

## Logging

Verbump logs all actions to `.verbump.log` in your repository root:

```bash
# View recent actions
tail -f .verbump.log
```

Log format:
```
[2024-07-19 14:30:15] Created new pre-commit hook: /path/to/repo/.git/hooks/pre-commit
[2024-07-19 14:30:45] Updated version to: 1.2.3.156 (file: version.txt)
```

## Best Practices

1. **Install Early**: Set up verbump when creating a new repository
2. **Tag Releases**: Create git tags for major releases (`git tag v1.0`)
3. **Consistent Workflow**: Let the hook handle versioning automatically
4. **CI Integration**: Use version.txt in your build and deployment scripts
5. **Backup Hooks**: Document verbump usage for team members

## Integration Examples

### Docker Build
```dockerfile
# Copy version file
COPY version.txt /app/version.txt

# Use in build args
ARG VERSION
RUN echo "Building version: $VERSION"
```

### GitHub Actions
```yaml
- name: Get Version
  id: version
  run: echo "version=$(cat version.txt)" >> $GITHUB_OUTPUT

- name: Create Release
  uses: actions/create-release@v1
  with:
    tag_name: v${{ steps.version.outputs.version }}
    release_name: Release ${{ steps.version.outputs.version }}
```

### Makefile Integration
```makefile
VERSION := $(shell cat version.txt)

build:
	@echo "Building version $(VERSION)"
	cargo build --release

release: build
	git tag v$(VERSION)
	git push origin v$(VERSION)
```

## See Also

- [Installation Guide]({{ '/installation/' | relative_url }}) - Installing the complete tool suite
- [Getting Started]({{ '/getting-started/' | relative_url }}) - Quick start with all tools
- [API Reference]({{ '/api-reference/' | relative_url }}) - Complete command reference