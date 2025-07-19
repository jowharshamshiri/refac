use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_scrap_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Run scrap with no arguments
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check .scrap directory was created
    assert!(temp_path.join(".scrap").exists());
    assert!(temp_path.join(".scrap").is_dir());
}

#[test]
fn test_scrap_updates_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a .gitignore file
    let gitignore_path = temp_path.join(".gitignore");
    fs::write(&gitignore_path, "*.log\ntarget/\n").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check .gitignore was updated
    let contents = fs::read_to_string(&gitignore_path).unwrap();
    assert!(contents.contains(".scrap/"));
}

#[test]
fn test_scrap_does_not_duplicate_gitignore_entry() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a .gitignore file with .scrap/ already in it
    let gitignore_path = temp_path.join(".gitignore");
    fs::write(&gitignore_path, "*.log\n.scrap/\ntarget/\n").unwrap();
    let original_contents = fs::read_to_string(&gitignore_path).unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check .gitignore was not changed
    let new_contents = fs::read_to_string(&gitignore_path).unwrap();
    assert_eq!(original_contents, new_contents);
}

#[test]
fn test_scrap_move_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    // Run scrap with file argument
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved 'test.txt' to"));
    
    // Check file was moved
    assert!(!test_file.exists());
    assert!(temp_path.join(".scrap").join("test.txt").exists());
    
    // Check content is preserved
    let content = fs::read_to_string(temp_path.join(".scrap").join("test.txt")).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_scrap_move_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test directory with content
    let test_dir = temp_path.join("testdir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content2").unwrap();
    
    // Run scrap with directory argument
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("testdir")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved 'testdir' to"));
    
    // Check directory was moved
    assert!(!test_dir.exists());
    let moved_dir = temp_path.join(".scrap").join("testdir");
    assert!(moved_dir.exists());
    assert!(moved_dir.is_dir());
    
    // Check contents are preserved
    assert!(moved_dir.join("file1.txt").exists());
    assert!(moved_dir.join("file2.txt").exists());
    assert_eq!(fs::read_to_string(moved_dir.join("file1.txt")).unwrap(), "content1");
    assert_eq!(fs::read_to_string(moved_dir.join("file2.txt")).unwrap(), "content2");
}

#[test]
fn test_scrap_move_with_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    // Run scrap with absolute path
    Command::cargo_bin("scrap")
        .unwrap()
        .arg(test_file.to_str().unwrap())
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check file was moved
    assert!(!test_file.exists());
    assert!(temp_path.join(".scrap").join("test.txt").exists());
}

#[test]
fn test_scrap_handles_name_conflicts() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create .scrap directory and a file in it
    let scrap_dir = temp_path.join(".scrap");
    fs::create_dir(&scrap_dir).unwrap();
    fs::write(scrap_dir.join("test.txt"), "original").unwrap();
    
    // Create a new file with same name
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "new content").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check both files exist with different names
    assert!(scrap_dir.join("test.txt").exists());
    assert!(scrap_dir.join("test_1.txt").exists());
    
    // Check contents
    assert_eq!(fs::read_to_string(scrap_dir.join("test.txt")).unwrap(), "original");
    assert_eq!(fs::read_to_string(scrap_dir.join("test_1.txt")).unwrap(), "new content");
}

#[test]
fn test_scrap_multiple_name_conflicts() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create .scrap directory with existing files
    let scrap_dir = temp_path.join(".scrap");
    fs::create_dir(&scrap_dir).unwrap();
    fs::write(scrap_dir.join("test.txt"), "original").unwrap();
    fs::write(scrap_dir.join("test_1.txt"), "first conflict").unwrap();
    fs::write(scrap_dir.join("test_2.txt"), "second conflict").unwrap();
    
    // Create a new file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "new content").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("test.txt")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check file was renamed to test_3.txt
    assert!(scrap_dir.join("test_3.txt").exists());
    assert_eq!(fs::read_to_string(scrap_dir.join("test_3.txt")).unwrap(), "new content");
}

#[test]
fn test_scrap_error_on_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Run scrap with nonexistent file
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("nonexistent.txt")
        .current_dir(temp_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_scrap_no_args_prints_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Run scrap with no arguments
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(".scrap"));
}

#[test]
fn test_scrap_handles_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a hidden file
    let hidden_file = temp_path.join(".hidden");
    fs::write(&hidden_file, "hidden content").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg(".hidden")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check file was moved
    assert!(!hidden_file.exists());
    assert!(temp_path.join(".scrap").join(".hidden").exists());
}

