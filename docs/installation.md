---
layout: default
title: Installation Guide
---

# Installation Guide

This guide covers multiple ways to install the Refac Tools suite on your system.

## Prerequisites

- **Rust toolchain** (if building from source)
- **Git** (for cloning the repository)

## Easy Installation (Recommended)

The fastest way to install all Refac Tools:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/refac.git
cd refac

# Run the installation script
./install.sh
```

The installation script will:
- Build all tools (`refac`, `scrap`, `unscrap`, `verbump`) in release mode
- Install to `~/.local/bin` by default
- Check for updates on subsequent runs

### Installation Options

```bash
./install.sh --help                    # See all options
./install.sh -d /usr/local/bin         # Install system-wide
./install.sh --force                   # Force reinstall
./install.sh --verbose                 # Verbose output
```

### Uninstall

```bash
./uninstall.sh                         # Remove all tools
./uninstall.sh -d /usr/local/bin       # Remove from custom directory
```

## Manual Installation

For users who prefer manual control:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/refac.git
cd refac

# Build in release mode
cargo build --release

# Install all tools
cargo install --path .

# Or install individual tools
cargo install --path . --bin refac
cargo install --path . --bin scrap
cargo install --path . --bin unscrap
cargo install --path . --bin verbump
```

The binaries will be installed to your Cargo bin directory (typically `~/.cargo/bin/` on Unix systems).

### Option 3: Development Build

For development or testing the latest changes:

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/refac.git
cd refac

# Run without installing
cargo run -- --help

# Or build in debug mode
cargo build
```

## Verification

Verify your installation:

```bash
# Check versions of all tools
refac --version
scrap --version
unscrap --version
verbump --version

# View help for each tool
refac --help
scrap --help
unscrap --help
verbump --help

# Test basic functionality
refac . "test" "test" --dry-run    # Test string replacement
scrap --help                       # Test scrap functionality
verbump status                     # Test verbump (outside git repo)
```

## Platform-Specific Notes

### Linux

On some distributions, you may need to install additional dependencies:

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install build-essential
```

**CentOS/RHEL/Fedora:**

```bash
sudo yum groupinstall "Development Tools"
# or on newer versions:
sudo dnf groupinstall "Development Tools"
```

### macOS

Install Rust via Homebrew or rustup:

```bash
# Via Homebrew
brew install rust

# Via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Ensure you have the Microsoft C++ Build Tools installed
3. Follow the standard installation process

## Updating

To update to the latest version:

```bash
# If installed via install.sh script
cd /path/to/refac
git pull
./install.sh --force

# If installed via cargo install
cd /path/to/refac
git pull
cargo install --path . --force

# If using pre-built binaries
# Download the latest release and replace all binaries
```

## Uninstalling

To remove Refac:

```bash
# If installed via install.sh script
./uninstall.sh

# If installed via cargo install (individual tools)
cargo uninstall refac
cargo uninstall scrap
cargo uninstall unscrap
cargo uninstall verbump

# If installed manually (remove all binaries)
sudo rm /usr/local/bin/{refac,scrap,unscrap,verbump}  # Linux/macOS
```

## Building from Source (Advanced)

### Custom Features

The default build includes all features. You can customize the build:

```bash
# Minimal build (no progress bars)
cargo build --release --no-default-features

# Development build with debug symbols
cargo build
```

### Build Options

```bash
# Build with optimizations for current CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Static binary (Linux)
cargo build --release --target x86_64-unknown-linux-musl

# Cross-compilation example (requires cross-compilation setup)
cargo build --release --target x86_64-pc-windows-gnu
```

### Running Tests

Before installation, you can run the test suite:

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo test --all-features

# Run integration tests
cargo test --test integration_tests
```

## Troubleshooting

### Common Issues

**"cargo: command not found"**

- Install Rust and Cargo from [rustup.rs](https://rustup.rs/)
- Ensure `~/.cargo/bin` is in your PATH

**"Permission denied" on installation**

- Use `sudo` for system-wide installation
- Or install to a user directory: `cargo install --path . --root ~/.local`

**"Binary not found after installation"**

- Check that `~/.cargo/bin` is in your PATH
- Add to your shell profile: `export PATH="$HOME/.cargo/bin:$PATH"`

**Build errors on older systems**

- Update Rust: `rustup update`
- Ensure you have a compatible C compiler

### Platform-Specific Issues

**Linux: "error: Microsoft Visual C++ 14.0 is required"**

- This error is actually for Windows - check you're building on the correct platform

**macOS: "xcrun: error: invalid active developer path"**

```bash
# Install Xcode command line tools
xcode-select --install
```

**Windows: "error: linker 'link.exe' not found"**

- Install Microsoft C++ Build Tools
- Or install Visual Studio with C++ support

## Getting Help

If you encounter issues:

1. Check the [troubleshooting section]({{ '/troubleshooting/' | relative_url }})
2. Search [existing issues](https://github.com/jowharshamshiri/refac/issues)
3. Create a [new issue](https://github.com/jowharshamshiri/refac/issues/new) with:
   - Your operating system and version
   - Rust version (`rustc --version`)
   - error message
   - Steps to reproduce

## Next Steps

Once installed, check out:

- [Usage Guide]({{ '/usage/' | relative_url }}) - Learn how to use Refac effectively
- [Command Reference]({{ '/api-reference/' | relative_url }}) - command documentation
- [Examples]({{ '/examples/' | relative_url }}) - Real-world usage examples
