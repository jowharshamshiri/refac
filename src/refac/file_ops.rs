use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use super::binary_detector::BinaryDetector;

/// File operations for the refac tool
pub struct FileOperations {
    binary_detector: BinaryDetector,
    backup_enabled: bool,
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations {
    pub fn new() -> Self {
        Self {
            binary_detector: BinaryDetector::default(),
            backup_enabled: false,
        }
    }

    pub fn with_backup(mut self, enabled: bool) -> Self {
        self.backup_enabled = enabled;
        self
    }

    /// Replace content in a file
    pub fn replace_content<P: AsRef<Path>>(
        &self,
        file_path: P,
        old_string: &str,
        new_string: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        // Read the file content
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Check if the file contains the target string
        if !content.contains(old_string) {
            return Ok(false);
        }

        // Create backup if enabled
        if self.backup_enabled {
            self.create_backup(file_path)?;
        }

        // Replace content
        let new_content = content.replace(old_string, new_string);

        // Write the modified content back
        fs::write(file_path, new_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        Ok(true)
    }

    /// Replace content in a file using streaming for large files
    pub fn replace_content_streaming<P: AsRef<Path>>(
        &self,
        file_path: P,
        old_string: &str,
        new_string: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        // Create backup if enabled
        if self.backup_enabled {
            self.create_backup(file_path)?;
        }

        let temp_file_path = file_path.with_extension("tmp");
        let mut modified = false;

        {
            let input_file = File::open(file_path)
                .with_context(|| format!("Failed to open input file: {}", file_path.display()))?;
            let reader = BufReader::new(input_file);

            let output_file = File::create(&temp_file_path)
                .with_context(|| format!("Failed to create temp file: {}", temp_file_path.display()))?;
            let mut writer = BufWriter::new(output_file);

            for line in reader.lines() {
                let line = line.with_context(|| {
                    format!("Failed to read line from file: {}", file_path.display())
                })?;
                
                let new_line = if line.contains(old_string) {
                    modified = true;
                    line.replace(old_string, new_string)
                } else {
                    line
                };

                writeln!(writer, "{}", new_line).with_context(|| {
                    format!("Failed to write to temp file: {}", temp_file_path.display())
                })?;
            }

            writer.flush().with_context(|| {
                format!("Failed to flush temp file: {}", temp_file_path.display())
            })?;
        }

        if modified {
            // Replace the original file with the modified one
            fs::rename(&temp_file_path, file_path).with_context(|| {
                format!(
                    "Failed to replace original file {} with temp file {}",
                    file_path.display(),
                    temp_file_path.display()
                )
            })?;
        } else {
            // Remove the temp file since no changes were made
            let _ = fs::remove_file(&temp_file_path);
        }

        Ok(modified)
    }

    /// Move/rename a file or directory
    pub fn move_item<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Ensure the target directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        fs::rename(from, to).with_context(|| {
            format!(
                "Failed to move {} to {}",
                from.display(),
                to.display()
            )
        })?;

        Ok(())
    }

    /// Copy a file
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Ensure the target directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        fs::copy(from, to).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                from.display(),
                to.display()
            )
        })?;

        Ok(())
    }

    /// Create a backup of a file
    pub fn create_backup<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let file_path = file_path.as_ref();
        let backup_path = self.generate_backup_path(file_path)?;

        fs::copy(file_path, &backup_path).with_context(|| {
            format!(
                "Failed to create backup from {} to {}",
                file_path.display(),
                backup_path.display()
            )
        })?;

        Ok(backup_path)
    }

    /// Generate a unique backup file path
    fn generate_backup_path<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let file_path = file_path.as_ref();
        let mut backup_path = file_path.with_extension(
            format!(
                "{}.bak",
                file_path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            )
        );

        // If backup already exists, find a unique name
        let mut counter = 1;
        while backup_path.exists() {
            backup_path = file_path.with_extension(
                format!(
                    "{}.bak.{}",
                    file_path.extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or(""),
                    counter
                )
            );
            counter += 1;
        }

        Ok(backup_path)
    }

    /// Check if a file contains a specific string
    pub fn file_contains_string<P: AsRef<Path>>(
        &self,
        file_path: P,
        search_string: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        Ok(content.contains(search_string))
    }

    /// Count occurrences of a string in a file
    pub fn count_string_occurrences<P: AsRef<Path>>(
        &self,
        file_path: P,
        search_string: &str,
    ) -> Result<usize> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(0);
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        Ok(content.matches(search_string).count())
    }

    /// Get file size
    pub fn get_file_size<P: AsRef<Path>>(&self, file_path: P) -> Result<u64> {
        let metadata = fs::metadata(file_path.as_ref())
            .with_context(|| format!("Failed to get metadata for: {}", file_path.as_ref().display()))?;
        Ok(metadata.len())
    }

    /// Check if a path is a text file
    pub fn is_text_file<P: AsRef<Path>>(&self, file_path: P) -> Result<bool> {
        self.binary_detector.is_text_file(file_path)
    }

    /// Get the reason why a file is considered binary
    pub fn get_binary_reason<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<String>> {
        self.binary_detector.get_binary_reason(file_path)
    }

    /// Safely create a directory and all its parents
    pub fn create_dir_all<P: AsRef<Path>>(&self, dir_path: P) -> Result<()> {
        fs::create_dir_all(dir_path.as_ref()).with_context(|| {
            format!("Failed to create directory: {}", dir_path.as_ref().display())
        })
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Check if a path is a directory
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Get file permissions (Unix-style)
    #[cfg(unix)]
    pub fn get_permissions<P: AsRef<Path>>(&self, path: P) -> Result<u32> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path.as_ref())
            .with_context(|| format!("Failed to get metadata for: {}", path.as_ref().display()))?;
        Ok(metadata.permissions().mode())
    }

    /// Set file permissions (Unix-style)
    #[cfg(unix)]
    pub fn set_permissions<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path.as_ref())?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path.as_ref(), perms)
            .with_context(|| format!("Failed to set permissions for: {}", path.as_ref().display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_replace_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Hello world")?;
        writeln!(file, "This is a test file")?;
        writeln!(file, "Hello again")?;

        // Replace content
        let modified = file_ops.replace_content(&test_file, "Hello", "Hi")?;
        assert!(modified);

        // Check the result
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("Hi world"));
        assert!(content.contains("Hi again"));
        assert!(!content.contains("Hello"));

        Ok(())
    }

    #[test]
    fn test_replace_content_no_match() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "This is a test file")?;

        // Try to replace non-existent content
        let modified = file_ops.replace_content(&test_file, "nonexistent", "replacement")?;
        assert!(!modified);

        // Content should be unchanged
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("This is a test file"));

        Ok(())
    }

    #[test]
    fn test_replace_content_streaming() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        for i in 0..100 {
            writeln!(file, "Line {} with pattern target here", i)?;
        }

        // Replace content using streaming
        let modified = file_ops.replace_content_streaming(&test_file, "target", "replacement")?;
        assert!(modified);

        // Check the result
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("replacement"));
        assert!(!content.contains("target"));

        Ok(())
    }

    #[test]
    fn test_move_item() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let source_file = temp_dir.path().join("source.txt");
        let mut file = File::create(&source_file)?;
        writeln!(file, "Test content")?;

        // Move the file
        let target_file = temp_dir.path().join("subdir").join("target.txt");
        file_ops.move_item(&source_file, &target_file)?;

        // Check that the file was moved
        assert!(!source_file.exists());
        assert!(target_file.exists());

        // Check content is preserved
        let content = fs::read_to_string(&target_file)?;
        assert!(content.contains("Test content"));

        Ok(())
    }

    #[test]
    fn test_copy_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let source_file = temp_dir.path().join("source.txt");
        let mut file = File::create(&source_file)?;
        writeln!(file, "Test content")?;

        // Copy the file
        let target_file = temp_dir.path().join("subdir").join("target.txt");
        file_ops.copy_file(&source_file, &target_file)?;

        // Check that both files exist
        assert!(source_file.exists());
        assert!(target_file.exists());

        // Check content is the same
        let source_content = fs::read_to_string(&source_file)?;
        let target_content = fs::read_to_string(&target_file)?;
        assert_eq!(source_content, target_content);

        Ok(())
    }

    #[test]
    fn test_create_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Original content")?;

        // Create backup
        let backup_path = file_ops.create_backup(&test_file)?;

        // Check that backup was created
        assert!(backup_path.exists());
        
        // Check backup content
        let backup_content = fs::read_to_string(&backup_path)?;
        assert!(backup_content.contains("Original content"));

        // Check original file still exists
        assert!(test_file.exists());

        Ok(())
    }

    #[test]
    fn test_file_contains_string() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "This file contains a specific pattern")?;

        // Test string that exists
        assert!(file_ops.file_contains_string(&test_file, "specific pattern")?);
        
        // Test string that doesn't exist
        assert!(!file_ops.file_contains_string(&test_file, "nonexistent")?);

        Ok(())
    }

    #[test]
    fn test_count_string_occurrences() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "test test test")?;
        writeln!(file, "another line")?;
        writeln!(file, "test again")?;

        // Count occurrences
        let count = file_ops.count_string_occurrences(&test_file, "test")?;
        assert_eq!(count, 4); // 3 in first line + 1 in last line

        // Count non-existent string
        let count = file_ops.count_string_occurrences(&test_file, "nonexistent")?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[test]
    fn test_backup_with_replace_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new().with_backup(true);
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Original content with target")?;

        // Replace content (should create backup automatically)
        let modified = file_ops.replace_content(&test_file, "target", "replacement")?;
        assert!(modified);

        // Check that backup was created
        let backup_path = test_file.with_extension("txt.bak");
        assert!(backup_path.exists());

        // Check backup contains original content
        let backup_content = fs::read_to_string(&backup_path)?;
        assert!(backup_content.contains("target"));

        // Check main file contains new content
        let main_content = fs::read_to_string(&test_file)?;
        assert!(main_content.contains("replacement"));
        assert!(!main_content.contains("target"));

        Ok(())
    }

    #[test]
    fn test_get_file_size() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file with known content
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!")?; // 13 bytes

        let size = file_ops.get_file_size(&test_file)?;
        assert_eq!(size, 13);

        Ok(())
    }

    #[test]
    fn test_utility_functions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create test file and directory
        let test_file = temp_dir.path().join("test.txt");
        let test_dir = temp_dir.path().join("test_dir");
        
        File::create(&test_file)?;
        fs::create_dir(&test_dir)?;

        // Test existence checks
        assert!(file_ops.exists(&test_file));
        assert!(file_ops.exists(&test_dir));
        assert!(!file_ops.exists(temp_dir.path().join("nonexistent")));

        // Test type checks
        assert!(file_ops.is_file(&test_file));
        assert!(!file_ops.is_file(&test_dir));
        assert!(file_ops.is_dir(&test_dir));
        assert!(!file_ops.is_dir(&test_file));

        Ok(())
    }
}