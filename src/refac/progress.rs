use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::cell::RefCell;
use std::time::Duration;
use colored::*;

/// Progress tracking for the rename operation
pub struct ProgressTracker {
    multi_progress: MultiProgress,
    main_bar: RefCell<Option<ProgressBar>>,
    content_bar: RefCell<Option<ProgressBar>>,
    rename_bar: RefCell<Option<ProgressBar>>,
    enabled: bool,
    verbose: bool,
}

impl ProgressTracker {
    pub fn new(enabled: bool, verbose: bool) -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            main_bar: RefCell::new(None),
            content_bar: RefCell::new(None),
            rename_bar: RefCell::new(None),
            enabled,
            verbose,
        }
    }

    /// Initialize the main progress bar
    pub fn init_main_progress(&self, total: u64, message: &str) {
        if !self.enabled {
            return;
        }

        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        *self.main_bar.borrow_mut() = Some(pb);
    }

    /// Initialize content replacement progress bar
    pub fn init_content_progress(&self, total: u64) {
        if !self.enabled || total == 0 {
            return;
        }

        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.yellow} Content: [{bar:30.yellow/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message("Replacing content".to_string());
        *self.content_bar.borrow_mut() = Some(pb);
    }

    /// Initialize rename progress bar
    pub fn init_rename_progress(&self, total: u64) {
        if !self.enabled || total == 0 {
            return;
        }

        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.magenta} Rename: [{bar:30.magenta/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message("Renaming files/directories".to_string());
        *self.rename_bar.borrow_mut() = Some(pb);
    }

    /// Update main progress
    pub fn update_main(&self, message: &str) {
        if let Some(pb) = self.main_bar.borrow().as_ref() {
            pb.inc(1);
            if self.verbose {
                pb.set_message(message.to_string());
            }
        }
    }

    /// Update content progress
    pub fn update_content(&self, file_path: &str) {
        if let Some(pb) = self.content_bar.borrow().as_ref() {
            pb.inc(1);
            if self.verbose {
                pb.set_message(format!("Processing: {}", file_path));
            }
        }
    }

    /// Update rename progress
    pub fn update_rename(&self, item_path: &str) {
        if let Some(pb) = self.rename_bar.borrow().as_ref() {
            pb.inc(1);
            if self.verbose {
                pb.set_message(format!("Renaming: {}", item_path));
            }
        }
    }

    /// Finish content progress
    pub fn finish_content(&self, message: &str) {
        if let Some(pb) = self.content_bar.borrow().as_ref() {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Finish rename progress
    pub fn finish_rename(&self, message: &str) {
        if let Some(pb) = self.rename_bar.borrow().as_ref() {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Finish main progress
    pub fn finish_main(&self, message: &str) {
        if let Some(pb) = self.main_bar.borrow().as_ref() {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Print a message without interfering with progress bars
    pub fn println(&self, message: &str) {
        if self.enabled {
            self.multi_progress.println(message).unwrap_or(());
        } else {
            println!("{}", message);
        }
    }

    /// Print an error message
    pub fn print_error(&self, message: &str) {
        let error_msg = format!("{} {}", "ERROR:".red().bold(), message);
        self.println(&error_msg);
    }

    /// Print a warning message
    pub fn print_warning(&self, message: &str) {
        let warning_msg = format!("{} {}", "WARNING:".yellow().bold(), message);
        self.println(&warning_msg);
    }

    /// Print an info message
    pub fn print_info(&self, message: &str) {
        let info_msg = format!("{} {}", "INFO:".blue().bold(), message);
        self.println(&info_msg);
    }

    /// Print a success message
    pub fn print_success(&self, message: &str) {
        let success_msg = format!("{} {}", "SUCCESS:".green().bold(), message);
        self.println(&success_msg);
    }

    /// Print verbose output
    pub fn print_verbose(&self, message: &str) {
        if self.verbose {
            let verbose_msg = format!("{} {}", "VERBOSE:".cyan(), message);
            self.println(&verbose_msg);
        }
    }

    /// Suspend progress bars for user input
    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if self.enabled {
            self.multi_progress.suspend(f)
        } else {
            f()
        }
    }

    /// Clear all progress bars
    pub fn clear(&self) {
        if let Some(pb) = self.main_bar.borrow().as_ref() {
            pb.finish_and_clear();
        }
        if let Some(pb) = self.content_bar.borrow().as_ref() {
            pb.finish_and_clear();
        }
        if let Some(pb) = self.rename_bar.borrow().as_ref() {
            pb.finish_and_clear();
        }
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Simple console output without progress bars
pub struct SimpleOutput {
    verbose: bool,
}

impl SimpleOutput {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn println(&self, message: &str) {
        println!("{}", message);
    }

    pub fn print_error(&self, message: &str) {
        eprintln!("{} {}", "ERROR:".red().bold(), message);
    }

    pub fn print_warning(&self, message: &str) {
        println!("{} {}", "WARNING:".yellow().bold(), message);
    }

    pub fn print_info(&self, message: &str) {
        println!("{} {}", "INFO:".blue().bold(), message);
    }

    pub fn print_success(&self, message: &str) {
        println!("{} {}", "SUCCESS:".green().bold(), message);
    }

    pub fn print_verbose(&self, message: &str) {
        if self.verbose {
            println!("{} {}", "VERBOSE:".cyan(), message);
        }
    }

    pub fn print_step(&self, step: usize, total: usize, message: &str) {
        println!("[{}/{}] {}", step, total, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(true, true);
        assert!(tracker.enabled);
        assert!(tracker.verbose);
        assert!(tracker.main_bar.borrow().is_none());
        assert!(tracker.content_bar.borrow().is_none());
        assert!(tracker.rename_bar.borrow().is_none());
    }

    #[test]
    fn test_progress_tracker_disabled() {
        let tracker = ProgressTracker::new(false, false);
        
        // Should not create progress bars when disabled
        tracker.init_main_progress(100, "test");
        tracker.init_content_progress(50);
        tracker.init_rename_progress(25);
        
        assert!(tracker.main_bar.borrow().is_none());
        assert!(tracker.content_bar.borrow().is_none());
        assert!(tracker.rename_bar.borrow().is_none());
    }

    #[test]
    fn test_simple_output() {
        let output = SimpleOutput::new(true);
        assert!(output.verbose);
        
        // These should not panic
        output.println("test message");
        output.print_info("info message");
        output.print_verbose("verbose message");
        output.print_step(1, 5, "step message");
    }

    #[test]
    fn test_simple_output_non_verbose() {
        let output = SimpleOutput::new(false);
        assert!(!output.verbose);
        
        // These should not panic
        output.println("test message");
        output.print_info("info message");
        output.print_verbose("verbose message"); // Should not print
        output.print_step(1, 5, "step message");
    }
}