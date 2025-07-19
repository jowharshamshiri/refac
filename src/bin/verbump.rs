use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use refac::verbump::{VerbumpConfig, VersionInfo, detect_project_files, update_version_file};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "verbump")]
#[command(about = "Automatic version bumping based on git commits and changes")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install verbump as a pre-commit hook in the current git repository
    Install {
        /// Force reinstallation even if already installed
        #[arg(short, long)]
        force: bool,
    },
    
    /// Remove verbump from pre-commit hooks
    #[command(alias = "unhook")]
    Uninstall,
    
    /// Show current version information without updating
    Show,
    
    /// Update version manually (normally done automatically via git hook)
    Update {
        /// Force update even if not in a git repository
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show verbump status and configuration
    Status,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {:#}", "Error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Install { force }) => install_hook(force)?,
        Some(Commands::Uninstall) => uninstall_hook()?,
        Some(Commands::Show) => show_version()?,
        Some(Commands::Update { force }) => update_version(force)?,
        Some(Commands::Status) => show_status()?,
        None => {
            // Default behavior: install hook if not installed, otherwise update version
            if is_hook_installed()? {
                update_version(false)?;
            } else {
                install_hook(false)?;
            }
        }
    }
    
    Ok(())
}

fn install_hook(force: bool) -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run verbump from within a git repository.");
    }
    
    let git_root = get_git_root()?;
    let hooks_dir = git_root.join(".git").join("hooks");
    let hook_file = hooks_dir.join("pre-commit");
    
    // Create hooks directory if it doesn't exist
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .context("Failed to create git hooks directory")?;
        log_action(&format!("Created git hooks directory: {}", hooks_dir.display()));
    }
    
    // Check if already installed
    if !force && is_hook_installed()? {
        println!("{} verbump is already installed as a pre-commit hook", "Info".blue());
        println!("{} Use 'verbump install --force' to reinstall", "Tip".yellow());
        return Ok(());
    }
    
    // Get current binary path
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;
    
    let verbump_block = format!(
        "#!/bin/bash\n# === VERBUMP BLOCK START ===\n# DO NOT EDIT THIS BLOCK MANUALLY\n# Use 'verbump uninstall' to remove this hook\n{} update --force\n# === VERBUMP BLOCK END ===\n",
        current_exe.display()
    );
    
    if hook_file.exists() {
        // Read existing hook content
        let existing_content = fs::read_to_string(&hook_file)
            .context("Failed to read existing pre-commit hook")?;
        
        // Remove any existing verbump block
        let cleaned_content = remove_verbump_block(&existing_content);
        
        // Append new verbump block
        let new_content = if cleaned_content.trim().is_empty() {
            verbump_block
        } else {
            format!("{}\n{}", cleaned_content.trim_end(), verbump_block)
        };
        
        fs::write(&hook_file, new_content)
            .context("Failed to update pre-commit hook")?;
        
        log_action(&format!("Updated existing pre-commit hook: {}", hook_file.display()));
    } else {
        // Create new hook file
        fs::write(&hook_file, &verbump_block)
            .context("Failed to create pre-commit hook")?;
        
        log_action(&format!("Created new pre-commit hook: {}", hook_file.display()));
    }
    
    // Make hook executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_file, perms)?;
    }
    
    println!("{} verbump installed successfully as a pre-commit hook", "Success".green());
    println!("{} Version will be automatically updated on each commit", "Info".blue());
    
    Ok(())
}

fn uninstall_hook() -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run verbump from within a git repository.");
    }
    
    let git_root = get_git_root()?;
    let hook_file = git_root.join(".git").join("hooks").join("pre-commit");
    
    if !hook_file.exists() {
        println!("{} No pre-commit hook found", "Info".yellow());
        return Ok(());
    }
    
    let existing_content = fs::read_to_string(&hook_file)
        .context("Failed to read pre-commit hook")?;
    
    if !existing_content.contains("=== VERBUMP BLOCK START ===") {
        println!("{} verbump is not installed in the pre-commit hook", "Info".yellow());
        return Ok(());
    }
    
    let cleaned_content = remove_verbump_block(&existing_content);
    
    if cleaned_content.trim().is_empty() || cleaned_content.trim() == "#!/bin/bash" {
        // Remove the entire hook file if only verbump was in it or only has shebang
        fs::remove_file(&hook_file)
            .context("Failed to remove pre-commit hook file")?;
        log_action(&format!("Removed empty pre-commit hook file: {}", hook_file.display()));
    } else {
        // Update hook file with verbump block removed
        fs::write(&hook_file, cleaned_content)
            .context("Failed to update pre-commit hook")?;
        log_action(&format!("Removed verbump block from pre-commit hook: {}", hook_file.display()));
    }
    
    println!("{} verbump uninstalled from pre-commit hook", "Success".green());
    
    Ok(())
}

fn show_version() -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run verbump from within a git repository.");
    }
    
    let version_info = VersionInfo::calculate()?;
    
    println!("{}", "Current Version Information:".bold());
    println!("  {}: {}", "Major (tag)".cyan(), version_info.major_version);
    println!("  {}: {}", "Minor (commits since tag)".cyan(), version_info.minor_version);
    println!("  {}: {}", "Patch (total changes)".cyan(), version_info.patch_version);
    println!("  {}: {}", "Full Version".green().bold(), version_info.full_version);
    
    Ok(())
}

