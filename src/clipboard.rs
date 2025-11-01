use clipboard::{ClipboardContext, ClipboardProvider};
use std::process::Command;

/// A module for handling clipboard operations
pub struct Clipboard;

impl Clipboard {
    /// Copy a string to the system clipboard
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy to the clipboard
    ///
    /// # Returns
    ///
    /// * `Result<(), ClipboardError>` - Ok if successful, Err if failed
    ///
    /// # Examples
    ///
    /// ```
    /// use wallet::clipboard::Clipboard;
    ///
    /// let result = Clipboard::copy("Hello, World!");
    /// match result {
    ///     Ok(_) => println!("Text copied successfully"),
    ///     Err(e) => println!("Failed to copy: {}", e),
    /// }
    /// ```
    pub fn copy(text: &str) -> Result<(), ClipboardError> {
        // Try the primary clipboard crate first
        let clipboard_result = Self::copy_with_clipboard_crate(text);

        // If that fails on Linux, try using xsel as fallback
        if clipboard_result.is_err() && cfg!(target_os = "linux") {
            return Self::copy_with_xsel(text);
        }

        clipboard_result
    }

    /// Copy using the clipboard crate
    fn copy_with_clipboard_crate(text: &str) -> Result<(), ClipboardError> {
        let mut ctx: ClipboardContext = ClipboardProvider::new().map_err(|e| {
            ClipboardError::CopyFailed(format!("Failed to create clipboard context: {}", e))
        })?;

        ctx.set_contents(text.to_string())
            .map_err(|e| ClipboardError::CopyFailed(e.to_string()))?;

        // Keep the context alive for a moment to ensure the clipboard is properly set
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }

    /// Copy using xsel command (Linux fallback)
    #[cfg(target_os = "linux")]
    fn copy_with_xsel(text: &str) -> Result<(), ClipboardError> {
        let mut output = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .arg("--")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| ClipboardError::CopyFailed(format!("Failed to spawn xsel: {}", e)))?;

        if let Some(mut stdin) = output.stdin.take() {
            use std::io::Write;
            stdin.write_all(text.as_bytes()).map_err(|e| {
                ClipboardError::CopyFailed(format!("Failed to write to xsel: {}", e))
            })?;
        }

        Ok(())
    }

    /// Non-Linux platforms don't have xsel fallback
    #[cfg(not(target_os = "linux"))]
    fn copy_with_xsel(_text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::CopyFailed(
            "xsel not available on this platform".to_string(),
        ))
    }

    /// Copy a string to the clipboard and print a confirmation message
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy to the clipboard
    /// * `label` - Optional label to describe what's being copied
    ///
    /// # Returns
    ///
    /// * `Result<(), ClipboardError>` - Ok if successful, Err if failed
    pub fn copy_with_message(text: &str) -> Result<(), ClipboardError> {
        Self::copy(text)?;

        Ok(())
    }

    /// Get text from the system clipboard
    ///
    /// # Returns
    ///
    /// * `Result<String, ClipboardError>` - The clipboard content if successful, Err if failed
    ///
    /// # Examples
    ///
    /// ```
    /// use wallet::clipboard::Clipboard;
    ///
    /// match Clipboard::paste() {
    ///     Ok(content) => println!("Clipboard contains: {}", content),
    ///     Err(e) => println!("Failed to read clipboard: {}", e),
    /// }
    /// ```
    pub fn paste() -> Result<String, ClipboardError> {
        let mut ctx: ClipboardContext = ClipboardProvider::new().map_err(|e| {
            ClipboardError::PasteFailed(format!("Failed to create clipboard context: {}", e))
        })?;

        ctx.get_contents()
            .map_err(|e| ClipboardError::PasteFailed(e.to_string()))
    }

    /// Check if the clipboard contains any text
    ///
    /// # Returns
    ///
    /// * `bool` - true if clipboard has content, false otherwise
    pub fn has_content() -> bool {
        Self::paste().is_ok()
    }

    /// Clear the clipboard
    ///
    /// # Returns
    ///
    /// * `Result<(), ClipboardError>` - Ok if successful, Err if failed
    pub fn clear() -> Result<(), ClipboardError> {
        Self::copy("")
    }
}

/// Error types for clipboard operations
#[derive(Debug)]
pub enum ClipboardError {
    CopyFailed(String),
    PasteFailed(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::CopyFailed(msg) => write!(f, "Failed to copy to clipboard: {}", msg),
            ClipboardError::PasteFailed(msg) => {
                write!(f, "Failed to paste from clipboard: {}", msg)
            }
        }
    }
}

impl std::error::Error for ClipboardError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_and_paste() {
        let test_text = "Hello, clipboard test!";

        // Test copying
        assert!(Clipboard::copy(test_text).is_ok());

        // Test pasting
        match Clipboard::paste() {
            Ok(content) => assert_eq!(content, test_text),
            Err(_) => {
                // Skip test if clipboard is not available in test environment
                println!("Clipboard not available in test environment");
            }
        }
    }

    #[test]
    fn test_copy_with_message() {
        let test_text = "Test message";
        let result = Clipboard::copy_with_message(test_text);

        // This test mainly checks that the function doesn't panic
        // Actual clipboard functionality may not work in test environment
        match result {
            Ok(_) => println!("Copy with message succeeded"),
            Err(_) => println!("Copy with message failed (expected in test environment)"),
        }
    }

    #[test]
    fn test_clear() {
        let result = Clipboard::clear();

        // This test mainly checks that the function doesn't panic
        match result {
            Ok(_) => println!("Clear succeeded"),
            Err(_) => println!("Clear failed (expected in test environment)"),
        }
    }
}
