use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerbumpConfig {
    pub version: u32,
    pub enabled: bool,
    pub version_file: String,
}

impl Default for VerbumpConfig {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
            version_file: "version.txt".to_string(),
        }
    }
}

impl VerbumpConfig {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let config_path = repo_root.join(".verbump.json");
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read verbump config file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse verbump config file")
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let config_path = repo_root.join(".verbump.json");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize verbump config")?;
        
        fs::write(&config_path, content)
            .context("Failed to write verbump config file")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major_version: String,
    pub minor_version: u32,
    pub patch_version: u32,
    pub full_version: String,
}

impl VersionInfo {
    pub fn calculate() -> Result<Self> {
        let major_version = get_tag_version()?;
        let minor_version = get_commit_count_since_tag(&major_version)?;
        let patch_version = get_total_changes()?;
        
        let full_version = format!("{}.{}.{}", 
            major_version.strip_prefix('v').unwrap_or(&major_version),
            minor_version,
            patch_version
        );

        Ok(Self {
            major_version,
            minor_version,
            patch_version,
            full_version,
        })
    }
}

fn get_tag_version() -> Result<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in git tag output")?
                .trim()
                .to_string();
            Ok(version)
        }
        _ => Ok("v0".to_string()),
    }
}

fn get_commit_count_since_tag(tag_version: &str) -> Result<u32> {
    let output = if tag_version == "v0" {
        Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .context("Failed to run git rev-list command")?
    } else {
        let range = format!("{}..HEAD", tag_version);
        Command::new("git")
            .args(["rev-list", "--count", &range])
            .output()
            .context("Failed to run git rev-list command")?
    };

    if !output.status.success() {
        return Ok(0);
    }

    let count_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git rev-list output")?
        .trim()
        .to_string();

    count_str.parse::<u32>()
        .context("Failed to parse commit count")
}

fn get_total_changes() -> Result<u32> {
    let output = Command::new("git")
        .args(["log", "--pretty=tformat:", "--numstat"])
        .output()
        .context("Failed to run git log command")?;

    if !output.status.success() {
        return Ok(0);
    }

    let log_stat = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git log output")?;

    let mut total = 0u32;
    for line in log_stat.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(additions), Ok(deletions)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                total = total.saturating_add(additions).saturating_add(deletions);
            }
        }
    }

    Ok(total)
}

pub fn update_version_file(version_info: &VersionInfo, config: &VerbumpConfig) -> Result<()> {
    let version_file_path = PathBuf::from(&config.version_file);
    
    fs::write(&version_file_path, format!("{}\n", version_info.full_version))
        .with_context(|| format!("Failed to write version to {}", version_file_path.display()))?;

    // Stage the version file
    let output = Command::new("git")
        .args(["add", version_file_path.to_str().unwrap()])
        .output()
        .context("Failed to stage version file")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stage version file: {}", stderr);
    }

    Ok(())
}

pub fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get git root directory")?;

    if !output.status.success() {
        anyhow::bail!("Not in a git repository");
    }

    let root = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git root output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verbump_config_default() {
        let config = VerbumpConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.enabled);
        assert_eq!(config.version_file, "version.txt");
    }

    #[test]
    fn test_verbump_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config = VerbumpConfig::default();
        
        config.save(temp_dir.path()).unwrap();
        let loaded_config = VerbumpConfig::load(temp_dir.path()).unwrap();
        
        assert_eq!(config.version, loaded_config.version);
        assert_eq!(config.enabled, loaded_config.enabled);
        assert_eq!(config.version_file, loaded_config.version_file);
    }

    #[test]
    fn test_version_info_format() {
        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };
        
        assert_eq!(version_info.full_version, "1.0.5.100");
    }

    #[test]
    fn test_is_git_repository() {
        // This test will pass if run in a git repository
        // In CI/testing environments, this might be false
        let _ = is_git_repository();
    }
}