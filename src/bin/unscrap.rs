use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use refac::scrap::{ScrapEntry, ScrapMetadata};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser, Debug)]
#[command(name = "unscrap")]
#[command(about = "Restore files from .scrap folder to their original locations")]
#[command(version = "0.1.0")]
struct Args {
    /// Name of file/directory in .scrap to restore
    name: Option<String>,
    
    /// Force restore even if destination exists
    #[arg(short, long)]
    force: bool,
    
    /// Restore to a different location
    #[arg(short = 't', long)]
    to: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {:#}", "Error".red(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let scrap_dir = current_dir.join(".scrap");
    
    if !scrap_dir.exists() {
        anyhow::bail!("No .scrap folder found in current directory");
    }
    
    match args.name {
        Some(name) => {
            // Restore specific file/directory
            restore_item(&scrap_dir, &name, args.to, args.force)?;
        }
        None => {
            // Undo last scrap operation
            undo_last_scrap(&scrap_dir)?;
        }
    }
    
    Ok(())
}

fn restore_item(scrap_dir: &Path, name: &str, custom_dest: Option<PathBuf>, force: bool) -> Result<()> {
    let source = scrap_dir.join(name);
    
    if !source.exists() {
        anyhow::bail!("'{}' not found in .scrap folder", name);
    }
    
    // Load metadata
    let mut metadata = ScrapMetadata::load(scrap_dir)?;
    
    let dest_path = if let Some(custom) = custom_dest {
        // Use custom destination
        if custom.is_dir() {
            custom.join(name)
        } else {
            custom
        }
    } else if let Some(entry) = metadata.get_entry(name) {
        // Use original path from metadata
        entry.original_path.clone()
    } else {
        // No metadata, restore to current directory
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(name)
    };
    
    // Check if destination exists
    if dest_path.exists() && !force {
        anyhow::bail!(
            "Destination '{}' already exists. Use --force to overwrite.",
            dest_path.display()
        );
    }
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }
    
    // Move the file/directory back
    fs::rename(&source, &dest_path)
        .with_context(|| format!("Failed to restore '{}' to '{}'", source.display(), dest_path.display()))?;
    
    // Remove from metadata
    metadata.remove_entry(name);
    metadata.save(scrap_dir)?;
    
    println!("{} '{}' to '{}'", "Restored".green(), name, dest_path.display());
    
    Ok(())
}

