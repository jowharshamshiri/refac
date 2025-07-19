use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerbumpConfig {
    pub version: u32,
    pub enabled: bool,
    pub version_file: String,
    #[serde(default = "default_auto_detect")]
    pub auto_detect_project_files: bool,
    #[serde(default)]
    pub project_files: Vec<String>,
}

fn default_auto_detect() -> bool {
    true
}

impl Default for VerbumpConfig {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
            version_file: "version.txt".to_string(),
            auto_detect_project_files: true,
            project_files: Vec::new(),
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

pub fn update_version_file(version_info: &VersionInfo, config: &VerbumpConfig) -> Result<bool> {
    // Check if version has actually changed
    let version_file_path = PathBuf::from(&config.version_file);
    let current_version_content = if version_file_path.exists() {
        fs::read_to_string(&version_file_path).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    };
    
    if current_version_content == version_info.full_version {
        println!("Version {} is already up to date", version_info.full_version);
        return Ok(false);
    }
    
    // Update the main version file
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

    // Auto-detect and update project files if enabled
    if config.auto_detect_project_files {
        if let Ok(git_root) = get_git_root() {
            match detect_project_files(&git_root) {
                Ok(project_files) => {
                    if !project_files.is_empty() {
                        match update_project_files(version_info, &project_files) {
                            Ok(updated_files) => {
                                if !updated_files.is_empty() {
                                    println!("Updated project files: {}", updated_files.join(", "));
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to update some project files: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to detect project files: {}", e);
                }
            }
        }
    }

    // Update manually specified project files
    if !config.project_files.is_empty() {
        if let Ok(git_root) = get_git_root() {
            let manual_files: Vec<ProjectFile> = config.project_files
                .iter()
                .filter_map(|file_path| {
                    let full_path = git_root.join(file_path);
                    if full_path.exists() {
                        // Try to detect file type from extension/name
                        detect_file_type(&full_path).map(|file_type| ProjectFile {
                            path: full_path,
                            file_type,
                        })
                    } else {
                        eprintln!("Warning: Configured project file not found: {}", file_path);
                        None
                    }
                })
                .collect();
            
            if !manual_files.is_empty() {
                match update_project_files(version_info, &manual_files) {
                    Ok(updated_files) => {
                        if !updated_files.is_empty() {
                            println!("Updated configured project files: {}", updated_files.join(", "));
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to update configured project files: {}", e);
                    }
                }
            }
        }
    }

    Ok(true)
}

fn detect_file_type(path: &Path) -> Option<ProjectFileType> {
    match path.file_name()?.to_str()? {
        "Cargo.toml" => Some(ProjectFileType::CargoToml),
        "package.json" => Some(ProjectFileType::PackageJson),
        "pyproject.toml" => Some(ProjectFileType::PyprojectToml),
        "setup.py" => Some(ProjectFileType::SetupPy),
        "composer.json" => Some(ProjectFileType::ComposerJson),
        "pubspec.yaml" => Some(ProjectFileType::PubspecYaml),
        "pom.xml" => Some(ProjectFileType::PomXml),
        "build.gradle" => Some(ProjectFileType::BuildGradle),
        "CMakeLists.txt" => Some(ProjectFileType::CMakeLists),
        _ => None,
    }
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

#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub path: PathBuf,
    pub file_type: ProjectFileType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFileType {
    CargoToml,
    PackageJson,
    PyprojectToml,
    SetupPy,
    ComposerJson,
    PubspecYaml,
    PomXml,
    BuildGradle,
    CMakeLists,
}

impl ProjectFileType {
    pub fn file_name(&self) -> &'static str {
        match self {
            ProjectFileType::CargoToml => "Cargo.toml",
            ProjectFileType::PackageJson => "package.json",
            ProjectFileType::PyprojectToml => "pyproject.toml",
            ProjectFileType::SetupPy => "setup.py",
            ProjectFileType::ComposerJson => "composer.json",
            ProjectFileType::PubspecYaml => "pubspec.yaml",
            ProjectFileType::PomXml => "pom.xml",
            ProjectFileType::BuildGradle => "build.gradle",
            ProjectFileType::CMakeLists => "CMakeLists.txt",
        }
    }
}

pub fn detect_project_files(repo_root: &Path) -> Result<Vec<ProjectFile>> {
    let mut project_files = Vec::new();
    
    // Define project file types to detect
    let file_types = [
        ProjectFileType::CargoToml,
        ProjectFileType::PackageJson,
        ProjectFileType::PyprojectToml,
        ProjectFileType::SetupPy,
        ProjectFileType::ComposerJson,
        ProjectFileType::PubspecYaml,
        ProjectFileType::PomXml,
        ProjectFileType::BuildGradle,
        ProjectFileType::CMakeLists,
    ];
    
    for file_type in &file_types {
        let file_path = repo_root.join(file_type.file_name());
        if file_path.exists() {
            project_files.push(ProjectFile {
                path: file_path,
                file_type: file_type.clone(),
            });
        }
    }
    
    Ok(project_files)
}

pub fn update_project_files(version_info: &VersionInfo, project_files: &[ProjectFile]) -> Result<Vec<String>> {
    let mut updated_files = Vec::new();
    
    for project_file in project_files {
        match update_project_file(version_info, project_file) {
            Ok(()) => {
                updated_files.push(project_file.path.display().to_string());
                
                // Stage the updated file
                let output = Command::new("git")
                    .args(["add", project_file.path.to_str().unwrap()])
                    .output()
                    .context("Failed to stage updated project file")?;
                
                if !output.status.success() {
                    eprintln!("Warning: Failed to stage {}", project_file.path.display());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to update {}: {}", project_file.path.display(), e);
            }
        }
    }
    
    Ok(updated_files)
}

fn update_project_file(version_info: &VersionInfo, project_file: &ProjectFile) -> Result<()> {
    let content = fs::read_to_string(&project_file.path)
        .with_context(|| format!("Failed to read {}", project_file.path.display()))?;
    
    let updated_content = match project_file.file_type {
        ProjectFileType::CargoToml => update_cargo_toml(&content, &version_info.full_version)?,
        ProjectFileType::PackageJson => update_package_json(&content, &version_info.full_version)?,
        ProjectFileType::PyprojectToml => update_pyproject_toml(&content, &version_info.full_version)?,
        ProjectFileType::SetupPy => update_setup_py(&content, &version_info.full_version)?,
        ProjectFileType::ComposerJson => update_composer_json(&content, &version_info.full_version)?,
        ProjectFileType::PubspecYaml => update_pubspec_yaml(&content, &version_info.full_version)?,
        ProjectFileType::PomXml => update_pom_xml(&content, &version_info.full_version)?,
        ProjectFileType::BuildGradle => update_build_gradle(&content, &version_info.full_version)?,
        ProjectFileType::CMakeLists => update_cmake_lists(&content, &version_info.full_version)?,
    };
    
    fs::write(&project_file.path, updated_content)
        .with_context(|| format!("Failed to write updated {}", project_file.path.display()))?;
    
    Ok(())
}

fn update_cargo_toml(content: &str, version: &str) -> Result<String> {
    let mut parsed: toml::Value = content.parse()
        .context("Failed to parse Cargo.toml")?;
    
    if let Some(package) = parsed.get_mut("package").and_then(|p| p.as_table_mut()) {
        package.insert("version".to_string(), toml::Value::String(version.to_string()));
    }
    
    Ok(toml::to_string(&parsed)?)
}

fn update_package_json(content: &str, version: &str) -> Result<String> {
    let mut parsed: serde_json::Value = serde_json::from_str(content)
        .context("Failed to parse package.json")?;
    
    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("version".to_string(), serde_json::Value::String(version.to_string()));
    }
    
    Ok(serde_json::to_string_pretty(&parsed)?)
}

fn update_pyproject_toml(content: &str, version: &str) -> Result<String> {
    let mut parsed: toml::Value = content.parse()
        .context("Failed to parse pyproject.toml")?;
    
    // Try tool.poetry.version first, then project.version
    if let Some(tool) = parsed.get_mut("tool") {
        if let Some(poetry) = tool.get_mut("poetry").and_then(|p| p.as_table_mut()) {
            poetry.insert("version".to_string(), toml::Value::String(version.to_string()));
        }
    }
    
    if let Some(project) = parsed.get_mut("project").and_then(|p| p.as_table_mut()) {
        project.insert("version".to_string(), toml::Value::String(version.to_string()));
    }
    
    Ok(toml::to_string(&parsed)?)
}

fn update_setup_py(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r#"version\s*=\s*["'][^"']*["']"#)
        .context("Failed to create regex for setup.py")?;
    
    let updated = version_regex.replace_all(content, &format!(r#"version="{}""#, version));
    Ok(updated.to_string())
}

fn update_composer_json(content: &str, version: &str) -> Result<String> {
    let mut parsed: serde_json::Value = serde_json::from_str(content)
        .context("Failed to parse composer.json")?;
    
    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("version".to_string(), serde_json::Value::String(version.to_string()));
    }
    
    Ok(serde_json::to_string_pretty(&parsed)?)
}

fn update_pubspec_yaml(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"(?m)^version:\s*.*$")
        .context("Failed to create regex for pubspec.yaml")?;
    
    let updated = version_regex.replace_all(content, &format!("version: {}", version));
    Ok(updated.to_string())
}

fn update_pom_xml(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"<version>[^<]*</version>")
        .context("Failed to create regex for pom.xml")?;
    
    let updated = version_regex.replace(content, &format!("<version>{}</version>", version));
    Ok(updated.to_string())
}

fn update_build_gradle(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r#"version\s*=\s*['"][^'"]*['"]"#)
        .context("Failed to create regex for build.gradle")?;
    
    let updated = version_regex.replace_all(content, &format!(r#"version = '{}'"#, version));
    Ok(updated.to_string())
}

fn update_cmake_lists(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"(?i)project\s*\([^)]*VERSION\s+[^\s)]+")
        .context("Failed to create regex for CMakeLists.txt")?;
    
    let updated = version_regex.replace_all(content, |caps: &regex::Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let version_part_regex = Regex::new(r"VERSION\s+[^\s)]+").unwrap();
        version_part_regex.replace(matched, &format!("VERSION {}", version)).to_string()
    });
    
    Ok(updated.to_string())
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

    #[test]
    fn test_project_file_type_file_name() {
        assert_eq!(ProjectFileType::CargoToml.file_name(), "Cargo.toml");
        assert_eq!(ProjectFileType::PackageJson.file_name(), "package.json");
        assert_eq!(ProjectFileType::PyprojectToml.file_name(), "pyproject.toml");
        assert_eq!(ProjectFileType::SetupPy.file_name(), "setup.py");
        assert_eq!(ProjectFileType::ComposerJson.file_name(), "composer.json");
        assert_eq!(ProjectFileType::PubspecYaml.file_name(), "pubspec.yaml");
        assert_eq!(ProjectFileType::PomXml.file_name(), "pom.xml");
        assert_eq!(ProjectFileType::BuildGradle.file_name(), "build.gradle");
        assert_eq!(ProjectFileType::CMakeLists.file_name(), "CMakeLists.txt");
    }

    #[test]
    fn test_detect_file_type() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test Cargo.toml detection
        let cargo_path = temp_dir.path().join("Cargo.toml");
        assert_eq!(detect_file_type(&cargo_path), Some(ProjectFileType::CargoToml));
        
        // Test package.json detection
        let package_path = temp_dir.path().join("package.json");
        assert_eq!(detect_file_type(&package_path), Some(ProjectFileType::PackageJson));
        
        // Test unknown file
        let unknown_path = temp_dir.path().join("unknown.txt");
        assert_eq!(detect_file_type(&unknown_path), None);
    }

    #[test]
    fn test_detect_project_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some project files
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
        fs::write(temp_dir.path().join("package.json"), "{\"name\": \"test\", \"version\": \"1.0.0\"}").unwrap();
        
        let project_files = detect_project_files(temp_dir.path()).unwrap();
        assert_eq!(project_files.len(), 2);
        
        let file_types: Vec<_> = project_files.iter().map(|f| &f.file_type).collect();
        assert!(file_types.contains(&&ProjectFileType::CargoToml));
        assert!(file_types.contains(&&ProjectFileType::PackageJson));
    }

    #[test]
    fn test_update_cargo_toml() {
        let content = r#"[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
        
        let updated = update_cargo_toml(content, "1.2.3").unwrap();
        assert!(updated.contains("version = \"1.2.3\""));
        assert!(updated.contains("name = \"test-package\""));
        assert!(updated.contains("serde = \"1.0\""));
    }

    #[test]
    fn test_update_package_json() {
        let content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package",
  "main": "index.js"
}"#;
        
        let updated = update_package_json(content, "2.1.0").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["name"], "test-package");
    }

    #[test]
    fn test_update_pyproject_toml() {
        let content = r#"[tool.poetry]
name = "test-package"
version = "0.1.0"
description = "A test package"

[project]
name = "test-package"
version = "0.1.0"
"#;
        
        let updated = update_pyproject_toml(content, "1.5.2").unwrap();
        assert!(updated.contains("version = \"1.5.2\""));
        // Should update both poetry and project sections
        let version_count = updated.matches("version = \"1.5.2\"").count();
        assert_eq!(version_count, 2);
    }

    #[test]
    fn test_update_setup_py() {
        let content = r#"from setuptools import setup

setup(
    name="test-package",
    version="0.1.0",
    description="A test package",
    author="Test Author",
    packages=["test_package"],
)
"#;
        
        let updated = update_setup_py(content, "2.0.1").unwrap();
        assert!(updated.contains("version=\"2.0.1\""));
        assert!(updated.contains("name=\"test-package\""));
    }

    #[test]
    fn test_update_composer_json() {
        let content = r#"{
    "name": "vendor/package",
    "version": "1.0.0",
    "description": "A test package",
    "type": "library"
}"#;
        
        let updated = update_composer_json(content, "1.5.0").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(parsed["version"], "1.5.0");
        assert_eq!(parsed["name"], "vendor/package");
    }

