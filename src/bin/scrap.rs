use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use flate2::write::GzEncoder;
use flate2::Compression;
use refac::scrap::{ScrapEntry, ScrapMetadata};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;
use tar::Builder;

#[derive(Parser, Debug)]
#[command(name = "scrap")]
#[command(about = "Smart file/directory management with a .scrap folder")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to file or directory to move to .scrap folder
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List contents of .scrap folder
    #[command(alias = "ls")]
    List {
        /// Sort by: name, date, size
        #[arg(short, long, default_value = "date")]
        sort: String,
    },

    /// Clean old items from .scrap folder
    Clean {
        /// Remove items older than N days
        #[arg(short, long, default_value = "30")]
        days: u64,
        
        /// Show what would be removed without actually removing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Remove all items from .scrap folder
    Purge {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Search for files in .scrap folder
    #[command(alias = "search")]
    Find {
        /// Pattern to search for (supports regex)
        pattern: String,
        
        /// Search in file contents as well
        #[arg(short, long)]
        content: bool,
    },

    /// Archive .scrap folder contents
    Archive {
        /// Output archive file name (defaults to .scrap-YYYY-MM-DD.tar.gz)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Remove files after archiving
        #[arg(short, long)]
        remove: bool,
    },
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
    
    // Create .scrap directory if it doesn't exist
    create_scrap_directory(&scrap_dir)?;
    
    // Update .gitignore if it exists
    update_gitignore(&current_dir)?;
    
    match args.command {
        Some(Commands::List { sort }) => list_scrap_contents(&scrap_dir, &sort)?,
        Some(Commands::Clean { days, dry_run }) => clean_old_items(&scrap_dir, days, dry_run)?,
        Some(Commands::Purge { force }) => purge_scrap_folder(&scrap_dir, force)?,
        Some(Commands::Find { pattern, content }) => find_in_scrap(&scrap_dir, &pattern, content)?,
        Some(Commands::Archive { output, remove }) => archive_scrap_folder(&scrap_dir, output, remove)?,
        None => {
            match args.path {
                Some(path) => {
                    // Move the specified file/directory to .scrap
                    move_to_scrap(&path, &scrap_dir, &current_dir)?;
                }
                None => {
                    // No args - list contents with default sort
                    list_scrap_contents(&scrap_dir, "date")?;
                }
            }
        }
    }
    
    Ok(())
}

fn create_scrap_directory(scrap_dir: &Path) -> Result<()> {
    if !scrap_dir.exists() {
        fs::create_dir(scrap_dir)
            .with_context(|| format!("Failed to create .scrap directory at {:?}", scrap_dir))?;
    }
    Ok(())
}

fn update_gitignore(current_dir: &Path) -> Result<()> {
    let gitignore_path = current_dir.join(".gitignore");
    
    if gitignore_path.exists() {
        // Check if .scrap/ is already in .gitignore
        let file = fs::File::open(&gitignore_path)
            .context("Failed to open .gitignore")?;
        let reader = BufReader::new(file);
        
        let mut found = false;
        for line in reader.lines() {
            let line = line.context("Failed to read line from .gitignore")?;
            if line.trim() == ".scrap/" || line.trim() == ".scrap" {
                found = true;
                break;
            }
        }
        
        if !found {
            // Append .scrap/ to .gitignore
            let mut file = OpenOptions::new()
                .append(true)
                .open(&gitignore_path)
                .context("Failed to open .gitignore for appending")?;
            
            // Add newline if file doesn't end with one
            let contents = fs::read_to_string(&gitignore_path)?;
            if !contents.is_empty() && !contents.ends_with('\n') {
                writeln!(file)?;
            }
            
            writeln!(file, ".scrap/")?;
        }
    }
    
    Ok(())
}

fn move_to_scrap(source: &Path, scrap_dir: &Path, current_dir: &Path) -> Result<()> {
    if !source.exists() {
        anyhow::bail!("Path '{}' does not exist", source.display());
    }
    
    let absolute_source = if source.is_absolute() {
        source.to_path_buf()
    } else {
        current_dir.join(source)
    };
    
    let file_name = source.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path: no filename"))?;
    
    let mut dest = scrap_dir.join(file_name);
    
    // Handle naming conflicts
    if dest.exists() {
        dest = find_unique_name(&dest)?;
    }
    
    // Load or create metadata
    let mut metadata = ScrapMetadata::load(scrap_dir)?;
    
    // Move the file/directory
    fs::rename(&absolute_source, &dest)
        .with_context(|| format!("Failed to move '{}' to '{}'", absolute_source.display(), dest.display()))?;
    
    // Add to metadata
    metadata.add_entry(
        dest.file_name().unwrap().to_str().unwrap(),
        absolute_source.clone(),
    );
    metadata.save(scrap_dir)?;
    
    println!("{} '{}' to '{}'", "Moved".green(), source.display(), dest.display());
    
    Ok(())
}

fn find_unique_name(path: &Path) -> Result<PathBuf> {
    let parent = path.parent()
        .ok_or_else(|| anyhow::anyhow!("Path has no parent directory"))?;
    let file_stem = path.file_stem()
        .ok_or_else(|| anyhow::anyhow!("Path has no file stem"))?;
    let extension = path.extension();
    
    let mut counter = 1;
    loop {
        let new_name = if let Some(ext) = extension {
            format!("{}_{}.{}", file_stem.to_string_lossy(), counter, ext.to_string_lossy())
        } else {
            format!("{}_{}", file_stem.to_string_lossy(), counter)
        };
        
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return Ok(new_path);
        }
        
        counter += 1;
        if counter > 10000 {
            anyhow::bail!("Could not find unique name after 10000 attempts");
        }
    }
}