fn undo_last_scrap(scrap_dir: &Path) -> Result<()> {
    let metadata = ScrapMetadata::load(scrap_dir)?;
    
    if metadata.entries.is_empty() {
        println!("{}", "No items to restore in .scrap folder".yellow());
        return Ok(());
    }
    
    // Find the most recently scrapped item
    let most_recent = metadata.entries.values()
        .max_by_key(|entry| entry.scrapped_at)
        .ok_or_else(|| anyhow::anyhow!("No items found in metadata"))?;
    
    let name = most_recent.scrapped_name.clone();
    let original_path = most_recent.original_path.clone();
    
    println!("{} last scrapped item: {} (from {})",
        "Restoring".bold(),
        name.cyan(),
        original_path.display().to_string().dimmed()
    );
    
    // Restore the item
    restore_item(scrap_dir, &name, None, false)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::Utc;
    
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let scrap_dir = temp_dir.path().join(".scrap");
        fs::create_dir(&scrap_dir).unwrap();
        (temp_dir, scrap_dir)
    }
    
    #[test]
    fn test_restore_item_with_metadata() {
        let (temp_dir, scrap_dir) = setup_test_env();
        
        // Create a scrapped file
        let scrapped_file = scrap_dir.join("test.txt");
        fs::write(&scrapped_file, "content").unwrap();
        
        // Create metadata
        let mut metadata = ScrapMetadata::new();
        metadata.add_entry("test.txt", temp_dir.path().join("original/test.txt"));
        metadata.save(&scrap_dir).unwrap();
        
        // Restore the file
        restore_item(&scrap_dir, "test.txt", None, false).unwrap();
        
        // Check file was restored to original location
        let restored = temp_dir.path().join("original/test.txt");
        assert!(restored.exists());
        assert_eq!(fs::read_to_string(&restored).unwrap(), "content");
        assert!(!scrapped_file.exists());
        
        // Check metadata was updated
        let updated_metadata = ScrapMetadata::load(&scrap_dir).unwrap();
        assert!(updated_metadata.entries.is_empty());
    }
    
    #[test]
    fn test_restore_item_without_metadata() {
        let (temp_dir, scrap_dir) = setup_test_env();
        
        // Create a scrapped file without metadata
        let scrapped_file = scrap_dir.join("test.txt");
        fs::write(&scrapped_file, "content").unwrap();
        
        // Change to temp directory and restore
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        
        // Restore should put it in current directory
        restore_item(&scrap_dir, "test.txt", None, false).unwrap();
        
        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
        
        // Check file was restored to temp directory (which was current dir during restore)
        let restored = temp_dir.path().join("test.txt");
        assert!(restored.exists());
        assert_eq!(fs::read_to_string(&restored).unwrap(), "content");
        assert!(!scrapped_file.exists());
    }
    
    #[test]
    fn test_restore_item_custom_destination() {
        let (temp_dir, scrap_dir) = setup_test_env();
        
        // Create a scrapped file
        let scrapped_file = scrap_dir.join("test.txt");
        fs::write(&scrapped_file, "content").unwrap();
        
        // Create custom destination directory
        let custom_dest = temp_dir.path().join("custom");
        fs::create_dir(&custom_dest).unwrap();
        
        // Restore to custom location
        restore_item(&scrap_dir, "test.txt", Some(custom_dest.clone()), false).unwrap();
        
        // Check file was restored to custom location
        let restored = custom_dest.join("test.txt");
        assert!(restored.exists());
        assert_eq!(fs::read_to_string(&restored).unwrap(), "content");
        assert!(!scrapped_file.exists());
    }
    
    #[test]
    fn test_restore_directory() {
        let (temp_dir, scrap_dir) = setup_test_env();
        
        // Create a scrapped directory
        let scrapped_dir = scrap_dir.join("testdir");
        fs::create_dir(&scrapped_dir).unwrap();
        fs::write(scrapped_dir.join("file.txt"), "content").unwrap();
        
        // Create metadata
        let mut metadata = ScrapMetadata::new();
        metadata.add_entry("testdir", temp_dir.path().join("original/testdir"));
        metadata.save(&scrap_dir).unwrap();
        
        // Restore the directory
        restore_item(&scrap_dir, "testdir", None, false).unwrap();
        
        // Check directory was restored
        let restored = temp_dir.path().join("original/testdir");
        assert!(restored.exists());
        assert!(restored.is_dir());
        assert!(restored.join("file.txt").exists());
        assert!(!scrapped_dir.exists());
    }
    
    #[test]
    fn test_restore_with_force() {
        let (temp_dir, scrap_dir) = setup_test_env();
        
        // Create existing file
        let existing = temp_dir.path().join("test.txt");
        fs::write(&existing, "existing").unwrap();
        
        // Create scrapped file
        let scrapped_file = scrap_dir.join("test.txt");
        fs::write(&scrapped_file, "new content").unwrap();
        
        // Without force should fail
        let result = restore_item(&scrap_dir, "test.txt", Some(temp_dir.path().to_path_buf()), false);
        assert!(result.is_err());
        
        // With force should succeed
        restore_item(&scrap_dir, "test.txt", Some(temp_dir.path().to_path_buf()), true).unwrap();
        
        // Check file was overwritten
        assert_eq!(fs::read_to_string(&existing).unwrap(), "new content");
        assert!(!scrapped_file.exists());
    }
    
    #[test]
    fn test_undo_last_scrap() {
        let (temp_dir, scrap_dir) = setup_test_env();
        env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create multiple scrapped files with metadata
        let mut metadata = ScrapMetadata::new();
        
        // Add first file (older)
        fs::write(scrap_dir.join("old.txt"), "old").unwrap();
        let entry1 = ScrapEntry {
            original_path: temp_dir.path().join("old.txt"),
            scrapped_at: Utc::now() - chrono::Duration::hours(1),
            scrapped_name: "old.txt".to_string(),
        };
        metadata.entries.insert("old.txt".to_string(), entry1);
        
        // Add second file (newer)
        fs::write(scrap_dir.join("new.txt"), "new").unwrap();
        let entry2 = ScrapEntry {
            original_path: temp_dir.path().join("new.txt"),
            scrapped_at: Utc::now(),
            scrapped_name: "new.txt".to_string(),
        };
        metadata.entries.insert("new.txt".to_string(), entry2);
        
        metadata.save(&scrap_dir).unwrap();
        
        // Undo should restore the newer file
        undo_last_scrap(&scrap_dir).unwrap();
        
        // Check newer file was restored
        assert!(temp_dir.path().join("new.txt").exists());
        assert!(!scrap_dir.join("new.txt").exists());
        
        // Check older file still in scrap
        assert!(scrap_dir.join("old.txt").exists());
        assert!(!temp_dir.path().join("old.txt").exists());
    }
    
    #[test]
    fn test_restore_nonexistent_item() {
        let (_temp_dir, scrap_dir) = setup_test_env();
        
        let result = restore_item(&scrap_dir, "nonexistent.txt", None, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}