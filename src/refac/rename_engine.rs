use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};

use crate::{
    ItemType, RenameConfig, RenameItem, RenameStats, utils,
};
use super::{
    cli::{Args, Mode, OutputFormat},
    collision_detector::{CollisionDetector, CollisionType},
    file_ops::FileOperations,
    progress::{ProgressTracker, SimpleOutput},
};

/// Main engine for executing rename operations
pub struct RenameEngine {
    config: RenameConfig,
    mode: Mode,
    file_ops: FileOperations,
    progress: Option<ProgressTracker>,
    simple_output: Option<SimpleOutput>,
    thread_count: usize,
    output_format: OutputFormat,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    ignore_case: bool,
    use_regex: bool,
}

impl RenameEngine {
    pub fn new(args: Args) -> Result<Self> {
        // Validate arguments
        args.validate().map_err(|e| anyhow::anyhow!(e))?;

        // Create configuration
        let config = RenameConfig::new(&args.root_dir, args.old_string.clone(), args.new_string.clone())?
            .with_dry_run(args.dry_run)
            .with_force(args.force)
            .with_verbose(args.verbose)
            .with_follow_symlinks(args.follow_symlinks)
            .with_backup(args.backup);

        // Setup progress tracking
        let show_progress = match args.progress {
            super::cli::ProgressMode::Always => true,
            super::cli::ProgressMode::Never => false,
            super::cli::ProgressMode::Auto => atty::is(atty::Stream::Stdout),
        };

        let (progress, simple_output) = if show_progress && args.format == OutputFormat::Human {
            (Some(ProgressTracker::new(true, args.verbose)), None)
        } else {
            (None, Some(SimpleOutput::new(args.verbose)))
        };

        Ok(Self {
            config,
            mode: args.get_mode(),
            file_ops: FileOperations::new().with_backup(args.backup),
            progress,
            simple_output,
            thread_count: args.get_thread_count(),
            output_format: args.format,
            max_depth: if args.max_depth > 0 { Some(args.max_depth) } else { None },
            include_patterns: args.include_patterns,
            exclude_patterns: args.exclude_patterns,
            ignore_case: args.ignore_case,
            use_regex: args.use_regex,
        })
    }

    /// Execute the rename operation
    pub fn execute(&self) -> Result<()> {
        self.print_header()?;

        // Phase 1: Discovery
        self.print_info("Phase 1: Discovering files and directories...")?;
        let (content_files, rename_items) = self.discover_items()?;

        // Phase 2: Collision Detection
        self.print_info("Phase 2: Checking for naming collisions...")?;
        self.check_collisions(&rename_items)?;

        // Phase 3: Summary and Confirmation
        let stats = self.show_summary(&content_files, &rename_items)?;
        if stats.total_changes() == 0 {
            self.print_success("No changes needed.")?;
            return Ok(());
        }

        if !self.confirm_changes()? {
            self.print_info("Operation cancelled by user.")?;
            return Ok(());
        }

        // Phase 4: Execute Changes
        self.execute_changes(&content_files, &rename_items)?;

        // Phase 5: Final Report
        self.show_final_report(&stats)?;

        Ok(())
    }

    /// Discover files for content replacement and items for renaming
    fn discover_items(&self) -> Result<(Vec<PathBuf>, Vec<RenameItem>)> {
        let mut content_files = Vec::new();
        let mut rename_items = Vec::new();

        // Setup progress
        if let Some(progress) = &self.progress {
            progress.init_main_progress(0, "Scanning files and directories...");
        }

        // Walk the directory tree
        let walker = WalkDir::new(&self.config.root_dir)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_entry(|e| self.should_process_entry(e));

        for entry in walker {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            // Skip the root directory itself
            if path == self.config.root_dir {
                continue;
            }

            // Apply include/exclude patterns
            if !self.matches_patterns(path)? {
                continue;
            }

            // Check for content replacement in files
            if self.should_process_content() && 
               self.should_process_files() && 
               path.is_file() {
                if self.file_needs_content_replacement(path)? {
                    content_files.push(path.to_path_buf());
                }
            }

            // Check for renaming
            if self.should_process_names() {
                if let Some(rename_item) = self.create_rename_item(path)? {
                    rename_items.push(rename_item);
                }
            }

            // Update progress
            if let Some(progress) = &self.progress {
                progress.update_main(&format!("Scanned: {}", path.display()));
            }
        }

        // Sort rename items by depth (deepest first for directories)
        rename_items.sort_by(|a, b| {
            match (&a.item_type, &b.item_type) {
                (ItemType::Directory, ItemType::Directory) => b.depth.cmp(&a.depth),
                (ItemType::Directory, ItemType::File) => std::cmp::Ordering::Less,
                (ItemType::File, ItemType::Directory) => std::cmp::Ordering::Greater,
                (ItemType::File, ItemType::File) => a.depth.cmp(&b.depth),
            }
        });

        if let Some(progress) = &self.progress {
            progress.finish_main("Discovery complete");
        }

        Ok((content_files, rename_items))
    }

