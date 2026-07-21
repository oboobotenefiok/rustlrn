// File: src/ui.rs
//! User interface rendering module for RustLrn
//!
//! Handles all terminal output, including colored text, code blocks,
//! syntax highlighting, progress bars, and interactive elements.

use crate::config::{Config, Theme, Verbosity, CodeDisplay};
use crate::error::{Result, RustlrnError};
use crate::executor::ExecutionResult;
use colored::*;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use terminal_size::{terminal_size, Width};

/// Terminal width and margin management
#[derive(Debug, Clone)]
pub struct Terminal {
    width: usize,
    margin: usize,
}

impl Terminal {
    pub fn new() -> Self {
        let width = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(80);
        
        // Minimum width enforcement
        let width = if width < 40 { 40 } else { width };
        
        Self {
            width,
            margin: if width > 80 { 2 } else { 0 },
        }
    }
    
    /// Get the terminal width
    pub fn width(&self) -> usize {
        self.width
    }
    
    /// Get the margin string
    pub fn margin_str(&self) -> String {
        " ".repeat(self.margin)
    }
    
    /// Get the code margin string (indented slightly more)
    pub fn code_margin_str(&self) -> String {
        " ".repeat(self.margin + 2)
    }
    
    /// Get a separator line
    pub fn separator(&self, char: char) -> String {
        let width = self.width.saturating_sub(self.margin * 2);
        char.to_string().repeat(width.min(40))
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

/// Clear the terminal screen
pub fn clear_screen() {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(&["/c", "cls"])
        .status();
    
    #[cfg(not(target_os = "windows"))]
    let _ = std::process::Command::new("clear").status();
}

/// Wait for user to press Enter
pub fn wait_for_enter() {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    println!("\n{}{}", margin, "[enter] Press Enter to continue...".dimmed());
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Wait for user input with timeout
pub fn wait_for_enter_with_timeout(timeout: Duration) -> bool {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    println!("\n{}{} (waiting {}s)", margin, 
        "[enter] Press Enter to continue...".dimmed(),
        timeout.as_secs().to_string().yellow()
    );
    
    let start = std::time::Instant::now();
    let mut input = String::new();
    
    while start.elapsed() < timeout {
        if let Ok(()) = io::stdin().read_line(&mut input) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    false
}

/// Show the application header
pub fn show_header(app_name: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    
    println!("{}{}", margin, app_name.cyan().bold());
    println!("{}{}", margin, terminal.separator('-').cyan());
}

/// Show a progress bar
pub fn show_progress_bar(percent: f32, label: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    let width = terminal.width().saturating_sub(margin.len() + 20);
    let bar_width = width.min(40);
    
    let filled = (bar_width as f32 * percent / 100.0) as usize;
    let empty = bar_width.saturating_sub(filled);
    
    let bar = "█".repeat(filled) + "░".repeat(empty);
    println!("{}{} {}%", margin, bar.green(), 
        format!("{:.1}", percent).cyan()
    );
    if !label.is_empty() {
        println!("{}{}", margin, label.dimmed());
    }
}

/// Show a lesson with proper formatting
pub fn show_lesson(content: &str, config: &Config, show_line_numbers: bool) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    let code_margin = terminal.code_margin_str();
    
    let mut in_code_block = false;
    let mut code_buffer: Vec<String> = Vec::new();
    let mut block_index = 0;
    let mut lang = "rust".to_string();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            if !in_code_block {
                println!();
            }
            continue;
        }
        
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block
                if !code_buffer.is_empty() {
                    render_code_block(
                        &code_buffer, 
                        block_index + 1, 
                        &lang,
                        &terminal,
                        config,
                        show_line_numbers
                    );
                    block_index += 1;
                    code_buffer.clear();
                }
                in_code_block = false;
            } else {
                // Start of code block - check for language
                lang = trimmed.trim_start_matches("```").trim().to_string();
                if lang.is_empty() {
                    lang = "rust".to_string();
                }
                in_code_block = true;
                code_buffer.clear();
            }
            continue;
        }
        
        if in_code_block {
            // Check for language specification line
            if !trimmed.starts_with("```") && 
               !trimmed.starts_with("//") && 
               trimmed == "rust" {
                continue;
            }
            code_buffer.push(line.to_string());
            continue;
        }
        
        render_text_line(trimmed, &margin);
    }
    
    // Handle any remaining code block
    if !code_buffer.is_empty() {
        render_code_block(
            &code_buffer, 
            block_index + 1, 
            &lang,
            &terminal,
            config,
            show_line_numbers
        );
    }
}

