use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::RenameItem;

/// Detects naming collisions in rename operations
#[derive(Debug)]
pub struct CollisionDetector {
    /// Map of new paths to their sources for collision detection
    target_paths: HashMap<PathBuf, Vec<PathBuf>>,
    /// Set of paths that already exist in the filesystem
    existing_paths: HashSet<PathBuf>,
    /// Collisions found during detection
    collisions: Vec<Collision>,
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub collision_type: CollisionType,
    pub target_path: PathBuf,
    pub source_paths: Vec<PathBuf>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollisionType {
    /// Multiple sources trying to rename to the same target
    MultipleSourcesSameTarget,
    /// Target path already exists in filesystem
    TargetAlreadyExists,
    /// Source and target are the same (no-op)
    SourceEqualsTarget,
    /// Case-only difference on case-insensitive filesystem
    CaseOnlyDifference,
    /// Directory trying to rename to existing file path
    DirectoryToFile,
    /// File trying to rename to existing directory path
    FileToDirectory,
}

impl Default for CollisionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionDetector {
    pub fn new() -> Self {
        Self {
            target_paths: HashMap::new(),
            existing_paths: HashSet::new(),
            collisions: Vec::new(),
        }
    }

    /// Add a rename operation to check for collisions
    pub fn add_rename(&mut self, source: PathBuf, target: PathBuf) {
        self.target_paths.entry(target).or_insert_with(Vec::new).push(source);
    }

    /// Add multiple rename operations
    pub fn add_renames(&mut self, items: &[RenameItem]) {
        for item in items {
            self.add_rename(item.original_path.clone(), item.new_path.clone());
        }
    }

    /// Add an existing filesystem path to check against
    pub fn add_existing_path<P: AsRef<Path>>(&mut self, path: P) {
        self.existing_paths.insert(path.as_ref().to_path_buf());
    }

    /// Scan a directory to populate existing paths
    pub fn scan_existing_paths<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root = root.as_ref();
        
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry.with_context(|| {
                format!("Failed to read directory entry while scanning for existing paths in {}", root.display())
            })?;
            
            self.add_existing_path(entry.path());
        }
        
