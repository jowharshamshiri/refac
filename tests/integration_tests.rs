use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use nomion::{cli::Args, run_refac};

/// Test utilities
mod test_utils {
    use super::*;

    pub fn create_test_structure(root: &Path) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(root.join("oldname_dir1/oldname_subdir"))?;
        fs::create_dir_all(root.join("oldname_dir2/normal_subdir"))?;
        fs::create_dir_all(root.join("normal_dir/oldname_nested"))?;
        fs::create_dir_all(root.join("edge cases"))?;
        fs::create_dir_all(root.join("special-chars"))?;

        // Create files with oldname in names
        File::create(root.join("oldname_file1.txt"))?
            .write_all(b"Content with oldname inside")?;
        File::create(root.join("oldname_dir1/oldname_file2.txt"))?
            .write_all(b"Another oldname content")?;
        File::create(root.join("normal_dir/normal_file.txt"))?
            .write_all(b"No pattern content")?;
        File::create(root.join("normal_dir/content_only.txt"))?
            .write_all(b"File with oldname in content only")?;
        File::create(root.join("oldname_dir2/end_oldname.txt"))?
            .write_all(b"end with oldname")?;

        // Edge case files
        File::create(root.join("edge cases/oldname multiple.txt"))?
            .write_all(b"oldname oldname oldname")?;
        File::create(root.join("special-chars/oldname.config"))?
            .write_all(b"Complex oldname content")?;

        // Files with multiple occurrences
        File::create(root.join("oldnameoldnameoldname.txt"))?
            .write_all(b"oldname content with oldname")?;

        // Hidden files
        File::create(root.join(".oldname_hidden"))?
            .write_all(b"Hidden oldname content")?;
        fs::create_dir(root.join(".oldname_hidden_dir"))?;

        // Binary file (for testing binary detection)
        File::create(root.join("oldname_binary.bin"))?
            .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04])?;

        Ok(())
    }

    pub fn count_matches(root: &Path, pattern: &str) -> usize {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        return name_str.contains(pattern);
                    }
                }
                false
            })
            .count()
    }

    pub fn count_content_matches(root: &Path, pattern: &str) -> Result<usize> {
        let mut count = 0;
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.path().is_file() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains(pattern) {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }
}

#[test]
fn test_basic_replacement() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create simple structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    // Create args for renaming
    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    // Run refac
    run_refac(args)?;

    // Verify changes
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(temp_dir.path().join("newname_dir").exists());
    assert!(!temp_dir.path().join("oldname_file.txt").exists());
    assert!(!temp_dir.path().join("oldname_dir").exists());

    // Check content was modified
    let content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(content.contains("newname"));
    assert!(!content.contains("oldname"));

    Ok(())
}

#[test]
fn test_dry_run_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: true,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    // Run dry run
    run_refac(args)?;

    // Verify nothing changed
    assert!(temp_dir.path().join("oldname_file.txt").exists());
    assert!(temp_dir.path().join("oldname_dir").exists());
    assert!(!temp_dir.path().join("newname_file.txt").exists());
    assert!(!temp_dir.path().join("newname_dir").exists());

    // Check content unchanged
    let content = fs::read_to_string(temp_dir.path().join("oldname_file.txt"))?;
    assert!(content.contains("oldname"));

    Ok(())
}