/// Render a code block with syntax highlighting
fn render_code_block(
    code_buffer: &[String],
    block_num: usize,
    lang: &str,
    terminal: &Terminal,
    _config: &Config,
    show_line_numbers: bool,
) {
    let code_margin = terminal.code_margin_str();
    let code_text = code_buffer.join("\n");
    
    // Border
    println!("{}{}", code_margin, "┌─ Code Block ─────────────".dimmed());
    
    // Show language if specified and not default
    if lang != "rust" && !lang.is_empty() {
        println!("{}{}", code_margin, format!("│ Language: {}", lang).dimmed());
    }
    
    // Render each line with optional line numbers
    for (i, code_line) in code_buffer.iter().enumerate() {
        if show_line_numbers {
            let line_num = format!("{:>4} │", i + 1);
            println!("{}{} {}", code_margin, line_num.dimmed(), code_line.dimmed());
        } else {
            println!("{}{}", code_margin, format!("│ {}", code_line).dimmed());
        }
    }
    
    println!("{}{}", code_margin, "└─────────────────────────".dimmed());
    
    // Show block ID for interaction
    let has_main = crate::executor::has_main_function(&code_text);
    if has_main || code_text.contains("println!") {
        println!("{}{} '{}'", 
            terminal.margin_str(), 
            "[ID]".green().bold(), 
            block_num
        );
    }
}

/// Render a text line with appropriate styling
fn render_text_line(trimmed: &str, margin: &str) {
    let line_type = LineType::classify(trimmed);
    let rendered = line_type.render(trimmed, margin);
    print!("{}", rendered);
}

/// Line type classification for text rendering
#[derive(Debug, Clone, Copy)]
enum LineType {
    Heading,
    Tip,
    Note,
    Checkmark,
    Bullet,
    SectionHeader,
    CodeLine,
    Error,
    Separator,
    MarkdownH1,
    MarkdownH2,
    MarkdownH3,
    TableRow,
    Plain,
}

impl LineType {
    fn classify(trimmed: &str) -> Self {
        if trimmed.starts_with("TIP:") || trimmed.starts_with("💡") {
            LineType::Tip
        } else if trimmed.starts_with("NOTE:") || trimmed.starts_with("📌") {
            LineType::Note
        } else if trimmed.starts_with('✓') || trimmed.starts_with('✅') {
            LineType::Checkmark
        } else if trimmed.starts_with('•') || trimmed.starts_with('-') || trimmed.starts_with('*') {
            LineType::Bullet
        } else if trimmed.starts_with("Example:") ||
                  trimmed.starts_with("LOCAL VARIABLE SCOPE:") ||
                  trimmed.starts_with("STACK vs. HEAP:") ||
                  trimmed.starts_with("📘") {
            LineType::SectionHeader
        } else if trimmed.starts_with("fn main()") ||
                  trimmed.starts_with("fn status()") ||
                  trimmed.starts_with("let") ||
                  trimmed.starts_with("pub fn") ||
                  trimmed.starts_with("impl") ||
                  trimmed.starts_with("struct") ||
                  trimmed.starts_with("enum") ||
                  trimmed.starts_with("match") {
            LineType::CodeLine
        } else if trimmed.contains("ERROR!") || trimmed.contains("⚠️") {
            LineType::Error
        } else if trimmed.starts_with("===") || trimmed.starts_with("━━━") || 
                  trimmed.starts_with("---") {
            LineType::Separator
        } else if trimmed.starts_with("# ") && !trimmed.starts_with("##") {
            LineType::MarkdownH1
        } else if trimmed.starts_with("## ") {
            LineType::MarkdownH2
        } else if trimmed.starts_with("### ") {
            LineType::MarkdownH3
        } else if trimmed.starts_with('|') && trimmed.contains('|') {
            LineType::TableRow
        } else if is_heading(trimmed) {
            LineType::Heading
        } else {
            LineType::Plain
        }
    }
    