#[test]
fn test_scrap_handles_files_with_no_extension() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create files without extensions
    let scrap_dir = temp_path.join(".scrap");
    fs::create_dir(&scrap_dir).unwrap();
    fs::write(scrap_dir.join("README"), "original").unwrap();
    
    let test_file = temp_path.join("README");
    fs::write(&test_file, "new content").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("README")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check file was renamed properly
    assert!(scrap_dir.join("README_1").exists());
    assert_eq!(fs::read_to_string(scrap_dir.join("README_1")).unwrap(), "new content");
}

#[test]
fn test_scrap_preserves_permissions() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a file with specific permissions
        let test_file = temp_path.join("test.sh");
        fs::write(&test_file, "#!/bin/bash\necho test").unwrap();
        
        // Set executable permissions
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_file, perms).unwrap();
        
        // Run scrap
        Command::cargo_bin("scrap")
            .unwrap()
            .arg("test.sh")
            .current_dir(temp_path)
            .assert()
            .success();
        
        // Check permissions were preserved
        let moved_file = temp_path.join(".scrap").join("test.sh");
        let moved_perms = fs::metadata(&moved_file).unwrap().permissions();
        assert_eq!(moved_perms.mode() & 0o777, 0o755);
    }
}

#[test]
fn test_scrap_handles_symlinks() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a file and a symlink to it
        let target_file = temp_path.join("target.txt");
        fs::write(&target_file, "target content").unwrap();
        
        let symlink_path = temp_path.join("link.txt");
        symlink(&target_file, &symlink_path).unwrap();
        
        // Run scrap on the symlink
        Command::cargo_bin("scrap")
            .unwrap()
            .arg("link.txt")
            .current_dir(temp_path)
            .assert()
            .success();
        
        // Check symlink was moved (not the target)
        assert!(!symlink_path.exists());
        assert!(target_file.exists()); // Original target should still exist
        
        let moved_link = temp_path.join(".scrap").join("link.txt");
        assert!(moved_link.exists());
        assert!(moved_link.symlink_metadata().unwrap().file_type().is_symlink());
    }
}

#[test]
fn test_scrap_handles_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create an empty directory
    let empty_dir = temp_path.join("emptydir");
    fs::create_dir(&empty_dir).unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("emptydir")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check directory was moved
    assert!(!empty_dir.exists());
    let moved_dir = temp_path.join(".scrap").join("emptydir");
    assert!(moved_dir.exists());
    assert!(moved_dir.is_dir());
    assert!(moved_dir.read_dir().unwrap().next().is_none()); // Still empty
}

#[test]
fn test_scrap_handles_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create nested directory structure
    let nested = temp_path.join("parent");
    fs::create_dir(&nested).unwrap();
    fs::create_dir(nested.join("child")).unwrap();
    fs::create_dir(nested.join("child").join("grandchild")).unwrap();
    fs::write(nested.join("file1.txt"), "content1").unwrap();
    fs::write(nested.join("child").join("file2.txt"), "content2").unwrap();
    fs::write(nested.join("child").join("grandchild").join("file3.txt"), "content3").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .arg("parent")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check entire structure was moved
    assert!(!nested.exists());
    let moved = temp_path.join(".scrap").join("parent");
    assert!(moved.exists());
    assert!(moved.join("child").exists());
    assert!(moved.join("child").join("grandchild").exists());
    assert_eq!(fs::read_to_string(moved.join("file1.txt")).unwrap(), "content1");
    assert_eq!(fs::read_to_string(moved.join("child").join("file2.txt")).unwrap(), "content2");
    assert_eq!(fs::read_to_string(moved.join("child").join("grandchild").join("file3.txt")).unwrap(), "content3");
}

#[test]
fn test_scrap_handles_gitignore_without_newline() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a .gitignore file without trailing newline
    let gitignore_path = temp_path.join(".gitignore");
    fs::write(&gitignore_path, "*.log").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check .gitignore was updated correctly
    let contents = fs::read_to_string(&gitignore_path).unwrap();
    assert_eq!(contents, "*.log\n.scrap/\n");
}

#[test]
fn test_scrap_handles_empty_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create an empty .gitignore file
    let gitignore_path = temp_path.join(".gitignore");
    fs::write(&gitignore_path, "").unwrap();
    
    // Run scrap
    Command::cargo_bin("scrap")
        .unwrap()
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check .gitignore was updated correctly
    let contents = fs::read_to_string(&gitignore_path).unwrap();
    assert_eq!(contents, ".scrap/\n");
}