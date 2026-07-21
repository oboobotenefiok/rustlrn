// File: src/config.rs
//! Configuration management for RustLrn
//!
//! Handles loading and saving user preferences including editor choice,
//! theme settings, auto-run preferences, and execution options.

use crate::error::{ErrorContext, Result, RustlrnError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::Duration;
use which::which;

/// Application theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "auto")]
    Auto,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

/// Output verbosity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Verbosity {
    #[serde(rename = "quiet")]
    Quiet,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "verbose")]
    Verbose,
    #[serde(rename = "debug")]
    Debug,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Normal
    }
}

/// Auto-run behavior for code blocks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoRun {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "on-edit")]
    OnEdit,
    #[serde(rename = "on-navigation")]
    OnNavigation,
    #[serde(rename = "always")]
    Always,
}

impl Default for AutoRun {
    fn default() -> Self {
        AutoRun::Never
    }
}

/// Code block display format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodeDisplay {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "minimal")]
    Minimal,
}

impl Default for CodeDisplay {
    fn default() -> Self {
        CodeDisplay::Full
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Editor command to use for editing code blocks
    pub editor: Option<String>,
    
    /// Theme preference
    #[serde(default)]
    pub theme: Theme,
    
    /// Verbosity level
    #[serde(default)]
    pub verbosity: Verbosity,
    
    /// Auto-run behavior
    #[serde(default)]
    pub auto_run: AutoRun,
    
    /// Code block display format
    #[serde(default)]
    pub code_display: CodeDisplay,
    
    /// Whether to show line numbers in code blocks
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    
    /// Whether to show execution time
    #[serde(default = "default_true")]
    pub show_execution_time: bool,
    
    /// Maximum number of retries for failed execution
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    
    /// Compilation timeout in seconds
    #[serde(default = "default_timeout")]
    pub compile_timeout: u64,
    
    /// Execution timeout in seconds
    #[serde(default = "default_timeout")]
    pub run_timeout: u64,
    
    /// User-specific settings
    #[serde(default)]
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: None,
            theme: Theme::default(),
            verbosity: Verbosity::default(),
            auto_run: AutoRun::default(),
            code_display: CodeDisplay::default(),
            show_line_numbers: true,
            show_execution_time: true,
            max_retries: 3,
            compile_timeout: 30,
            run_timeout: 10,
            custom_settings: HashMap::new(),
        }
    }
}

// Default helper functions
fn default_true() -> bool { true }
fn default_max_retries() -> usize { 3 }
fn default_timeout() -> u64 { 30 }

impl Config {
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".rustlrn").join("config.toml")
    }
    
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        
        if !path.exists() {
            // Return default config
            return Ok(Config::default());
        }
        
        let content = fs::read_to_string(&path)
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to read config file")?;
        
        if content.is_empty() {
            return Ok(Config::default());
        }
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| RustlrnError::Config(format!("Invalid config file: {}", e)))?;
        
        // Validate the loaded config
        config.validate()?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        // Validate before saving
        self.validate()?;
        
        let path = Self::config_path();
        
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| RustlrnError::Io(e))
                    .with_context("Failed to create config directory")?;
            }
        }
        
        let content = toml::to_string(self)
            .map_err(|e| RustlrnError::Toml(e))
            .with_context("Failed to serialize config")?;
        
        fs::write(&path, content)
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to write config file")?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate editor if set
        if let Some(editor) = &self.editor {
            if !Self::validate_editor(editor) {
                return Err(RustlrnError::Config(format!(
                    "Editor '{}' not found in PATH",
                    editor
                )));
            }
        }
        
        // Validate timeout values
        if self.compile_timeout < 1 || self.compile_timeout > 300 {
            return Err(RustlrnError::Config(
                "Compile timeout must be between 1 and 300 seconds".to_string()
            ));
        }
        
        if self.run_timeout < 1 || self.run_timeout > 300 {
            return Err(RustlrnError::Config(
                "Run timeout must be between 1 and 300 seconds".to_string()
            ));
        }
        
        if self.max_retries > 10 {
            return Err(RustlrnError::Config(
                "Max retries must be between 0 and 10".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate that an editor command exists
    pub fn validate_editor(editor_cmd: &str) -> bool {
        let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
        if cmd_parts.is_empty() {
            return false;
        }
        
        let base_cmd = cmd_parts[0];
        which(base_cmd).is_ok()
    }
    
    /// Check if editor command blocks (for GUI editors)
    pub fn is_blocking_editor(editor_cmd: &str) -> bool {
        let non_blocking = ["code", "subl", "atom", "notepad", "notepad++"];
        let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
        
        if cmd_parts.is_empty() {
            return false;
        }
        
        let base_cmd = cmd_parts[0];
        
        // Check if it's a known GUI editor without wait flag
        if non_blocking.contains(&base_cmd) &&
           !editor_cmd.contains("--wait") &&
           !editor_cmd.contains("-w") &&
           !editor_cmd.contains("--blocking") {
            return false;
        }
        
        true
    }
    
    /// Get the configured editor command
    pub fn get_editor(&self) -> Option<&str> {
        self.editor.as_deref()
    }
    
    /// Check if editor is configured
    pub fn is_editor_configured(&self) -> bool {
        self.editor.is_some()
    }
    
    /// Set the editor command
    pub fn set_editor(&mut self, command: String) -> Result<()> {
        if !Self::validate_editor(&command) {
            let cmd_parts: Vec<&str> = command.split_whitespace().collect();
            let base_cmd = cmd_parts.first().unwrap_or(&"");
            return Err(RustlrnError::Config(format!(
                "Editor '{}' not found in PATH",
                base_cmd
            )));
        }
        
        self.editor = Some(command);
        Ok(())
    }
    
    /// Get a custom setting
    pub fn get_custom<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.custom_settings
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    
    /// Set a custom setting
    pub fn set_custom<T: serde::Serialize>(&mut self, key: String, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| RustlrnError::Config(format!("Invalid setting value: {}", e)))?;
        
        self.custom_settings.insert(key, json_value);
        Ok(())
    }
    
    /// Get the configured editor with default fallback
    pub fn get_editor_with_fallback(&self, fallback: &str) -> String {
        self.editor.clone().unwrap_or_else(|| fallback.to_string())
    }
}

/// Configuration builder for programmatic configuration
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    pub fn editor(mut self, editor: String) -> Self {
        self.config.editor = Some(editor);
        self
    }
    
    pub fn theme(mut self, theme: Theme) -> Self {
        self.config.theme = theme;
        self
    }
    
    pub fn verbosity(mut self, verbosity: Verbosity) -> Self {
        self.config.verbosity = verbosity;
        self
    }
    
    pub fn auto_run(mut self, auto_run: AutoRun) -> Self {
        self.config.auto_run = auto_run;
        self
    }
    
    pub fn code_display(mut self, code_display: CodeDisplay) -> Self {
        self.config.code_display = code_display;
        self
    }
    
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.config.show_line_numbers = show;
        self
    }
    
    pub fn show_execution_time(mut self, show: bool) -> Self {
        self.config.show_execution_time = show;
        self
    }
    
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.config.max_retries = max_retries;
        self
    }
    
    pub fn compile_timeout(mut self, timeout: u64) -> Self {
        self.config.compile_timeout = timeout;
        self
    }
    
    pub fn run_timeout(mut self, timeout: u64) -> Self {
        self.config.run_timeout = timeout;
        self
    }
    
    pub fn build(self) -> Result<Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Load configuration with error handling
