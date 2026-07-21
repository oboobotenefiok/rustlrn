// File: src/error.rs
//! Centralized error handling for the RustLrn application
//!
//! This module provides a unified error type that wraps all possible
//! errors that can occur in the application, with proper context
//! and error propagation.

use std::io;
use std::path::PathBuf;
use std::process;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Main error type for the RustLrn application
#[derive(Error, Debug)]
pub enum RustlrnError {
    /// I/O errors (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Configuration file errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Editor-related errors
    #[error("Editor error: {0}")]
    Editor(String),

    /// Code execution errors
    #[error("Code execution error: {0}")]
    Execution(String),

    /// Code compilation errors
    #[error("Compilation error: {0}")]
    Compilation(String),

    /// Lesson loading errors
    #[error("Lesson loading error: {0}")]
    Lesson(String),

    /// Parse errors (for command parsing, etc.)
    #[error("Parse error: {0}")]
    Parse(String),

    /// User cancellation
    #[error("Operation cancelled by user")]
    Cancelled,

    /// Invalid state transitions
    #[error("Invalid state: {0}")]
    State(String),

    /// UTF-8 conversion errors
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// TOML serialization/deserialization errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),

    /// TOML deserialization errors
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// Anyhow error wrapper
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Type alias for Result with RustlrnError
pub type Result<T> = std::result::Result<T, RustlrnError>;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<C: Into<String>>(self, context: C) -> Result<T>;

    /// Add context for editor operations
    fn editor_context<C: Into<String>>(self, context: C) -> Result<T>;

    /// Add context for configuration operations
    fn config_context<C: Into<String>>(self, context: C) -> Result<T>;
}

impl<T> ErrorContext<T> for std::result::Result<T, io::Error> {
    fn with_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| RustlrnError::Io(e).with_context(context))
    }

    fn editor_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| RustlrnError::Editor(e.to_string()))
            .with_context(context)
    }

    fn config_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| RustlrnError::Config(e.to_string()))
            .with_context(context)
    }
}

impl<T> ErrorContext<T> for std::result::Result<T, RustlrnError> {
    fn with_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| {
            let ctx = context.into();
            match e {
                RustlrnError::Io(io_err) => {
                    RustlrnError::Io(io::Error::new(io_err.kind(), format!("{}: {}", ctx, io_err)))
                }
                _ => e, // Keep original error for non-I/O cases
            }
        })
    }

    fn editor_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(context)
    }

    fn config_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(context)
    }
}

/// Convert process::Output errors to RustlrnError
impl From<process::Output> for RustlrnError {
    fn from(output: process::Output) -> Self {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if output.status.success() {
            RustlrnError::Execution(format!("Process failed with: {}", stderr))
        } else {
            RustlrnError::Compilation(format!(
                "Compilation failed:\nStdout: {}\nStderr: {}",
                stdout, stderr
            ))
        }
    }
}

/// Extension for path operations
pub trait PathErrorExt {
    /// Convert path to string with error context
    fn path_string(&self) -> Result<String>;
}

impl PathErrorExt for PathBuf {
    fn path_string(&self) -> Result<String> {
        self.to_str()
            .ok_or_else(|| RustlrnError::Config(format!("Invalid path: {:?}", self)))
            .map(String::from)
    }
}

impl PathErrorExt for std::path::Path {
    fn path_string(&self) -> Result<String> {
        self.to_str()
            .ok_or_else(|| RustlrnError::Config(format!("Invalid path: {:?}", self)))
            .map(String::from)
    }
}

/// Error handling utilities for the UI
pub mod ui_errors {
    use super::*;

    /// Display a user-friendly error message
    pub fn display_error(error: &RustlrnError) -> String {
        match error {
            RustlrnError::Editor(msg) => format!("Editor error: {}", msg),
            RustlrnError::Config(msg) => format!("Configuration error: {}", msg),
            RustlrnError::Execution(msg) => format!("Execution error: {}", msg),
            RustlrnError::Compilation(msg) => format!("Compilation error:\n{}", msg),
            RustlrnError::Io(err) => format!("I/O error: {}", err),
            RustlrnError::Lesson(msg) => format!("Lesson error: {}", msg),
            RustlrnError::Parse(msg) => format!("Parse error: {}", msg),
            RustlrnError::State(msg) => format!("State error: {}", msg),
            RustlrnError::Cancelled => "Operation cancelled".to_string(),
            RustlrnError::Utf8(err) => format!("Invalid UTF-8: {}", err),
            RustlrnError::Toml(err) => format!("TOML serialization error: {}", err),
            RustlrnError::TomlDe(err) => format!("TOML deserialization error: {}", err),
            RustlrnError::Anyhow(err) => format!("Error: {}", err),
        }
    }

    /// Check if an error is recoverable
    pub fn is_recoverable(error: &RustlrnError) -> bool {
        matches!(error, 
            RustlrnError::Editor(_) | 
            RustlrnError::Config(_) | 
            RustlrnError::Parse(_) | 
            RustlrnError::State(_)
        )
    }

    /// Suggest recovery actions for common errors
    pub fn suggest_recovery(error: &RustlrnError) -> Option<String> {
        match error {
            RustlrnError::Editor(_) => {
                Some("Run 'rustlrn editor <command>' to configure your editor".to_string())
            }
            RustlrnError::Config(msg) if msg.contains("editor") => {
                Some("Run 'rustlrn editor <command>' to set an editor".to_string())
            }
            RustlrnError::Compilation(msg) if msg.contains("rustc") => {
                Some("Make sure Rust is installed and 'rustc' is in your PATH".to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RustlrnError::Editor("Failed to open editor".to_string());
        assert!(matches!(err, RustlrnError::Editor(_)));
        assert_eq!(err.to_string(), "Editor error: Failed to open editor");
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(RustlrnError::Editor("Original error".to_string()))
            .with_context("Additional context");
        
        if let Err(e) = result {
            assert!(matches!(e, RustlrnError::Editor(msg) if msg.contains("Original error")));
        } else {
            panic!("Expected error");
        }
    }
}