    /// Check if an entry should be processed
    fn should_process_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        
        // Don't skip the root directory itself, even if it's hidden
        if path == self.config.root_dir {
            return true;
        }
        
        // Skip hidden files unless explicitly included
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    let should_include = self.include_patterns.iter().any(|p| p == ".*" || p.contains("*"));
                    if !should_include {
                        return false;
                    }
                }
            }
        }

        // Check file type restrictions
        match self.mode {
            Mode::FilesOnly => path.is_file(),
            Mode::DirsOnly => path.is_dir(),
            _ => true,
        }
    }

    /// Check if a path matches include/exclude patterns
    fn matches_patterns(&self, path: &Path) -> Result<bool> {
        // If there are include patterns, the file must match at least one
        if !self.include_patterns.is_empty() {
            let matches = self.include_patterns.iter().any(|pattern| {
                self.path_matches_pattern(path, pattern)
            });
            if !matches {
                return Ok(false);
            }
        }

        // If there are exclude patterns, the file must not match any
        if !self.exclude_patterns.is_empty() {
            let excluded = self.exclude_patterns.iter().any(|pattern| {
                self.path_matches_pattern(path, pattern)
            });
            if excluded {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a path matches a glob pattern
    fn path_matches_pattern(&self, path: &Path, pattern: &str) -> bool {
        // Simple glob matching - could be enhanced with a proper glob library
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let pattern_str = if self.ignore_case {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };
        let compare_str = if self.ignore_case {
            file_name.to_lowercase()
        } else {
            file_name.to_string()
        };


        if self.use_regex {
            // Use regex matching
            if let Ok(regex) = regex::Regex::new(&pattern_str) {
                return regex.is_match(&compare_str);
            }
        }

        // Special case for ".*" pattern to match hidden files
        if pattern_str == ".*" {
            return compare_str.starts_with('.');
        }

        // Simple glob-style matching
        if pattern_str.contains('*') {
            let parts: Vec<&str> = pattern_str.split('*').collect();
            if parts.len() == 2 {
                return compare_str.starts_with(parts[0]) && compare_str.ends_with(parts[1]);
            }
        }

        compare_str.contains(&pattern_str)
    }

    /// Check if a file needs content replacement
    fn file_needs_content_replacement(&self, path: &Path) -> Result<bool> {
        if !self.file_ops.is_text_file(path)? {
            return Ok(false);
        }

        let search_string = if self.ignore_case {
            // For case-insensitive search, we'd need to read the file content
            // This is simplified - a full implementation would use regex
            &self.config.old_string.to_lowercase()
        } else {
            &self.config.old_string
        };

        self.file_ops.file_contains_string(path, search_string)
    }

    /// Create a rename item if the path needs renaming
    fn create_rename_item(&self, path: &Path) -> Result<Option<RenameItem>> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", path.display()))?;

        let contains_pattern = if self.ignore_case {
            file_name.to_lowercase().contains(&self.config.old_string.to_lowercase())
        } else {
            file_name.contains(&self.config.old_string)
        };


        if !contains_pattern {
            return Ok(None);
        }

        // Apply type restrictions
        let item_type = if path.is_file() {
            if !self.should_process_files() {
                return Ok(None);
            }
            ItemType::File
        } else {
            if !self.should_process_dirs() {
                return Ok(None);
            }
            ItemType::Directory
        };

        // Calculate new name
        let new_name = if self.ignore_case {
            // Case-insensitive replacement
            file_name.to_lowercase().replace(
                &self.config.old_string.to_lowercase(),
                &self.config.new_string
            )
        } else {
            utils::replace_all(file_name, &self.config.old_string, &self.config.new_string)
        };

        let new_path = path.with_file_name(new_name);
        let depth = utils::calculate_depth(path, &self.config.root_dir);

        Ok(Some(RenameItem {
            original_path: path.to_path_buf(),
            new_path,
            item_type,
            depth,
        }))
    }

    /// Check for collisions in the rename operations
    fn check_collisions(&self, rename_items: &[RenameItem]) -> Result<()> {
        if rename_items.is_empty() {
            return Ok(());
        }

        let mut detector = CollisionDetector::new();
        
        // Scan existing paths
        detector.scan_existing_paths(&self.config.root_dir)?;
        
        // Add rename operations
        detector.add_renames(rename_items);
        
        // Detect collisions
        let collisions = detector.detect_collisions()?;
        
        if !collisions.is_empty() {
            self.print_error("Naming collisions detected!")?;
            
            for collision in &collisions {
                match collision.collision_type {
                    CollisionType::SourceEqualsTarget => {
                        // Skip no-op renames
                        continue;
                    }
                    _ => {
                        self.print_error(&collision.description)?;
                    }
                }
            }
            
            let serious_collisions: Vec<_> = collisions.iter()
                .filter(|c| c.collision_type != CollisionType::SourceEqualsTarget)
                .collect();
                
            if !serious_collisions.is_empty() {
                anyhow::bail!("Cannot proceed due to {} naming collision(s)", serious_collisions.len());
            }
        }

        Ok(())
    }

    /// Show summary of changes and get user confirmation
    fn show_summary(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<RenameStats> {
        let mut stats = RenameStats::default();
        stats.files_with_content_changes = content_files.len();
        stats.files_renamed = rename_items.iter().filter(|item| item.item_type == ItemType::File).count();
        stats.directories_renamed = rename_items.iter().filter(|item| item.item_type == ItemType::Directory).count();

        match self.output_format {
            OutputFormat::Json => {
                let summary = serde_json::json!({
                    "summary": {
                        "content_changes": stats.files_with_content_changes,
                        "file_renames": stats.files_renamed,
                        "directory_renames": stats.directories_renamed,
                        "total_changes": stats.total_changes()
                    },
                    "dry_run": self.config.dry_run
                });
                println!("{}", serde_json::to_string_pretty(&summary)?);
            }
            OutputFormat::Plain => {
                println!("Content changes: {}", stats.files_with_content_changes);
                println!("File renames: {}", stats.files_renamed);
                println!("Directory renames: {}", stats.directories_renamed);
                println!("Total changes: {}", stats.total_changes());
            }
            OutputFormat::Human => {
                self.print_info("=== CHANGE SUMMARY ===")?;
                self.print_info(&format!("Content modifications: {} file(s)", stats.files_with_content_changes))?;
                self.print_info(&format!("File renames:         {} file(s)", stats.files_renamed))?;
                self.print_info(&format!("Directory renames:    {} directory(ies)", stats.directories_renamed))?;
                self.print_info(&format!("Total changes:        {}", stats.total_changes()))?;

                if self.config.verbose {
                    if !content_files.is_empty() {
                        self.print_info("\nFiles with content to modify:")?;
                        for file in content_files {
                            self.print_verbose(&format!("  {}", file.display()))?;
                        }
                    }

                    if !rename_items.is_empty() {
                        self.print_info("\nItems to rename:")?;
                        for item in rename_items {
                            self.print_verbose(&format!("  {} → {}", 
                                item.original_path.display(), 
                                item.new_path.display()))?;
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Confirm changes with the user
    fn confirm_changes(&self) -> Result<bool> {
        if self.config.force || self.config.dry_run {
            return Ok(true);
        }

        match self.output_format {
            OutputFormat::Json => Ok(true), // No confirmation in JSON mode
            OutputFormat::Plain | OutputFormat::Human => {
                self.print_warning("This operation will modify your files and directories.")?;
                
                let confirmation = if let Some(progress) = &self.progress {
                    progress.suspend(|| {
                        dialoguer::Confirm::new()
                            .with_prompt("Do you want to proceed?")
                            .default(false)
                            .interact()
                    })
                } else {
                    dialoguer::Confirm::new()
                        .with_prompt("Do you want to proceed?")
                        .default(false)
                        .interact()
                };

                confirmation.with_context(|| "Failed to get user confirmation")
            }
        }
    }

    /// Execute the actual changes
    fn execute_changes(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<()> {
        if self.config.dry_run {
            self.print_info("DRY RUN MODE: No actual changes will be made.")?;
            return Ok(());
        }

        // Phase 1: Content replacement
        if !content_files.is_empty() && self.should_process_content() {
            self.execute_content_changes(content_files)?;
        }

        // Phase 2: Rename items (directories first, then files)
        if !rename_items.is_empty() && self.should_process_names() {
            self.execute_renames(rename_items)?;
        }

        Ok(())
    }

    /// Execute content changes
    fn execute_content_changes(&self, content_files: &[PathBuf]) -> Result<()> {
        self.print_info("Replacing content in files...")?;

        if let Some(progress) = &self.progress {
            progress.init_content_progress(content_files.len() as u64);
        }

        let errors = Arc::new(Mutex::new(Vec::new()));
        let _progress_ref = &self.progress;
        let config_ref = &self.config;
        let file_ops_ref = &self.file_ops;
        let errors_ref = Arc::clone(&errors);

        if self.thread_count > 1 {
            // Parallel processing (without progress updates due to thread safety)
            content_files.par_iter().for_each(|file_path| {
                let result = file_ops_ref.replace_content(
                    file_path,
                    &config_ref.old_string,
                    &config_ref.new_string,
                );

                match result {
                    Err(e) => {
                        errors_ref.lock().unwrap().push(format!("Failed to modify {}: {}", file_path.display(), e));
                    }
                    _ => {}
                }
            });
        } else {
            // Sequential processing
            for file_path in content_files {
                let result = file_ops_ref.replace_content(
                    file_path,
                    &config_ref.old_string,
                    &config_ref.new_string,
                );

                match result {
                    Ok(modified) => {
                        if modified && config_ref.verbose {
                            self.print_verbose(&format!("Modified: {}", file_path.display()))?;
                        }
                    }
                    Err(e) => {
                        self.print_error(&format!("Failed to modify {}: {}", file_path.display(), e))?;
                    }
                }

                if let Some(progress) = &self.progress {
                    progress.update_content(&file_path.display().to_string());
                }
            }
        }

        // Report any errors from parallel processing
        let errors = errors.lock().unwrap();
        for error in errors.iter() {
            self.print_error(error)?;
        }

        if let Some(progress) = &self.progress {
            progress.finish_content(&format!("Content replacement complete ({} files)", content_files.len()));
        }

        Ok(())
    }

    /// Execute rename operations
    fn execute_renames(&self, rename_items: &[RenameItem]) -> Result<()> {
        self.print_info("Renaming files and directories...")?;

        if let Some(progress) = &self.progress {
            progress.init_rename_progress(rename_items.len() as u64);
        }

        for item in rename_items {
            // Skip no-op renames
            if item.original_path == item.new_path {
                if let Some(progress) = &self.progress {
                    progress.update_rename(&item.original_path.display().to_string());
                }
                continue;
            }

            let result = self.file_ops.move_item(&item.original_path, &item.new_path);

            match result {
                Ok(()) => {
                    if self.config.verbose {
                        self.print_verbose(&format!("Renamed: {} → {}", 
                            item.original_path.display(), 
                            item.new_path.display()))?;
                    }
                }
                Err(e) => {
                    self.print_error(&format!("Failed to rename {} to {}: {}", 
                        item.original_path.display(), 
                        item.new_path.display(),
                        e))?;
                }
            }

            if let Some(progress) = &self.progress {
                progress.update_rename(&item.original_path.display().to_string());
            }
        }

        if let Some(progress) = &self.progress {
            progress.finish_rename(&format!("Rename complete ({} items)", rename_items.len()));
        }

        Ok(())
    }

    /// Show final report
    fn show_final_report(&self, stats: &RenameStats) -> Result<()> {
        match self.output_format {
            OutputFormat::Json => {
                let report = serde_json::json!({
                    "result": "success",
                    "stats": {
                        "content_changes": stats.files_with_content_changes,
                        "file_renames": stats.files_renamed,
                        "directory_renames": stats.directories_renamed,
                        "total_changes": stats.total_changes(),
                        "errors": stats.errors.len()
                    },
                    "dry_run": self.config.dry_run
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            OutputFormat::Plain => {
                if self.config.dry_run {
                    println!("Dry run complete. No changes were made.");
                } else {
                    println!("Operation completed successfully.");
                }
                println!("Total changes: {}", stats.total_changes());
            }
            OutputFormat::Human => {
                self.print_success("=== OPERATION COMPLETE ===")?;
                if self.config.dry_run {
                    self.print_info("Dry run complete. No changes were made.")?;
                } else {
                    self.print_success("Operation completed successfully!")?;
                    self.print_info(&format!("Total changes applied: {}", stats.total_changes()))?;
                }

                if !stats.errors.is_empty() {
                    self.print_warning(&format!("{} error(s) occurred:", stats.errors.len()))?;
                    for error in &stats.errors {
                        self.print_error(error)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Print header information
    fn print_header(&self) -> Result<()> {
        if self.output_format != OutputFormat::Human {
            return Ok(());
        }

        self.print_success("=== NOMION REFAC TOOL ===")?;
        self.print_info(&format!("Root directory: {}", self.config.root_dir.display()))?;
        self.print_info(&format!("Old string: '{}'", self.config.old_string))?;
        self.print_info(&format!("New string: '{}'", self.config.new_string))?;
        self.print_info(&format!("Mode: {:?}", self.mode))?;
        
        if self.config.dry_run {
            self.print_warning("DRY RUN MODE: No changes will be made")?;
        }
        
        if self.config.backup {
            self.print_info("Backup mode: Enabled")?;
        }

        Ok(())
    }

    // Utility methods for printing
    fn print_info(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_info(message);
        } else if let Some(output) = &self.simple_output {
            output.print_info(message);
        }
        Ok(())
    }

    fn print_error(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_error(message);
        } else if let Some(output) = &self.simple_output {
            output.print_error(message);
        }
        Ok(())
    }

    fn print_warning(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_warning(message);
        } else if let Some(output) = &self.simple_output {
            output.print_warning(message);
        }
        Ok(())
    }

    fn print_success(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_success(message);
        } else if let Some(output) = &self.simple_output {
            output.print_success(message);
        }
        Ok(())
    }

    fn print_verbose(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_verbose(message);
        } else if let Some(output) = &self.simple_output {
            output.print_verbose(message);
        }
        Ok(())
    }

    // Mode checking methods
    fn should_process_files(&self) -> bool {
        self.mode.should_process_files()
    }

    fn should_process_dirs(&self) -> bool {
        self.mode.should_process_dirs()
    }

    fn should_process_content(&self) -> bool {
        !matches!(self.mode, Mode::NamesOnly)
    }

    fn should_process_names(&self) -> bool {
        !matches!(self.mode, Mode::ContentOnly)
    }
}

// Extension traits to add methods to the Mode and Config types
trait ModeExt {
    fn should_process_files(&self) -> bool;
    fn should_process_dirs(&self) -> bool;
}

impl ModeExt for Mode {
    fn should_process_files(&self) -> bool {
        matches!(self, Mode::Full | Mode::FilesOnly | Mode::ContentOnly | Mode::NamesOnly)
    }

    fn should_process_dirs(&self) -> bool {
        matches!(self, Mode::Full | Mode::DirsOnly | Mode::NamesOnly)
    }
}