pub fn load_config() -> Config {
    Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Could not load config: {}", e);
        Config::default()
    })
}

/// Save configuration with error handling
pub fn save_config(config: &Config) -> bool {
    config.save().is_ok()
}

/// Check if editor is configured using default loading
pub fn is_editor_configured() -> bool {
    let config = load_config();
    config.is_editor_configured()
}

/// Get the configured editor command using default loading
pub fn get_editor() -> Option<String> {
    let config = load_config();
    config.editor
}

/// Validate an editor command using default validation
pub fn validate_editor(editor_cmd: &str) -> bool {
    Config::validate_editor(editor_cmd)
}

/// Check if editor blocks using default validation
pub fn is_blocking_editor(editor_cmd: &str) -> bool {
    Config::is_blocking_editor(editor_cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.editor.is_none());
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.verbosity, Verbosity::Normal);
        assert_eq!(config.auto_run, AutoRun::Never);
        assert_eq!(config.code_display, CodeDisplay::Full);
        assert!(config.show_line_numbers);
        assert!(config.show_execution_time);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.compile_timeout, 30);
        assert_eq!(config.run_timeout, 10);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Invalid timeout
        config.compile_timeout = 0;
        assert!(config.validate().is_err());
        
        config.compile_timeout = 30;
        config.run_timeout = 0;
        assert!(config.validate().is_err());
        
        // Valid config
        config.run_timeout = 10;
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .editor("nano".to_string())
            .theme(Theme::Auto)
            .verbosity(Verbosity::Verbose)
            .auto_run(AutoRun::OnEdit)
            .max_retries(5)
            .build()
            .unwrap();
        
        assert_eq!(config.editor, Some("nano".to_string()));
        assert_eq!(config.theme, Theme::Auto);
        assert_eq!(config.verbosity, Verbosity::Verbose);
        assert_eq!(config.auto_run, AutoRun::OnEdit);
        assert_eq!(config.max_retries, 5);
    }
    
    #[test]
    fn test_editor_validation() {
        // This test may fail depending on system, but we can test the logic
        // For actual validation, we'd need to mock which
        let cmd = "rustc"; // Should be available
        let result = Config::validate_editor(cmd);
        assert!(result);
    }
    
    #[test]
    fn test_custom_settings() {
        let mut config = Config::default();
        
        // Set custom setting
        config.set_custom("test_key".to_string(), 42).unwrap();
        assert_eq!(config.get_custom::<i32>("test_key"), Some(42));
        
        // Get non-existent setting
        assert_eq!(config.get_custom::<i32>("non_existent"), None);
    }
    
    #[test]
    fn test_config_save_load() {
        let temp_dir = tempdir().unwrap();
        let original_path = Config::config_path();
        
        // We can't easily override the path, but we can test the functionality
        // For now, just test the save/load logic without filesystem
        let config = Config::default();
        
        // Test serialization
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config.editor, deserialized.editor);
        assert_eq!(config.theme, deserialized.theme);
        assert_eq!(config.verbosity, deserialized.verbosity);
    }
}