        Ok(())
    }

    /// Detect all collisions
    pub fn detect_collisions(&mut self) -> Result<Vec<Collision>> {
        self.collisions.clear();

        // Check for multiple sources targeting the same destination
        for (target, sources) in &self.target_paths {
            if sources.len() > 1 {
                self.collisions.push(Collision {
                    collision_type: CollisionType::MultipleSourcesSameTarget,
                    target_path: target.clone(),
                    source_paths: sources.clone(),
                    description: format!(
                        "Multiple files/directories trying to rename to the same target: {} (sources: {})",
                        target.display(),
                        sources.iter().map(|s| s.display().to_string()).collect::<Vec<_>>().join(", ")
                    ),
                });
            }
        }

        // Check for targets that already exist
        for (target, sources) in &self.target_paths {
            // Skip if source equals target (no-op)
            if sources.len() == 1 && sources[0] == *target {
                self.collisions.push(Collision {
                    collision_type: CollisionType::SourceEqualsTarget,
                    target_path: target.clone(),
                    source_paths: sources.clone(),
                    description: format!("Source and target are identical: {}", target.display()),
                });
                continue;
            }

            if self.existing_paths.contains(target) {
                // Additional check: is this a file trying to overwrite a directory or vice versa?
                let target_is_dir = target.is_dir();
                let source_is_dir = sources.first().map(|s| s.is_dir()).unwrap_or(false);
                
                let collision_type = if target_is_dir && !source_is_dir {
                    CollisionType::FileToDirectory
                } else if !target_is_dir && source_is_dir {
                    CollisionType::DirectoryToFile
                } else {
                    CollisionType::TargetAlreadyExists
                };

                self.collisions.push(Collision {
                    collision_type,
                    target_path: target.clone(),
                    source_paths: sources.clone(),
                    description: format!("Target path already exists: {}", target.display()),
                });
            }
        }

        // Check for case-only differences on case-insensitive filesystems
        if self.is_case_insensitive_filesystem()? {
            self.detect_case_collisions()?;
        }

        Ok(self.collisions.clone())
    }

    /// Check if we're on a case-insensitive filesystem
    fn is_case_insensitive_filesystem(&self) -> Result<bool> {
        // Simple heuristic: check if we're on macOS or Windows
        Ok(cfg!(target_os = "macos") || cfg!(target_os = "windows"))
    }

    /// Detect case-only collisions on case-insensitive filesystems
    fn detect_case_collisions(&mut self) -> Result<()> {
        let mut lowercase_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        // Group paths by their lowercase versions
        for target in self.target_paths.keys() {
            if let Some(target_str) = target.to_str() {
                let lowercase = target_str.to_lowercase();
                lowercase_map.entry(lowercase).or_insert_with(Vec::new).push(target.clone());
            }
        }

        // Check for case conflicts
        for (_, paths) in lowercase_map {
            if paths.len() > 1 {
                for path in &paths {
                    if let Some(sources) = self.target_paths.get(path) {
                        self.collisions.push(Collision {
                            collision_type: CollisionType::CaseOnlyDifference,
                            target_path: path.clone(),
                            source_paths: sources.clone(),
                            description: format!(
                                "Case-only difference detected on case-insensitive filesystem: {}",
                                path.display()
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all detected collisions
    pub fn get_collisions(&self) -> &[Collision] {
        &self.collisions
    }

    /// Check if any collisions were detected
    pub fn has_collisions(&self) -> bool {
        !self.collisions.is_empty()
    }

    /// Get the number of collisions
    pub fn collision_count(&self) -> usize {
        self.collisions.len()
    }

    /// Filter collisions by type
    pub fn get_collisions_by_type(&self, collision_type: CollisionType) -> Vec<&Collision> {
        self.collisions
            .iter()
            .filter(|c| c.collision_type == collision_type)
            .collect()
    }

    /// Get a summary of collisions by type
    pub fn get_collision_summary(&self) -> HashMap<CollisionType, usize> {
        let mut summary = HashMap::new();
        for collision in &self.collisions {
            *summary.entry(collision.collision_type.clone()).or_insert(0) += 1;
        }
        summary
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.target_paths.clear();
        self.existing_paths.clear();
        self.collisions.clear();
    }

    /// Generate a detailed collision report
    pub fn generate_report(&self) -> String {
        if self.collisions.is_empty() {
            return "No collisions detected.".to_string();
        }

        let mut report = String::new();
        report.push_str(&format!("Collision Report ({} issues found):\n", self.collisions.len()));
        report.push_str("=" .repeat(50).as_str());
        report.push('\n');

        let summary = self.get_collision_summary();
        for (collision_type, count) in &summary {
            report.push_str(&format!("  {:?}: {} issue(s)\n", collision_type, count));
        }
        report.push('\n');

        for (i, collision) in self.collisions.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, collision.description));
            report.push_str(&format!("   Type: {:?}\n", collision.collision_type));
            report.push_str(&format!("   Target: {}\n", collision.target_path.display()));
            if collision.source_paths.len() == 1 {
                report.push_str(&format!("   Source: {}\n", collision.source_paths[0].display()));
            } else {
                report.push_str("   Sources:\n");
                for source in &collision.source_paths {
                    report.push_str(&format!("     - {}\n", source.display()));
                }
            }
            report.push('\n');
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};

    #[test]
    fn test_no_collisions() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        detector.add_rename(
            PathBuf::from("/test/old1.txt"),
            PathBuf::from("/test/new1.txt"),
        );
        detector.add_rename(
            PathBuf::from("/test/old2.txt"),
            PathBuf::from("/test/new2.txt"),
        );

        let collisions = detector.detect_collisions()?;
        assert!(collisions.is_empty());
        assert!(!detector.has_collisions());

        Ok(())
    }

    #[test]
    fn test_multiple_sources_same_target() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        detector.add_rename(
            PathBuf::from("/test/old1.txt"),
            PathBuf::from("/test/target.txt"),
        );
        detector.add_rename(
            PathBuf::from("/test/old2.txt"),
            PathBuf::from("/test/target.txt"),
        );

        let collisions = detector.detect_collisions()?;
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].collision_type, CollisionType::MultipleSourcesSameTarget);
        assert!(detector.has_collisions());

        Ok(())
    }

    #[test]
    fn test_target_already_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut detector = CollisionDetector::new();
        
        // Create an existing file
        let existing_file = temp_dir.path().join("existing.txt");
        File::create(&existing_file)?;
        
        detector.add_existing_path(&existing_file);
        detector.add_rename(
            temp_dir.path().join("source.txt"),
            existing_file.clone(),
        );

        let collisions = detector.detect_collisions()?;
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].collision_type, CollisionType::TargetAlreadyExists);

        Ok(())
    }

    #[test]
    fn test_source_equals_target() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        let path = PathBuf::from("/test/same.txt");
        detector.add_rename(path.clone(), path);

        let collisions = detector.detect_collisions()?;
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].collision_type, CollisionType::SourceEqualsTarget);

        Ok(())
    }

    #[test]
    fn test_file_to_directory_collision() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut detector = CollisionDetector::new();
        
        // Create an existing directory
        let existing_dir = temp_dir.path().join("existing_dir");
        fs::create_dir(&existing_dir)?;
        
        detector.add_existing_path(&existing_dir);
        detector.add_rename(
            temp_dir.path().join("source.txt"), // file
            existing_dir.clone(), // directory
        );

        let collisions = detector.detect_collisions()?;
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].collision_type, CollisionType::FileToDirectory);

        Ok(())
    }

    #[test]
    fn test_directory_to_file_collision() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut detector = CollisionDetector::new();
        
        // Create an existing file
        let existing_file = temp_dir.path().join("existing.txt");
        File::create(&existing_file)?;
        
        detector.add_existing_path(&existing_file);
        // Create the source directory first
        let source_dir = temp_dir.path().join("source_dir");
        fs::create_dir(&source_dir)?;
        
        detector.add_rename(
            source_dir, // directory
            existing_file.clone(), // file
        );

        let collisions = detector.detect_collisions()?;
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].collision_type, CollisionType::DirectoryToFile);

        Ok(())
    }

    #[test]
    fn test_scan_existing_paths() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut detector = CollisionDetector::new();
        
        // Create some files and directories
        File::create(temp_dir.path().join("file1.txt"))?;
        File::create(temp_dir.path().join("file2.txt"))?;
        fs::create_dir(temp_dir.path().join("subdir"))?;
        File::create(temp_dir.path().join("subdir").join("file3.txt"))?;

        detector.scan_existing_paths(temp_dir.path())?;
        
        // The detector should have found all the created paths
        assert!(detector.existing_paths.len() >= 4); // at least the files/dirs we created

        Ok(())
    }

    #[test]
    fn test_collision_summary() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        // Add multiple types of collisions
        detector.add_rename(
            PathBuf::from("/test/old1.txt"),
            PathBuf::from("/test/target.txt"),
        );
        detector.add_rename(
            PathBuf::from("/test/old2.txt"),
            PathBuf::from("/test/target.txt"),
        );
        
        let same_path = PathBuf::from("/test/same.txt");
        detector.add_rename(same_path.clone(), same_path);

        detector.detect_collisions()?;
        
        let summary = detector.get_collision_summary();
        assert_eq!(summary.get(&CollisionType::MultipleSourcesSameTarget), Some(&1));
        assert_eq!(summary.get(&CollisionType::SourceEqualsTarget), Some(&1));
        assert_eq!(summary.len(), 2);

        Ok(())
    }

    #[test]
    fn test_collision_report() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        detector.add_rename(
            PathBuf::from("/test/old1.txt"),
            PathBuf::from("/test/target.txt"),
        );
        detector.add_rename(
            PathBuf::from("/test/old2.txt"),
            PathBuf::from("/test/target.txt"),
        );

        detector.detect_collisions()?;
        
        let report = detector.generate_report();
        assert!(report.contains("Collision Report"));
        assert!(report.contains("MultipleSourcesSameTarget"));
        assert!(report.contains("target.txt"));

        Ok(())
    }

    #[test]
    fn test_no_collisions_report() {
        let detector = CollisionDetector::new();
        let report = detector.generate_report();
        assert_eq!(report, "No collisions detected.");
    }

    #[test]
    fn test_clear() -> Result<()> {
        let mut detector = CollisionDetector::new();
        
        detector.add_rename(
            PathBuf::from("/test/old.txt"),
            PathBuf::from("/test/new.txt"),
        );
        detector.add_existing_path("/test/existing.txt");
        detector.detect_collisions()?;

        assert!(!detector.target_paths.is_empty());
        assert!(!detector.existing_paths.is_empty());

        detector.clear();

        assert!(detector.target_paths.is_empty());
        assert!(detector.existing_paths.is_empty());
        assert!(detector.collisions.is_empty());

        Ok(())
    }
}