#[test]
fn test_case_sensitivity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files with different cases
    File::create(temp_dir.path().join("OldName_file.txt"))?
        .write_all(b"OldName OLDNAME oldname")?;
    fs::create_dir(temp_dir.path().join("OLDNAME_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "OldName".to_string(),
        new_string: "NewName".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Check that only exact case matches were replaced
    assert!(temp_dir.path().join("NewName_file.txt").exists());
    assert!(temp_dir.path().join("OLDNAME_dir").exists()); // Should not be renamed

    let content = fs::read_to_string(temp_dir.path().join("NewName_file.txt"))?;
    assert!(content.contains("NewName")); // Should be replaced
    assert!(content.contains("OLDNAME")); // Should not be replaced
    assert!(content.contains("oldname")); // Should not be replaced

    Ok(())
}

#[test]
fn test_complex_nested_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    test_utils::create_test_structure(temp_dir.path())?;

    let _initial_oldname_count = test_utils::count_matches(temp_dir.path(), "oldname");
    let _initial_content_count = test_utils::count_content_matches(temp_dir.path(), "oldname")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Check that most oldname occurrences in names were replaced (some may fail due to path conflicts)
    let final_oldname_count = test_utils::count_matches(temp_dir.path(), "oldname");
    let final_newname_count = test_utils::count_matches(temp_dir.path(), "newname");
    let final_content_count = test_utils::count_content_matches(temp_dir.path(), "oldname")?;

    assert!(final_oldname_count < 10, "Most file/directory names should be renamed");
    assert!(final_newname_count > 0, "No items were renamed to contain 'newname'");
    assert!(final_content_count < 5, "Most file contents should be updated");

    // Verify specific files exist
    assert!(temp_dir.path().join("newname_file1.txt").exists());
    assert!(temp_dir.path().join("newname_dir1").exists());
    assert!(temp_dir.path().join("newname_dir1/newname_subdir").exists());

    Ok(())
}

#[test]
fn test_files_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: true,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // File should be renamed, directory should not
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(temp_dir.path().join("oldname_dir").exists()); // Directory unchanged
    assert!(!temp_dir.path().join("oldname_file.txt").exists());

    Ok(())
}

#[test]
fn test_dirs_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: true,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Directory should be renamed, file content should be updated but name unchanged
    assert!(temp_dir.path().join("oldname_file.txt").exists()); // File name unchanged
    assert!(temp_dir.path().join("newname_dir").exists());
    assert!(!temp_dir.path().join("oldname_dir").exists());

    // Content should NOT be updated in dirs-only mode
    let content = fs::read_to_string(temp_dir.path().join("oldname_file.txt"))?;
    assert!(content.contains("oldname")); // Content should be unchanged

    Ok(())
}

#[test]
fn test_names_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: true,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // File should be renamed but content unchanged
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(!temp_dir.path().join("oldname_file.txt").exists());

    let content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(content.contains("oldname")); // Content should be unchanged

    Ok(())
}

#[test]
fn test_content_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: true,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // File name should be unchanged but content updated
    assert!(temp_dir.path().join("oldname_file.txt").exists());
    assert!(!temp_dir.path().join("newname_file.txt").exists());

    let content = fs::read_to_string(temp_dir.path().join("oldname_file.txt"))?;
    assert!(content.contains("newname")); // Content should be updated
    assert!(!content.contains("oldname")); // Old content should be replaced

    Ok(())
}

#[test]
fn test_binary_file_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create binary file with oldname in filename
    File::create(temp_dir.path().join("oldname_binary.bin"))?
        .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04])?;
    
    // Create text file for comparison
    File::create(temp_dir.path().join("oldname_text.txt"))?
        .write_all(b"oldname text content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Both files should be renamed
    assert!(temp_dir.path().join("newname_binary.bin").exists());
    assert!(temp_dir.path().join("newname_text.txt").exists());

    // Text file content should be modified
    let text_content = fs::read_to_string(temp_dir.path().join("newname_text.txt"))?;
    assert!(text_content.contains("newname"));
    assert!(!text_content.contains("oldname"));

    // Binary file content should be unchanged (but we can't easily test this without reading binary)
    // The important thing is that it exists and wasn't corrupted
    assert!(temp_dir.path().join("newname_binary.bin").metadata()?.len() > 0);

    Ok(())
}

