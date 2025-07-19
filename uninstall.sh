#!/bin/bash

# Refac Tools Uninstallation Script
# This script removes all refac tools (refac, scrap, unscrap, verbump)

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# Parse command line arguments
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
FORCE_REMOVE=false
VERBOSE=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Uninstall Refac Tools (refac, scrap, unscrap, verbump)"
    echo ""
    echo "OPTIONS:"
    echo "  -d, --dir DIR        Installation directory to remove from (default: $DEFAULT_INSTALL_DIR)"
    echo "  -f, --force          Force removal without confirmation"
    echo "  -v, --verbose        Verbose output"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                           # Remove from default location"
    echo "  $0 -d /usr/local/bin         # Remove from system directory"
    echo "  $0 --force                   # Remove without confirmation"
}

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE_REMOVE=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Verbose output function
verbose_log() {
    if [ "$VERBOSE" = true ]; then
        log "$1"
    fi
}

# Find installed binaries
find_installed_binaries() {
    local binaries=("refac" "scrap" "unscrap" "verbump")
    local found_binaries=()
    
    for binary in "${binaries[@]}"; do
        local binary_path="$INSTALL_DIR/$binary"
        if [ -f "$binary_path" ]; then
            found_binaries+=("$binary_path")
            verbose_log "Found: $binary_path"
        fi
    done
    
    # Also check for shell integration script
    local shell_script="$INSTALL_DIR/scrap-shell-integration.sh"
    if [ -f "$shell_script" ]; then
        found_binaries+=("$shell_script")
        verbose_log "Found: $shell_script"
    fi
    
    echo "${found_binaries[@]}"
}

# Get versions of installed tools
get_installed_info() {
    local binaries=("refac" "scrap" "unscrap" "verbump")
    
    for binary in "${binaries[@]}"; do
        local binary_path="$INSTALL_DIR/$binary"
        if [ -x "$binary_path" ]; then
            local version=$("$binary_path" --version 2>/dev/null | head -n1 || echo "unknown version")
            log "Found $binary: $version"
        fi
    done
}

# Confirm removal
confirm_removal() {
    if [ "$FORCE_REMOVE" = true ]; then
        return 0
    fi
    
    echo ""
    warn "This will remove the following refac tools from $INSTALL_DIR:"
    echo ""
    
    local binaries_to_remove=($(find_installed_binaries))
    
    if [ ${#binaries_to_remove[@]} -eq 0 ]; then
        log "No refac tools found in $INSTALL_DIR"
        exit 0
    fi
    
    for binary in "${binaries_to_remove[@]}"; do
        echo "  - $(basename "$binary")"
    done
    
    echo ""
    read -p "Are you sure you want to continue? (y/N) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Uninstallation cancelled"
        exit 0
    fi
}

# Remove binaries
remove_binaries() {
    local binaries_to_remove=($(find_installed_binaries))
    local removed_count=0
    
    if [ ${#binaries_to_remove[@]} -eq 0 ]; then
        warn "No refac tools found in $INSTALL_DIR"
        return 0
    fi
    
    log "Removing refac tools from $INSTALL_DIR..."
    
    for binary in "${binaries_to_remove[@]}"; do
        if [ -f "$binary" ]; then
            verbose_log "Removing $(basename "$binary")..."
            rm -f "$binary"
            if [ ! -f "$binary" ]; then
                success "Removed $(basename "$binary")"
                ((removed_count++))
            else
                error "Failed to remove $(basename "$binary")"
            fi
        fi
    done
    
    if [ $removed_count -gt 0 ]; then
        success "Successfully removed $removed_count file(s)"
    fi
}

# Check for remaining shell integration
check_shell_integration() {
    local profile_files=(
        "$HOME/.bashrc"
        "$HOME/.bash_profile"
        "$HOME/.zshrc"
        "$HOME/.profile"
    )
    
    local found_integration=false
    
    for profile in "${profile_files[@]}"; do
        if [ -f "$profile" ] && grep -q "scrap-shell-integration" "$profile" 2>/dev/null; then
            warn "Found shell integration in $profile"
            warn "You may want to remove the line: source $INSTALL_DIR/scrap-shell-integration.sh"
            found_integration=true
        fi
    done
    
    if [ "$found_integration" = false ]; then
        log "No shell integration references found in common profile files"
    fi
}

# Main uninstallation function
main() {
    log "Refac Tools Uninstallation"
    log "Checking installation directory: $INSTALL_DIR"
    
    if [ ! -d "$INSTALL_DIR" ]; then
        warn "Installation directory does not exist: $INSTALL_DIR"
        exit 0
    fi
    
    get_installed_info
    confirm_removal
    remove_binaries
    check_shell_integration
    
    echo ""
    success "🗑️  Refac Tools uninstallation completed!"
    log "If you want to reinstall, run: ./install.sh"
}

# Run the main function
main "$@"