fn list_scrap_contents(scrap_dir: &Path, sort_by: &str) -> Result<()> {
    let metadata = ScrapMetadata::load(scrap_dir)?;
    
    if !scrap_dir.exists() || scrap_dir.read_dir()?.next().is_none() {
        println!("{}", "The .scrap folder is empty.".yellow());
        return Ok(());
    }
    
    let mut entries: Vec<(PathBuf, Option<ScrapEntry>)> = Vec::new();
    
    for entry in fs::read_dir(scrap_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        
        // Skip hidden files like .metadata.json
        if file_name.starts_with('.') {
            continue;
        }
        
        let scrap_entry = metadata.get_entry(file_name).cloned();
        entries.push((path, scrap_entry));
    }
    
    // Sort entries
    match sort_by {
        "name" => entries.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name())),
        "date" => entries.sort_by(|a, b| {
            let time_a = a.1.as_ref().map(|e| e.scrapped_at).unwrap_or_default();
            let time_b = b.1.as_ref().map(|e| e.scrapped_at).unwrap_or_default();
            time_b.cmp(&time_a)
        }),
        "size" => entries.sort_by_key(|e| {
            fs::metadata(&e.0).map(|m| m.len()).unwrap_or(0)
        }),
        _ => anyhow::bail!("Invalid sort option: {}", sort_by),
    }
    
    println!("{}", "Contents of .scrap folder:".bold());
    println!("{}", "-".repeat(80));
    
    for (path, scrap_entry) in entries {
        let metadata = fs::metadata(&path)?;
        let file_name = path.file_name().unwrap().to_string_lossy();
        let is_dir = metadata.is_dir();
        let size = if is_dir {
            "<DIR>".to_string()
        } else {
            format_size(metadata.len())
        };
        
        let type_indicator = if is_dir { "📁" } else { "📄" };
        
        if let Some(entry) = scrap_entry {
            let elapsed = Utc::now().signed_duration_since(entry.scrapped_at);
            let age = format_duration(elapsed);
            
            println!(
                "{} {:<30} {:>10} {:<15} from: {}",
                type_indicator,
                file_name.cyan(),
                size,
                age.dimmed(),
                entry.original_path.display().to_string().dimmed()
            );
        } else {
            println!(
                "{} {:<30} {:>10}",
                type_indicator,
                file_name.cyan(),
                size
            );
        }
    }
    
    Ok(())
}

