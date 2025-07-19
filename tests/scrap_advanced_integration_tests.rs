use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_scrap_with_items(temp_dir: &Path) {
    let scrap_dir = temp_dir.join(".scrap");
    fs::create_dir(&scrap_dir).unwrap();
    
    // Create some test files
    fs::write(scrap_dir.join("file1.txt"), "content1").unwrap();
    fs::write(scrap_dir.join("file2.log"), "log content").unwrap();
    
    // Create a directory
    let test_dir = scrap_dir.join("testdir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("nested.txt"), "nested content").unwrap();
}

#[test]
fn test_scrap_list_default() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Test default list behavior (no arguments)
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contents of .scrap folder:"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.log"))
        .stdout(predicate::str::contains("testdir"));
}

#[test]
fn test_scrap_list_explicit() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Test explicit list command
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("list")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contents of .scrap folder:"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.log"))
        .stdout(predicate::str::contains("testdir"));
}

#[test]
fn test_scrap_list_sort_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("list")
        .arg("--sort")
        .arg("name")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contents of .scrap folder:"));
}

#[test]
fn test_scrap_list_sort_size() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("list")
        .arg("--sort")
        .arg("size")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contents of .scrap folder:"));
}

#[test]
fn test_scrap_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("The .scrap folder is empty"));
}

#[test]
fn test_scrap_clean_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("clean")
        .arg("--days")
        .arg("0")  // Clean everything
        .arg("--dry-run")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would remove items older than 0 days"))
        .stdout(predicate::str::contains("Would remove:"));
}

#[test]
fn test_scrap_clean_actual() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // First check items exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("clean")
        .arg("--days")
        .arg("0")  // Clean everything
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removing items older than 0 days"))
        .stdout(predicate::str::contains("Removed:"));
    
    // Check items were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
}

#[test]
fn test_scrap_purge_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Check items exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("purge")
        .arg("--force")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Purged:"));
    
    // Check all items were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.log").exists());
    assert!(!temp_path.join(".scrap").join("testdir").exists());
}

#[test]
fn test_scrap_purge_empty_folder() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("purge")
        .arg("--force")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("The .scrap folder is already empty"));
}

#[test]
fn test_scrap_find_filename() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("find")
        .arg("file.*txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Searching for 'file.*txt'"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("filename match"));
}

#[test]
fn test_scrap_find_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("find")
        .arg("nonexistent")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_scrap_find_content() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("find")
        .arg("content1")
        .arg("--content")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("content match"));
}

#[test]
fn test_scrap_archive() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    let archive_name = "test-archive.tar.gz";
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("archive")
        .arg("--output")
        .arg(archive_name)
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Archiving .scrap folder"))
        .stdout(predicate::str::contains("Archived"));
    
    // Check archive was created
    assert!(temp_path.join(archive_name).exists());
    
    // Check original files still exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
}

#[test]
fn test_scrap_archive_with_remove() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    let archive_name = "test-archive-remove.tar.gz";
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("archive")
        .arg("--output")
        .arg(archive_name)
        .arg("--remove")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived"))
        .stdout(predicate::str::contains("Removed archived files"));
    
    // Check archive was created
    assert!(temp_path.join(archive_name).exists());
    
    // Check original files were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.log").exists());
    assert!(!temp_path.join(".scrap").join("testdir").exists());
}

#[test]
fn test_scrap_archive_empty_folder() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("archive")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("The .scrap folder is empty"));
}

#[test]
fn test_scrap_archive_default_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("archive")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Archiving .scrap folder to .scrap-"));
}

#[test]
fn test_scrap_with_metadata_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    // Scrap the file
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check metadata was created
    let metadata_file = temp_path.join(".scrap").join(".metadata.json");
    assert!(metadata_file.exists());
    
    // Check metadata content
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(metadata_content.contains("test.txt"));
    assert!(metadata_content.contains("original_path"));
    assert!(metadata_content.contains("scrapped_at"));
}

#[test]
fn test_scrap_list_with_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // List should show metadata info
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"))
        .stdout(predicate::str::contains("from:"));
}

#[test]
fn test_unscrap_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Verify file was moved
    assert!(!test_file.exists());
    assert!(temp_path.join(".scrap").join("test.txt").exists());
    
    // Restore the file
    Command::cargo_bin("unscrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored 'test.txt'"));
    
    // Verify file was restored
    assert!(test_file.exists());
    assert!(!temp_path.join(".scrap").join("test.txt").exists());
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "test content");
}

#[test]
fn test_unscrap_undo_last() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap two files
    let test_file1 = temp_path.join("file1.txt");
    let test_file2 = temp_path.join("file2.txt");
    fs::write(&test_file1, "content1").unwrap();
    fs::write(&test_file2, "content2").unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("file1.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Wait a bit to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("file2.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Undo last (should restore file2.txt)
    Command::cargo_bin("unscrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Restoring last scrapped item: file2.txt"));
    
    // Verify correct file was restored
    assert!(test_file2.exists());
    assert!(!test_file1.exists());
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.txt").exists());
}

#[test]
fn test_unscrap_custom_destination() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Create custom destination
    let custom_dir = temp_path.join("custom");
    fs::create_dir(&custom_dir).unwrap();
    
    // Restore to custom location
    Command::cargo_bin("unscrap")
        .unwrap()
        .arg("test.txt")
        .arg("--to")
        .arg(&custom_dir)
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Verify file was restored to custom location
    assert!(custom_dir.join("test.txt").exists());
    assert!(!test_file.exists());
    assert!(!temp_path.join(".scrap").join("test.txt").exists());
}