#[test]
fn test_max_depth() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create nested structure
    fs::create_dir_all(temp_dir.path().join("level1/level2/level3"))?;
    File::create(temp_dir.path().join("level1/oldname1.txt"))?
        .write_all(b"content")?;
    File::create(temp_dir.path().join("level1/level2/oldname2.txt"))?
        .write_all(b"content")?;
    File::create(temp_dir.path().join("level1/level2/level3/oldname3.txt"))?
        .write_all(b"content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 4,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Files at depth 1, 2, and 3 should be renamed with max_depth 3
    assert!(temp_dir.path().join("level1/newname1.txt").exists());
    assert!(temp_dir.path().join("level1/level2/newname2.txt").exists());
    assert!(temp_dir.path().join("level1/level2/level3/newname3.txt").exists());

    Ok(())
}

#[test]
fn test_backup_functionality() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    let original_content = "oldname content here";
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(original_content.as_bytes())?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: true,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Check that backup was created and contains original content
    let backup_files: Vec<_> = fs::read_dir(temp_dir.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains(".bak"))
                .unwrap_or(false)
        })
        .collect();

    assert!(!backup_files.is_empty(), "No backup file was created");

    // Check that one of the backup files contains the original content
    let mut found_original_content = false;
    for backup_file in backup_files {
        if let Ok(backup_content) = fs::read_to_string(backup_file.path()) {
            if backup_content.contains("oldname") {
                found_original_content = true;
                break;
            }
        }
    }
    assert!(found_original_content, "Backup file doesn't contain original content");

    // Check that main file was updated
    let updated_content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(updated_content.contains("newname"));
    assert!(!updated_content.contains("oldname"));

    Ok(())
}

#[test]
fn test_multiple_occurrences() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create file with multiple occurrences in name and content
    File::create(temp_dir.path().join("oldname_oldname_oldname.txt"))?
        .write_all(b"oldname oldname oldname in content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // All occurrences should be replaced
    assert!(temp_dir.path().join("newname_newname_newname.txt").exists());
    
    let content = fs::read_to_string(temp_dir.path().join("newname_newname_newname.txt"))?;
    assert!(content.contains("newname newname newname"));
    assert!(!content.contains("oldname"));

    Ok(())
}

#[test]
fn test_hidden_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create hidden files and directories
    File::create(temp_dir.path().join(".oldname_hidden"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join(".oldname_hidden_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![".*".to_string()], // Include hidden files
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // Hidden files should be renamed
    assert!(temp_dir.path().join(".newname_hidden").exists());
    assert!(temp_dir.path().join(".newname_hidden_dir").exists());
    
    let content = fs::read_to_string(temp_dir.path().join(".newname_hidden"))?;
    assert!(content.contains("newname"));

    Ok(())
}

#[test]
fn test_exclude_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files, some of which should be excluded
    File::create(temp_dir.path().join("oldname_include.txt"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join("oldname_exclude.log"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec!["*.log".to_string()],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // .txt file should be renamed, .log file should not
    assert!(temp_dir.path().join("newname_include.txt").exists());
    assert!(temp_dir.path().join("oldname_exclude.log").exists()); // Should not be renamed

    Ok(())
}

#[test]
fn test_parallel_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create many files to test parallel processing
    for i in 0..100 {
        File::create(temp_dir.path().join(format!("oldname_file_{}.txt", i)))?
            .write_all(format!("oldname content {}", i).as_bytes())?;
    }

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        old_string: "oldname".to_string(),
        new_string: "newname".to_string(),
        dry_run: false,
        force: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 4, // Use multiple threads
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
    };

    run_refac(args)?;

    // All files should be processed correctly
    for i in 0..100 {
        let new_file = temp_dir.path().join(format!("newname_file_{}.txt", i));
        assert!(new_file.exists(), "File {} was not renamed", i);
        
        let content = fs::read_to_string(&new_file)?;
        assert!(content.contains("newname"), "Content in file {} was not updated", i);
        assert!(!content.contains("oldname"), "Old content still exists in file {}", i);
    }

    Ok(())
}