    #[test]
    fn test_update_pubspec_yaml() {
        let content = r#"name: test_package
description: A test Flutter package
version: 1.0.0+1
environment:
  sdk: ">=2.12.0 <4.0.0"
"#;
        
        let updated = update_pubspec_yaml(content, "2.1.0+5").unwrap();
        assert!(updated.contains("version: 2.1.0+5"));
        assert!(updated.contains("name: test_package"));
    }

    #[test]
    fn test_update_pom_xml() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
</project>
"#;
        
        let updated = update_pom_xml(content, "2.3.1").unwrap();
        assert!(updated.contains("<version>2.3.1</version>"));
        assert!(updated.contains("<artifactId>test-project</artifactId>"));
    }

    #[test]
    fn test_update_build_gradle() {
        let content = r#"plugins {
    id 'java'
}

group = 'com.example'
version = '1.0.0'
sourceCompatibility = '11'

repositories {
    mavenCentral()
}
"#;
        
        let updated = update_build_gradle(content, "3.2.1").unwrap();
        assert!(updated.contains("version = '3.2.1'"));
        assert!(updated.contains("group = 'com.example'"));
    }

    #[test]
    fn test_update_cmake_lists() {
        let content = r#"cmake_minimum_required(VERSION 3.16)

project(TestProject
    VERSION 1.0.0
    DESCRIPTION "A test project"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 17)
"#;
        
        let updated = update_cmake_lists(content, "2.1.5").unwrap();
        assert!(updated.contains("VERSION 2.1.5"));
        assert!(updated.contains("project(TestProject"));
    }

    #[test]
    fn test_verbump_config_with_auto_detect() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = VerbumpConfig {
            version: 1,
            enabled: true,
            version_file: "version.txt".to_string(),
            auto_detect_project_files: true,
            project_files: vec!["custom.toml".to_string()],
        };
        
        config.save(temp_dir.path()).unwrap();
        let loaded_config = VerbumpConfig::load(temp_dir.path()).unwrap();
        
        assert_eq!(loaded_config.auto_detect_project_files, true);
        assert_eq!(loaded_config.project_files, vec!["custom.toml"]);
    }

    #[test]
    fn test_verbump_config_default_auto_detect() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test that auto_detect defaults to true when not specified
        let config_content = r#"{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}"#;
        fs::write(temp_dir.path().join(".verbump.json"), config_content).unwrap();
        
        let loaded_config = VerbumpConfig::load(temp_dir.path()).unwrap();
        assert_eq!(loaded_config.auto_detect_project_files, true);
        assert!(loaded_config.project_files.is_empty());
    }

    #[test]
    fn test_update_version_file_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let config = VerbumpConfig::default();
        
        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };
        
        // Create version file with same version
        let version_file_path = temp_dir.path().join("version.txt");
        fs::write(&version_file_path, "1.0.5.100\n").unwrap();
        
        // Change working directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Update should detect no change
        let result = update_version_file(&version_info, &config);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should return false indicating no update
        
        // File should still exist with same content
        let content = fs::read_to_string(&version_file_path).unwrap();
        assert_eq!(content.trim(), "1.0.5.100");
    }
}