fn update_version(force: bool) -> Result<()> {
    if !force && !is_git_repository() {
        anyhow::bail!("Not in a git repository. Use --force to update anyway.");
    }
    
    let git_root = if is_git_repository() {
        get_git_root()?
    } else {
        env::current_dir().context("Failed to get current directory")?
    };
    
    let config = VerbumpConfig::load(&git_root)?;
    
    if !config.enabled {
        println!("{} verbump is disabled in configuration", "Info".yellow());
        return Ok(());
    }
    
    let version_info = VersionInfo::calculate()?;
    let updated = update_version_file(&version_info, &config)?;
    
    if updated {
        println!("{} Updated version to: {}", "Success".green(), version_info.full_version);
        log_action(&format!("Updated version to: {} (file: {})", version_info.full_version, config.version_file));
    }
    
    Ok(())
}

fn show_status() -> Result<()> {
    if !is_git_repository() {
        println!("{}: Not in a git repository", "Status".red());
        return Ok(());
    }
    
    let git_root = get_git_root()?;
    let config = VerbumpConfig::load(&git_root)?;
    
    println!("{}", "Verbump Status:".bold());
    println!("  {}: {}", "Git Repository".cyan(), "✓".green());
    println!("  {}: {}", "Hook Installed".cyan(), 
        if is_hook_installed()? { "✓".green() } else { "✗".red() });
    println!("  {}: {}", "Enabled".cyan(), 
        if config.enabled { "✓".green() } else { "✗".red() });
    println!("  {}: {}", "Version File".cyan(), config.version_file);
    
    if let Ok(version_info) = VersionInfo::calculate() {
        println!("  {}: {}", "Current Version".cyan(), version_info.full_version);
    }
    
    // Check if version file exists
    let version_file_path = PathBuf::from(&config.version_file);
    println!("  {}: {}", "Version File Exists".cyan(),
        if version_file_path.exists() { "✓".green() } else { "✗".red() });
    
    // Show auto-detection status
    println!("  {}: {}", "Auto-detect Project Files".cyan(),
        if config.auto_detect_project_files { "✓".green() } else { "✗".red() });
    
    // Show detected project files
    if config.auto_detect_project_files {
        match detect_project_files(&git_root) {
            Ok(project_files) => {
                if !project_files.is_empty() {
                    println!("  {}: ", "Detected Project Files".cyan());
                    for project_file in &project_files {
                        println!("    • {} ({})", 
                            project_file.path.display(),
                            project_file.file_type.file_name());
                    }
                } else {
                    println!("  {}: {}", "Detected Project Files".cyan(), "None".yellow());
                }
            }
            Err(e) => {
                println!("  {}: {} ({})", "Detected Project Files".cyan(), "Error".red(), e);
            }
        }
    }
    
    // Show manually configured project files
    if !config.project_files.is_empty() {
        println!("  {}: ", "Configured Project Files".cyan());
        for file_path in &config.project_files {
            let full_path = git_root.join(file_path);
            println!("    • {} ({})", 
                file_path,
                if full_path.exists() { "✓".green() } else { "✗".red() });
        }
    }
    
    Ok(())
}

fn is_hook_installed() -> Result<bool> {
    if !is_git_repository() {
        return Ok(false);
    }
    
    let git_root = get_git_root()?;
    let hook_file = git_root.join(".git").join("hooks").join("pre-commit");
    
    if !hook_file.exists() {
        return Ok(false);
    }
    
    let content = fs::read_to_string(&hook_file)
        .context("Failed to read pre-commit hook")?;
    
    Ok(content.contains("=== VERBUMP BLOCK START ==="))
}

fn remove_verbump_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_verbump_block = false;
    let ends_with_newline = content.ends_with('\n');
    
    for line in lines {
        if line.contains("=== VERBUMP BLOCK START ===") {
            in_verbump_block = true;
            continue;
        }
        
        if line.contains("=== VERBUMP BLOCK END ===") {
            in_verbump_block = false;
            continue;
        }
        
        if !in_verbump_block {
            result.push(line);
        }
    }
    
    let mut output = result.join("\n");
    if ends_with_newline && !output.is_empty() {
        output.push('\n');
    }
    output
}

fn log_action(message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    
    // Try to append to log file, but don't fail if we can't
    if let Ok(git_root) = get_git_root() {
        let log_file = git_root.join(".verbump.log");
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }
}

fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_git_root() -> Result<PathBuf> {
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
    
    #[test]
    fn test_remove_verbump_block() {
        let content = r#"#!/bin/bash
# Some existing content
echo "Before verbump"

# === VERBUMP BLOCK START ===
# DO NOT EDIT THIS BLOCK MANUALLY
/path/to/verbump update --force
# === VERBUMP BLOCK END ===

echo "After verbump"
"#;
        
        let result = remove_verbump_block(content);
        assert!(!result.contains("VERBUMP BLOCK"));
        assert!(result.contains("Before verbump"));
        assert!(result.contains("After verbump"));
    }
    
    #[test]
    fn test_remove_verbump_block_only() {
        let content = r#"#!/bin/bash
# === VERBUMP BLOCK START ===
/path/to/verbump update --force
# === VERBUMP BLOCK END ===
"#;
        
        let result = remove_verbump_block(content);
        assert_eq!(result.trim(), "#!/bin/bash");
    }
    
    #[test]
    fn test_remove_verbump_block_none() {
        let content = r#"#!/bin/bash
echo "No verbump block here"
"#;
        
        let result = remove_verbump_block(content);
        assert_eq!(result, content);
    }
}