    fn render(self, trimmed: &str, margin: &str) -> String {
        match self {
            LineType::Heading => format!("{}{}\n", margin, trimmed.yellow().bold()),
            LineType::Tip => format!("{}{}", margin, trimmed.green().bold()),
            LineType::Note => format!("{}{}", margin, trimmed.blue().bold()),
            LineType::Checkmark => format!("{}{}", margin, format!("  {}", trimmed).green()),
            LineType::Bullet => format!("{}{}", margin, format!("  {}", trimmed).cyan()),
            LineType::SectionHeader => format!("\n{}{}\n", margin, trimmed.cyan().bold()),
            LineType::CodeLine => format!("{}{}", margin, format!("  {}", trimmed).dimmed()),
            LineType::Error => format!("{}{}", margin, format!("  {}", trimmed).red().bold()),
            LineType::Separator => format!("{}{}\n", margin, trimmed.dimmed()),
            LineType::MarkdownH1 => format!("\n{}{}\n", margin, trimmed[1..].trim().cyan().bold().underline()),
            LineType::MarkdownH2 => format!("\n{}{}\n", margin, trimmed[2..].trim().yellow().bold()),
            LineType::MarkdownH3 => format!("{}{}\n", margin, trimmed[3..].trim().white().bold()),
            LineType::TableRow => format!("{}{}\n", margin, trimmed.dimmed()),
            LineType::Plain => format!("{}{}\n", margin, trimmed),
        }
    }
}

/// Check if text is a heading
fn is_heading(text: &str) -> bool {
    text.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c == ':' || c == '•' || c == '━' || c == '═')
        && text.len() > 3
        && !text.starts_with("TIP")
        && !text.starts_with("NOTE")
        && !text.starts_with("Based on")
        && !text.starts_with("*Credit")
        && !text.starts_with('|')
        && !text.starts_with("```")
}

/// Show controls with proper formatting
pub fn show_controls(config: &Config) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    
    println!();
    println!("{}{}", margin, terminal.separator('-').dimmed());
    println!(
        "{}{} {}  {} {}  {} {}  {} {}  {} {}  {} {}",
        margin,
        "[n]".green().bold(),
        "next".dimmed(),
        "[p]".yellow().bold(),
        "previous".dimmed(),
        "[r#]".magenta().bold(),
        "run".dimmed(),
        "[ed#]".cyan().bold(),
        "edit".dimmed(),
        "[z#]".red().bold(),
        "reset".dimmed(),
        "[q]".red().bold(),
        "quit".dimmed()
    );
    
    if !config.is_editor_configured() {
        println!(
            "{}{} No editor set. Run: rustlrn editor <command>",
            margin,
            "[!]".yellow().bold()
        );
    }
    
    // Show auto-run status if enabled
    match config.auto_run {
        crate::config::AutoRun::Always => {
            println!("{}{} Auto-run: enabled", margin, "[⏳]".green().bold());
        }
        crate::config::AutoRun::OnEdit => {
            println!("{}{} Auto-run: on edit", margin, "[⏳]".cyan().bold());
        }
        _ => {}
    }
    
    println!();
}

/// Show execution result with proper formatting
pub fn show_execution_result(result: &ExecutionResult, config: &Config) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    
    println!();
    println!("{}{}", margin, "═".repeat(40).cyan());
    println!("{}{}", margin, "▶ EXECUTION RESULT".cyan().bold());
    println!("{}{}", margin, "─".repeat(40).dimmed());
    
    if result.success {
        println!("{}{}", margin, "✓ SUCCESS".green().bold());
        if result.output.is_empty() {
            println!("{}{}", margin, "  (no output)".dimmed());
        } else {
            println!("\n{}{}", margin, "Output:".dimmed());
            for line in result.output.lines() {
                println!("{}{}", margin, format!("  {}", line));
            }
        }
    } else {
        println!("{}{}", margin, "✗ ERROR".red().bold());
        if !result.error.is_empty() {
            println!("\n{}{}", margin, "Compiler/Runtime Error:".red().dimmed());
            for line in result.error.lines() {
                println!("{}{}", margin, format!("  {}", line).red());
            }
        }
    }
    
    // Show timing information
    if config.show_execution_time {
        println!("\n{}{}", margin, "─".repeat(40).dimmed());
        println!("{}{} Compilation: {}  Execution: {}", 
            margin,
            "[⏱]".dimmed(),
            format!("{:.2}ms", result.compilation_time.as_secs_f64() * 1000.0).yellow(),
            format!("{:.2}ms", result.execution_time.as_secs_f64() * 1000.0).yellow()
        );
    }
    
    println!("{}{}", margin, "═".repeat(40).cyan());
    wait_for_enter();
}

/// Show an error message
pub fn show_error(msg: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    eprintln!("{}{}", margin, format!("✗ {}", msg).red().bold());
}

/// Show a warning message
pub fn show_warning(msg: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    println!("{}{}", margin, format!("⚠ {}", msg).yellow().bold());
}

/// Show an info message
pub fn show_info(msg: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    println!("{}{}", margin, format!("ℹ {}", msg).cyan().bold());
}

/// Show a success message
pub fn show_success(msg: &str) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    println!("{}{}", margin, format!("✓ {}", msg).green().bold());
}

