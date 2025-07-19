use clap::Parser;
use colored::*;
use refac::{cli::Args, run_refac};
use std::process;

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Run the refac operation
    match run_refac(args) {
        Ok(()) => {
            // Success - exit with code 0
            process::exit(0);
        }
        Err(error) => {
            // Print error and exit with non-zero code
            eprintln!("{} {}", "ERROR:".red().bold(), error);
            
            // Print error chain if available
            let mut source = error.source();
            while let Some(err) = source {
                eprintln!("  Caused by: {}", err);
                source = err.source();
            }
            
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_version() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
        Ok(())
    }

    #[test]
    fn test_help() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A robust cross-platform tool"));
        Ok(())
    }

    #[test]
    fn test_missing_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("required"));
        Ok(())
    }

    #[test]
    fn test_invalid_directory() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&["/nonexistent/directory", "old", "new"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("does not exist"));
        Ok(())
    }

    #[test]
    fn test_empty_strings() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Test empty old string
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[temp_dir.path().to_str().unwrap(), "", "new"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("cannot be empty"));

        // Test empty new string
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[temp_dir.path().to_str().unwrap(), "old", ""]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("cannot be empty"));

        Ok(())
    }

    #[test]
    fn test_path_separator_in_new_string() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[temp_dir.path().to_str().unwrap(), "old", "new/path"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("path separator"));

        Ok(())
    }

    #[test]
    fn test_conflicting_mode_flags() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[
            temp_dir.path().to_str().unwrap(),
            "old",
            "new",
            "--files-only",
            "--dirs-only"
        ]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Cannot specify more than one mode"));

        Ok(())
    }

    #[test]
    fn test_dry_run_flag() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create a test file
        std::fs::write(temp_dir.path().join("oldname.txt"), "content")?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[
            temp_dir.path().to_str().unwrap(),
            "oldname",
            "newname",
            "--dry-run",
            "--force"
        ]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("DRY RUN"));

        // File should still exist with original name
        assert!(temp_dir.path().join("oldname.txt").exists());
        assert!(!temp_dir.path().join("newname.txt").exists());

        Ok(())
    }

    #[test]
    fn test_json_output() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create a test file
        std::fs::write(temp_dir.path().join("oldname.txt"), "content")?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[
            temp_dir.path().to_str().unwrap(),
            "oldname",
            "newname",
            "--dry-run",
            "--format", "json"
        ]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("summary"));

        Ok(())
    }

    #[test]
    fn test_verbose_flag() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create a test file
        std::fs::write(temp_dir.path().join("oldname.txt"), "content")?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[
            temp_dir.path().to_str().unwrap(),
            "oldname",
            "newname",
            "--dry-run",
            "--force",
            "--verbose"
        ]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("REFAC TOOL"));

        Ok(())
    }

    #[test]
    fn test_max_depth() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create nested structure
        std::fs::create_dir_all(temp_dir.path().join("level1/level2/level3"))?;
        std::fs::write(temp_dir.path().join("level1/level2/level3/oldname.txt"), "content")?;
        
        let mut cmd = Command::cargo_bin("refac")?;
        cmd.args(&[
            temp_dir.path().to_str().unwrap(),
            "oldname",
            "newname",
            "--dry-run",
            "--force",
            "--max-depth", "2"
        ]);
        cmd.assert()
            .success();

        Ok(())
    }
}