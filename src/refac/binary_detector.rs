use anyhow::{Context, Result};
use content_inspector::{ContentType, inspect};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Detects if a file is binary or text
pub struct BinaryDetector {
    /// Maximum number of bytes to read for detection
    max_sample_size: usize,
    /// Threshold for binary detection (percentage of non-printable characters)
    binary_threshold: f64,
}

impl Default for BinaryDetector {
    fn default() -> Self {
        Self {
            max_sample_size: 8192, // 8KB sample
            binary_threshold: 0.3,  // 30% non-printable = binary
        }
    }
}

impl BinaryDetector {
    pub fn new(max_sample_size: usize, binary_threshold: f64) -> Self {
        Self {
            max_sample_size,
            binary_threshold,
        }
    }

    /// Check if a file is binary using multiple detection methods
    pub fn is_binary<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = path.as_ref();
        
        // First, check file extension for known binary types
        if self.is_binary_by_extension(path) {
            return Ok(true);
        }

        // Check using content_inspector crate (fast method)
        if let Ok(content_type) = self.detect_by_content_inspector(path) {
            match content_type {
                ContentType::BINARY => return Ok(true),
                ContentType::UTF_8 | ContentType::UTF_8_BOM | 
                ContentType::UTF_16LE | ContentType::UTF_16BE |
                ContentType::UTF_32LE | ContentType::UTF_32BE => return Ok(false),
            }
        }

        // Fallback to manual analysis
        self.is_binary_by_content_analysis(path)
    }

    /// Check if file is likely binary based on file extension
    fn is_binary_by_extension(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return BINARY_EXTENSIONS.contains(&ext_lower.as_str());
            }
        }
        false
    }

    /// Use content_inspector crate for fast detection
    fn detect_by_content_inspector(&self, path: &Path) -> Result<ContentType> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file for binary detection: {}", path.display()))?;
        
        let mut buffer = vec![0; self.max_sample_size];
        let bytes_read = file.read(&mut buffer)
            .with_context(|| format!("Failed to read file for binary detection: {}", path.display()))?;
        
        buffer.truncate(bytes_read);
        Ok(inspect(&buffer))
    }

    /// Manual content analysis for edge cases
    fn is_binary_by_content_analysis(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open file for content analysis: {}", path.display()))?;
        
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.max_sample_size];
        
        let bytes_read = reader.read(&mut buffer)
            .with_context(|| format!("Failed to read file for content analysis: {}", path.display()))?;
        
        if bytes_read == 0 {
            return Ok(false); // Empty files are treated as text
        }

        buffer.truncate(bytes_read);
        
        // Check for null bytes (strong indicator of binary)
        if buffer.contains(&0) {
            return Ok(true);
        }

        // Count non-printable characters
        let non_printable_count = buffer.iter()
            .filter(|&&byte| !is_printable_ascii(byte) && !is_valid_utf8_start(byte))
            .count();

        let non_printable_ratio = non_printable_count as f64 / bytes_read as f64;
        Ok(non_printable_ratio > self.binary_threshold)
    }

    /// Check if the file appears to be a text file that we should process
    pub fn is_text_file<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        Ok(!self.is_binary(path)?)
    }

    /// Get a description of why a file is considered binary
    pub fn get_binary_reason<P: AsRef<Path>>(&self, path: P) -> Result<Option<String>> {
        let path = path.as_ref();
        
        if self.is_binary_by_extension(path) {
            return Ok(Some("Binary file extension".to_string()));
        }

        if let Ok(content_type) = self.detect_by_content_inspector(path) {
            match content_type {
                ContentType::BINARY => return Ok(Some("Content inspection detected binary".to_string())),
                _ => {}
            }
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.max_sample_size];
        let bytes_read = reader.read(&mut buffer)?;
        
        if bytes_read == 0 {
            return Ok(None);
        }

        buffer.truncate(bytes_read);
        
        if buffer.contains(&0) {
            return Ok(Some("Contains null bytes".to_string()));
        }

        let non_printable_count = buffer.iter()
            .filter(|&&byte| !is_printable_ascii(byte) && !is_valid_utf8_start(byte))
            .count();

        let non_printable_ratio = non_printable_count as f64 / bytes_read as f64;
        if non_printable_ratio > self.binary_threshold {
            return Ok(Some(format!("High ratio of non-printable characters: {:.1}%", non_printable_ratio * 100.0)));
        }

        Ok(None)
    }
}

/// Check if a byte is printable ASCII
fn is_printable_ascii(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7E | 0x09 | 0x0A | 0x0D) // printable ASCII + tab, newline, carriage return
}

/// Check if a byte could be the start of a valid UTF-8 sequence
fn is_valid_utf8_start(byte: u8) -> bool {
    // UTF-8 start bytes: 0xxxxxxx, 110xxxxx, 1110xxxx, 11110xxx
    byte < 0x80 || (byte >= 0xC0 && byte < 0xF8)
}