fn clean_old_items(scrap_dir: &Path, days: u64, dry_run: bool) -> Result<()> {
    let mut metadata = ScrapMetadata::load(scrap_dir)?;
    let cutoff = Utc::now() - Duration::days(days as i64);
    let mut removed_count = 0;
    let mut total_size = 0u64;
    
    println!("{} items older than {} days...", 
        if dry_run { "Would remove" } else { "Removing" }, 
        days
    );
    
    for entry in fs::read_dir(scrap_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        
        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }
        
        let should_remove = if let Some(scrap_entry) = metadata.get_entry(file_name) {
            scrap_entry.scrapped_at < cutoff
        } else {
            // If no metadata, check file modification time
            let file_metadata = fs::metadata(&path)?;
            let modified = file_metadata.modified()?;
            let modified_time: chrono::DateTime<Utc> = modified.into();
            modified_time < cutoff
        };
        
        if should_remove {
            let size = if path.is_dir() {
                dir_size(&path)?
            } else {
                fs::metadata(&path)?.len()
            };
            
            total_size += size;
            
            if dry_run {
                println!("  {} {} ({})", "Would remove:".yellow(), path.display(), format_size(size));
            } else {
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
                metadata.remove_entry(file_name);
                println!("  {} {} ({})", "Removed:".red(), path.display(), format_size(size));
            }
            removed_count += 1;
        }
    }
    
    if !dry_run {
        metadata.save(scrap_dir)?;
    }
    
    println!("\n{}: {} items ({})", 
        if dry_run { "Would remove" } else { "Removed" }.bold(),
        removed_count,
        format_size(total_size)
    );
    
    Ok(())
}

fn purge_scrap_folder(scrap_dir: &Path, force: bool) -> Result<()> {
    if !scrap_dir.exists() {
        println!("{}", "The .scrap folder doesn't exist.".yellow());
        return Ok(());
    }
    
    let item_count = fs::read_dir(scrap_dir)?.count();
    
    if item_count == 0 {
        println!("{}", "The .scrap folder is already empty.".yellow());
        return Ok(());
    }
    
    if !force {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Are you sure you want to remove all {} items from .scrap?", item_count))
            .default(false)
            .interact()?;
        
        if !confirmed {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }
    
    let mut removed_count = 0;
    let mut total_size = 0u64;
    
    for entry in fs::read_dir(scrap_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        let size = if path.is_dir() {
            dir_size(&path)?
        } else {
            fs::metadata(&path)?.len()
        };
        
        total_size += size;
        
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
        removed_count += 1;
    }
    
    println!("{} {} items ({})", 
        "Purged:".red().bold(),
        removed_count,
        format_size(total_size)
    );
    
    Ok(())
}

fn find_in_scrap(scrap_dir: &Path, pattern: &str, search_content: bool) -> Result<()> {
    use regex::Regex;
    
    let regex = Regex::new(pattern)
        .with_context(|| format!("Invalid regex pattern: {}", pattern))?;
    
    let metadata = ScrapMetadata::load(scrap_dir)?;
    let mut found_count = 0;
    
    println!("{} for '{}'...\n", "Searching".bold(), pattern);
    
    for entry in fs::read_dir(scrap_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        
        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }
        
        // Check filename
        if regex.is_match(file_name) {
            print_search_result(&path, &metadata, file_name, "filename match");
            found_count += 1;
        } else if search_content && path.is_file() {
            // Check file content
            if let Ok(content) = fs::read_to_string(&path) {
                if regex.is_match(&content) {
                    print_search_result(&path, &metadata, file_name, "content match");
                    found_count += 1;
                }
            }
        }
    }
    
    if found_count == 0 {
        println!("{}", "No matches found.".yellow());
    } else {
        println!("\n{} {} matches", "Found".green().bold(), found_count);
    }
    
    Ok(())
}

