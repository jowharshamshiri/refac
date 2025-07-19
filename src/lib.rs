pub mod refac;
pub mod scrap;
pub mod verbump;

use anyhow::{Context, Result};
use std::path::Path;

/// Read version from version.txt file at project root
pub fn get_version() -> &'static str {
    include_str!("../version.txt").trim()
}

// Re-export from refac module for backward compatibility
pub use refac::cli as cli;
pub use refac::cli::{Args, Mode};
pub use refac::rename_engine::RenameEngine;

// Re-export from scrap module
pub use scrap::scrap_common::{ScrapMetadata, ScrapEntry};

// Re-export from verbump module
pub use verbump::{VerbumpConfig, VersionInfo};

/// Main entry point for the refac operation
pub fn run_refac(args: Args) -> Result<()> {
    let engine = RenameEngine::new(args)?;
    engine.execute()
}

/// Represents a file or directory that needs to be processed
#[derive(Debug, Clone)]
pub struct RenameItem {
    pub original_path: std::path::PathBuf,
    pub new_path: std::path::PathBuf,
    pub item_type: ItemType,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    File,
    Directory,
}

/// Configuration for the rename operation
#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub root_dir: std::path::PathBuf,
    pub old_string: String,
    pub new_string: String,
    pub dry_run: bool,
    pub force: bool,
    pub verbose: bool,
    pub follow_symlinks: bool,
    pub backup: bool,
}

impl RenameConfig {
    pub fn new<P: AsRef<Path>>(
        root_dir: P,
        old_string: String,
        new_string: String,
    ) -> Result<Self> {
        let root_path = root_dir.as_ref().canonicalize()
            .with_context(|| format!("Failed to resolve root directory: {}", root_dir.as_ref().display()))?;
        
        if old_string.is_empty() {
            anyhow::bail!("Old string cannot be empty");
        }
        
        if new_string.is_empty() {
            anyhow::bail!("New string cannot be empty");
        }
        
        if new_string.contains('/') || new_string.contains('\\') {
            anyhow::bail!("New string cannot contain path separators");
        }
        
        Ok(Self {
            root_dir: root_path,
            old_string,
            new_string,
            dry_run: false,
            force: false,
            verbose: false,
            follow_symlinks: false,
            backup: false,
        })
    }
    
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
    
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
        self.follow_symlinks = follow_symlinks;
        self
    }
    
    pub fn with_backup(mut self, backup: bool) -> Self {
        self.backup = backup;
        self
    }
}

/// Statistics about the rename operation
#[derive(Debug, Default, Clone)]
pub struct RenameStats {
    pub files_with_content_changes: usize,
    pub files_renamed: usize,
    pub directories_renamed: usize,
    pub files_processed: usize,
    pub errors: Vec<String>,
}

impl RenameStats {
    pub fn total_changes(&self) -> usize {
        self.files_with_content_changes + self.files_renamed + self.directories_renamed
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

/// Utility functions
pub mod utils {
    use std::path::Path;
    
    /// Calculate the depth of a path relative to a root
    pub fn calculate_depth(path: &Path, root: &Path) -> usize {
        path.strip_prefix(root)
            .map(|p| p.components().count())
            .unwrap_or(0)
    }
    
    /// Check if a string contains the target pattern
    pub fn contains_pattern(text: &str, pattern: &str) -> bool {
        text.contains(pattern)
    }
    
    /// Replace all occurrences of old with new in the string
    pub fn replace_all(text: &str, old: &str, new: &str) -> String {
        text.replace(old, new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_rename_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RenameConfig::new(
            temp_dir.path(),
            "old".to_string(),
            "new".to_string(),
        ).unwrap();
        
        assert_eq!(config.old_string, "old");
        assert_eq!(config.new_string, "new");
        assert!(!config.dry_run);
        assert!(!config.force);
    }
    
    #[test]
    fn test_rename_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Empty old string should fail
        assert!(RenameConfig::new(temp_dir.path(), "".to_string(), "new".to_string()).is_err());
        
        // Empty new string should fail
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "".to_string()).is_err());
        
        // Path separator in new string should fail
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "new/path".to_string()).is_err());
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "new\\path".to_string()).is_err());
    }
    
    #[test]
    fn test_utils_replace_all() {
        assert_eq!(utils::replace_all("hello world", "l", "x"), "hexxo worxd");
        assert_eq!(utils::replace_all("test test test", "test", "demo"), "demo demo demo");
        assert_eq!(utils::replace_all("no match", "xyz", "abc"), "no match");
    }
    
    #[test]
    fn test_utils_contains_pattern() {
        assert!(utils::contains_pattern("hello world", "hello"));
        assert!(utils::contains_pattern("hello world", "world"));
        assert!(utils::contains_pattern("hello world", "o w"));
        assert!(!utils::contains_pattern("hello world", "xyz"));
    }
    
    #[test]
    fn test_get_version() {
        let version = get_version();
        // Should not be empty
        assert!(!version.is_empty());
        // Should be a valid version format (x.y.z)
        assert!(version.contains('.'));
        // Should match current version in version.txt
        assert_eq!(version, "0.27.19182");
    }
    
    #[test]
    fn test_version_consistency() {
        // Test that version.txt exists and is readable
        let version_content = std::fs::read_to_string("version.txt").expect("version.txt should exist");
        let file_version = version_content.trim();
        
        // Should match what get_version() returns
        assert_eq!(get_version(), file_version);
    }
}