/// Show help text
pub fn show_help(config: &Config) {
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    
    println!("\n{}{}", margin, "[help] Commands:".yellow().bold());
    println!("  {} r1, r2, r3    - Run specific code block", "[r#]".cyan().bold());
    println!("  {} ed1, ed2, ed3 - Edit specific code block", "[ed#]".green().bold());
    println!("  {} ed0           - Write code from scratch", "[ed0]".green().bold());
    println!("  {} z1, z2, z3    - Reset specific code block", "[z#]".red().bold());
    println!("  {} n             - Next lesson", "[n]".green().bold());
    println!("  {} p             - Previous lesson", "[p]".yellow().bold());
    println!("  {} q             - Quit", "[q]".red().bold());
    println!("  {} h             - Show this help", "[h]".cyan().bold());
    
    // Show editor info
    if let Some(editor) = config.get_editor() {
        println!("\n{}{} Editor: {}", margin, "[config]".cyan().bold(), editor.cyan());
    } else {
        println!("\n{}{} No editor configured!", margin, "[warn]".yellow().bold());
        println!("  {} Set editor: rustlrn editor <command>", "[setup]".cyan().bold());
        println!("  {} Example: rustlrn editor nano", "[example]".dimmed());
    }
    
    // Show version info
    println!("\n{}{} Version: {}", margin, "[info]".dimmed(), env!("CARGO_PKG_VERSION").dimmed());
}

/// Edit code with configured editor
pub fn edit_code_with_editor(
    code: &str,
    block_num: usize,
    config: &Config,
) -> Result<String> {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;
    
    let editor_cmd = config
        .editor
        .as_ref()
        .ok_or_else(|| RustlrnError::Editor(
            "No editor configured!\n\n\
             Please set your editor with: rustlrn editor <command>\n\
             Examples:\n\
               rustlrn editor nano\n\
               rustlrn editor micro\n\
               rustlrn editor vim\n\
               rustlrn editor \"code --wait\"\n\
               rustlrn editor \"subl -w\"".to_string()
        ))?;
    
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| RustlrnError::Editor(format!("Failed to create temp file: {}", e)))?;
    
    if !code.is_empty() {
        write!(temp_file, "{}", code)
            .map_err(|e| RustlrnError::Editor(format!("Failed to write code: {}", e)))?;
        temp_file.flush()
            .map_err(|e| RustlrnError::Editor(format!("Failed to flush temp file: {}", e)))?;
    }
    
    let temp_path = temp_file.path();
    let temp_path_str = temp_path.to_str()
        .ok_or_else(|| RustlrnError::Editor("Invalid temp file path".to_string()))?;
    
    let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    let (editor_cmd_base, editor_args) = cmd_parts.split_first()
        .ok_or_else(|| RustlrnError::Editor(format!("Empty editor command: '{}'", editor_cmd)))?;
    
    let terminal = Terminal::new();
    let margin = terminal.margin_str();
    
    let block_label = if block_num > 0 {
        format!(" block #{}", block_num)
    } else {
        " (new code)".to_string()
    };
    
    println!(
        "{}{} Opening editor{}... Edit, save, and close when done.",
        margin,
        "[hint]".yellow().bold(),
        block_label
    );
    
    // Execute editor
    let status = Command::new(editor_cmd_base)
        .args(editor_args)
        .arg(temp_path_str)
        .status()
        .map_err(|e| RustlrnError::Editor(format!("Failed to open editor: {}", e)))?;
    
    if !status.success() {
        return Err(RustlrnError::Editor("Editor exited with error".to_string()));
    }
    
    // Read the edited code
    let edited_code = fs::read_to_string(temp_path)
        .map_err(|e| RustlrnError::Editor(format!("Failed to read edited code: {}", e)))?;
    
    if edited_code.trim().is_empty() {
        return Err(RustlrnError::Cancelled);
    }
    
    Ok(edited_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terminal_creation() {
        let terminal = Terminal::new();
        assert!(terminal.width() >= 40);
        assert!(terminal.margin() >= 0);
    }
    
    #[test]
    fn test_separator_generation() {
        let terminal = Terminal::new();
        let sep = terminal.separator('-');
        assert!(!sep.is_empty());
        assert!(sep.len() <= 40);
    }
    
    #[test]
    fn test_heading_detection() {
        assert!(is_heading("INTRODUCTION"));
        assert!(is_heading("CHAPTER 1:"));
        assert!(!is_heading("TIP: Something"));
        assert!(!is_heading("NOTE: Something"));
        assert!(!is_heading("```rust"));
    }
}