/// Common binary file extensions
const BINARY_EXTENSIONS: &[&str] = &[
    // Executables
    "exe", "dll", "so", "dylib", "app", "deb", "rpm", "msi", "dmg",
    // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "cab",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "svg", "ico", "cur",
    // Videos
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp",
    // Audio
    "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma",
    // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
    // Databases
    "db", "sqlite", "sqlite3", "mdb", "accdb",
    // Object files
    "o", "obj", "lib", "a", "pdb",
    // Java
    "class", "jar", "war", "ear",
    // .NET
    "pdb", "mdb",
    // Others
    "bin", "dat", "pak", "wad", "iso", "img", "vdi", "vmdk", "qcow2",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_binary_extension_detection() {
        let detector = BinaryDetector::default();
        
        // Test binary extensions
        assert!(detector.is_binary_by_extension(Path::new("test.exe")));
        assert!(detector.is_binary_by_extension(Path::new("test.jpg")));
        assert!(detector.is_binary_by_extension(Path::new("test.pdf")));
        assert!(detector.is_binary_by_extension(Path::new("TEST.EXE"))); // case insensitive
        
        // Test text extensions
        assert!(!detector.is_binary_by_extension(Path::new("test.txt")));
        assert!(!detector.is_binary_by_extension(Path::new("test.rs")));
        assert!(!detector.is_binary_by_extension(Path::new("test.py")));
        assert!(!detector.is_binary_by_extension(Path::new("Makefile")));
    }

    #[test]
    fn test_text_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a text file
        let text_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&text_file)?;
        writeln!(file, "This is a text file with some content.")?;
        writeln!(file, "It has multiple lines and should be detected as text.")?;
        
        assert!(detector.is_text_file(&text_file)?);
        assert!(!detector.is_binary(&text_file)?);

        Ok(())
    }

    #[test] 
    fn test_binary_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a binary file with null bytes
        let binary_file = temp_dir.path().join("test.bin");
        let mut file = File::create(&binary_file)?;
        file.write_all(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD])?;
        
        assert!(!detector.is_text_file(&binary_file)?);
        assert!(detector.is_binary(&binary_file)?);

        Ok(())
    }

    #[test]
    fn test_empty_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create an empty file
        let empty_file = temp_dir.path().join("empty.txt");
        File::create(&empty_file)?;
        
        // Empty files should be treated as text
        assert!(detector.is_text_file(&empty_file)?);
        assert!(!detector.is_binary(&empty_file)?);

        Ok(())
    }

    #[test]
    fn test_utf8_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a UTF-8 file with unicode content
        let utf8_file = temp_dir.path().join("unicode.txt");
        let mut file = File::create(&utf8_file)?;
        writeln!(file, "Hello, 世界! 🌍")?;
        writeln!(file, "This file contains émojis and unicode characters: 日本語")?;
        
        assert!(detector.is_text_file(&utf8_file)?);
        assert!(!detector.is_binary(&utf8_file)?);

        Ok(())
    }

    #[test]
    fn test_binary_reason() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Test extension-based detection
        let exe_file = temp_dir.path().join("test.exe");
        File::create(&exe_file)?;
        let reason = detector.get_binary_reason(&exe_file)?;
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("extension"));

        // Test null byte detection
        let binary_file = temp_dir.path().join("test.bin"); // Use .bin extension to force binary detection first
        let mut file = File::create(&binary_file)?;
        file.write_all(b"some text\x00more text")?;
        drop(file); // Ensure file is closed
        let reason = detector.get_binary_reason(&binary_file)?;
        assert!(reason.is_some());
        let reason_str = reason.unwrap();
        assert!(reason_str.contains("extension") || reason_str.contains("null") || reason_str.contains("binary"));

        // Test text file (should not be binary)
        let text_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&text_file)?;
        writeln!(file, "This is just text")?;
        let reason = detector.get_binary_reason(&text_file)?;
        assert!(reason.is_none());

        Ok(())
    }

    #[test]
    fn test_printable_ascii() {
        // Test printable characters
        assert!(is_printable_ascii(b' '));  // space
        assert!(is_printable_ascii(b'A'));  // letter
        assert!(is_printable_ascii(b'0'));  // digit
        assert!(is_printable_ascii(b'~'));  // tilde
        assert!(is_printable_ascii(b'\t')); // tab
        assert!(is_printable_ascii(b'\n')); // newline
        assert!(is_printable_ascii(b'\r')); // carriage return

        // Test non-printable characters
        assert!(!is_printable_ascii(0x00)); // null
        assert!(!is_printable_ascii(0x01)); // control character
        assert!(!is_printable_ascii(0x7F)); // DEL
        assert!(!is_printable_ascii(0x80)); // extended ASCII
        assert!(!is_printable_ascii(0xFF)); // extended ASCII
    }

    #[test]
    fn test_utf8_start_detection() {
        // Valid UTF-8 start bytes
        assert!(is_valid_utf8_start(0x41));  // ASCII 'A'
        assert!(is_valid_utf8_start(0xC2));  // 2-byte UTF-8 start
        assert!(is_valid_utf8_start(0xE0));  // 3-byte UTF-8 start
        assert!(is_valid_utf8_start(0xF0));  // 4-byte UTF-8 start

        // Invalid UTF-8 start bytes
        assert!(!is_valid_utf8_start(0x80)); // continuation byte
        assert!(!is_valid_utf8_start(0xBF)); // continuation byte
        assert!(!is_valid_utf8_start(0xF8)); // invalid start byte
        assert!(!is_valid_utf8_start(0xFF)); // invalid start byte
    }
}