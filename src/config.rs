//! Configuration management for rustlrn
//!
//! Handles loading and saving user preferences like editor choice.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub editor: Option<String>,  // No default - user must set it
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: None,
        }
    }
}

/// Get the config file path
pub fn config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    let mut path = home;
    path.push(".rustlrn");
    path.push("config.toml");
    path
}

/// Load configuration from file
pub fn load_config() -> Config {
    let path = config_path();
    
    if !path.exists() {
        // Return default config with no editor set
        return Config::default();
    }

    let content = fs::read_to_string(&path).unwrap_or_else(|_| {
        eprintln!("Warning: Could not read config file. Using defaults.");
        return String::new();
    });

    if content.is_empty() {
        return Config::default();
    }

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Invalid config file: {}", e);
            eprintln!("Using default configuration.");
            Config::default()
        }
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) {
    let path = config_path();
    
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Could not create config directory");
        }
    }

    let content = toml::to_string(config).expect("Could not serialize config");
    fs::write(&path, content).expect("Could not write config file");
}

/// Validate that an editor command exists
pub fn validate_editor(editor_cmd: &str) -> bool {
    use std::process::Command;
    
    // Extract the base command (first word) for validation
    let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return false;
    }
    
    let base_cmd = cmd_parts[0];
    
    // Try to find the command
    if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(base_cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("which")
            .arg(base_cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Check if editor command blocks (for GUI editors)
pub fn is_blocking_editor(editor_cmd: &str) -> bool {
    // Non-blocking GUI editors usually don't have --wait or similar flags
    let non_blocking = ["code", "subl", "atom", "notepad"];
    let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    
    if cmd_parts.is_empty() {
        return false;
    }
    
    let base_cmd = cmd_parts[0];
    
    // Check if it's a known GUI editor without wait flag
    if non_blocking.contains(&base_cmd) && !editor_cmd.contains("--wait") && !editor_cmd.contains("-w") {
        return false;
    }
    
    true
}

/// Check if editor is configured
pub fn is_editor_configured() -> bool {
    let config = load_config();
    config.editor.is_some()
}

/// Get the configured editor command
pub fn get_editor() -> Option<String> {
    let config = load_config();
    config.editor
}