fn archive_scrap_folder(scrap_dir: &Path, output: Option<PathBuf>, remove: bool) -> Result<()> {
    if !scrap_dir.exists() || fs::read_dir(scrap_dir)?.next().is_none() {
        println!("{}", "The .scrap folder is empty.".yellow());
        return Ok(());
    }
    
    let archive_name = output.unwrap_or_else(|| {
        let date = chrono::Local::now().format("%Y-%m-%d");
        PathBuf::from(format!(".scrap-{}.tar.gz", date))
    });
    
    println!("{} .scrap folder to {}...", "Archiving".bold(), archive_name.display());
    
    let tar_gz = fs::File::create(&archive_name)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    
    let mut file_count = 0;
    let mut total_size = 0u64;
    
    for entry in fs::read_dir(scrap_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        
        // Skip hidden files except .metadata.json
        if file_name.starts_with('.') && file_name != ".metadata.json" {
            continue;
        }
        
        if path.is_dir() {
            tar.append_dir_all(file_name, &path)?;
            total_size += dir_size(&path)?;
        } else {
            tar.append_path_with_name(&path, file_name)?;
            total_size += fs::metadata(&path)?.len();
        }
        file_count += 1;
    }
    
    tar.finish()?;
    
    println!("{} {} files ({}) to {}", 
        "Archived".green().bold(),
        file_count,
        format_size(total_size),
        archive_name.display()
    );
    
    if remove {
        for entry in fs::read_dir(scrap_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            
            // Don't remove hidden files
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
        
        // Clear metadata
        let metadata = ScrapMetadata::new();
        metadata.save(scrap_dir)?;
        
        println!("{} archived files from .scrap folder", "Removed".red());
    }
    
    Ok(())
}

// Helper functions

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

fn format_duration(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    
    if days > 0 {
        format!("{} days ago", days)
    } else if hours > 0 {
        format!("{} hours ago", hours)
    } else if minutes > 0 {
        format!("{} mins ago", minutes)
    } else {
        "just now".to_string()
    }
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            size += entry.metadata()?.len();
        }
    }
    
    Ok(size)
}

fn print_search_result(path: &Path, metadata: &ScrapMetadata, file_name: &str, match_type: &str) {
    let type_indicator = if path.is_dir() { "📁" } else { "📄" };
    
    print!("{} {} ", type_indicator, file_name.cyan());
    
    if let Some(entry) = metadata.get_entry(file_name) {
        let elapsed = Utc::now().signed_duration_since(entry.scrapped_at);
        let age = format_duration(elapsed);
        print!("({}, from: {}) ", age.dimmed(), entry.original_path.display().to_string().dimmed());
    }
    
    println!("[{}]", match_type.yellow());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_scrap_directory() {
        let temp_dir = TempDir::new().unwrap();
        let scrap_dir = temp_dir.path().join(".scrap");
        
        assert!(!scrap_dir.exists());
        create_scrap_directory(&scrap_dir).unwrap();
        assert!(scrap_dir.exists());
        
        // Should not fail if directory already exists
        create_scrap_directory(&scrap_dir).unwrap();
    }
    
    #[test]
    fn test_update_gitignore_no_file() {
        let temp_dir = TempDir::new().unwrap();
        // Should not fail if .gitignore doesn't exist
        update_gitignore(temp_dir.path()).unwrap();
    }
    
    #[test]
    fn test_update_gitignore_add_entry() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        
        // Create .gitignore with some content
        fs::write(&gitignore_path, "*.log\ntarget/\n").unwrap();
        
        update_gitignore(temp_dir.path()).unwrap();
        
        let contents = fs::read_to_string(&gitignore_path).unwrap();
        assert!(contents.contains(".scrap/"));
    }
    
    #[test]
    fn test_find_unique_name() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().join("test.txt");
        
        // First call should add _1
        let unique = find_unique_name(&base_path).unwrap();
        assert_eq!(unique.file_name().unwrap(), "test_1.txt");
        
        // Create the file and test again
        fs::write(&temp_dir.path().join("test_1.txt"), "").unwrap();
        let unique = find_unique_name(&base_path).unwrap();
        assert_eq!(unique.file_name().unwrap(), "test_2.txt");
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }
}