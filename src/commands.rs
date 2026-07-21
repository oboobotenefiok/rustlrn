// File: src/commands.rs
//! Command handling and parsing for the RustLrn application
//!
//! This module provides a command handler that parses user input
//! and executes the appropriate actions on the application state.

use crate::error::{Result, RustlrnError};
use crate::state::{AppState, LessonId, BlockId};
use crate::ui;
use crate::executor::{self, ExecutionConfig};
use crate::config::Config;

/// Result of a command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResult {
    /// Continue the main loop
    Continue,
    /// Quit the application
    Quit,
    /// Navigation occurred
    Navigated,
    /// Code was executed
    Executed,
    /// Code was edited
    Edited,
    /// Code was reset
    Reset,
    /// Help was shown
    Help,
}

/// Command handler that processes user input
pub struct CommandHandler<'a> {
    state: &'a mut AppState,
    lessons: &'a [String],
    config: &'a Config,
}

impl<'a> CommandHandler<'a> {
    /// Create a new command handler
    pub fn new(state: &'a mut AppState, lessons: &'a [String]) -> Self {
        let config = state.config();
        Self {
            state,
            lessons,
            config,
        }
    }
    
    /// Handle a command string
    pub fn handle(&mut self, input: &str) -> Result<CommandResult> {
        let input = input.trim();
        
        if input.is_empty() {
            return Ok(CommandResult::Continue);
        }
        
        // Parse the command
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];
        
        match cmd {
            "n" | "next" => self.cmd_next(),
            "p" | "prev" | "previous" => self.cmd_previous(),
            "q" | "quit" | "exit" => self.cmd_quit(),
            "h" | "help" | "?" => self.cmd_help(),
            
            // Run commands: r, run, r1, r2, etc.
            cmd if cmd.starts_with('r') && cmd.len() > 1 => {
                self.cmd_run_block(&cmd[1..])
            }
            "r" | "run" if parts.len() > 1 => {
                self.cmd_run_block(parts[1])
            }
            "r" | "run" => self.cmd_run_current(),
            
            // Edit commands: ed, ed1, ed2, ed0, etc.
            cmd if cmd.starts_with("ed") && cmd.len() > 2 => {
                self.cmd_edit_block(&cmd[2..])
            }
            "ed" if parts.len() > 1 => {
                self.cmd_edit_block(parts[1])
            }
            "ed" => self.cmd_edit_last(),
            
            // Reset commands: z, z1, z2, etc.
            cmd if cmd.starts_with('z') && cmd.len() > 1 => {
                self.cmd_reset_block(&cmd[1..])
            }
            "z" | "reset" if parts.len() > 1 => {
                self.cmd_reset_block(parts[1])
            }
            "z" | "reset" => self.cmd_reset_current(),
            
            // Complete command (mark lesson as complete)
            "c" | "complete" | "done" => self.cmd_complete(),
            
            // Go to specific lesson
            cmd if cmd.starts_with('g') && cmd.len() > 1 => {
                self.cmd_goto(&cmd[1..])
            }
            "g" | "goto" if parts.len() > 1 => {
                self.cmd_goto(parts[1])
            }
            
            // Status command
            "s" | "status" | "stats" => self.cmd_status(),
            
            // Editor shortcut
            "e" | "edit" if parts.len() > 1 => {
                self.cmd_edit_block(parts[1])
            }
            
            _ => {
                // Check if it's a number (go to lesson)
                if let Ok(num) = input.parse::<usize>() {
                    return self.cmd_goto(&num.to_string());
                }
                
                Err(RustlrnError::Parse(format!("Unknown command: '{}'", input)))
            }
        }
    }
    
    /// Navigate to the next lesson
    fn cmd_next(&mut self) -> Result<CommandResult> {
        match self.state.navigate_next() {
            Ok(true) => {
                let terminal = ui::Terminal::new();
                let margin = terminal.margin_str();
                let current = self.state.current_lesson().as_usize() + 1;
                println!("{}{} Lesson {} of {}", 
                    margin,
                    "[navigate]".green().bold(),
                    current,
                    self.lessons.len()
                );
                ui::wait_for_enter();
                Ok(CommandResult::Navigated)
            }
            Err(e) => {
                ui::show_error(&e.to_string());
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
            _ => {
                ui::show_error("Already at the last lesson");
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
        }
    }
    
    /// Navigate to the previous lesson
    fn cmd_previous(&mut self) -> Result<CommandResult> {
        match self.state.navigate_previous() {
            Ok(true) => {
                let terminal = ui::Terminal::new();
                let margin = terminal.margin_str();
                let current = self.state.current_lesson().as_usize() + 1;
                println!("{}{} Lesson {} of {}", 
                    margin,
                    "[navigate]".green().bold(),
                    current,
                    self.lessons.len()
                );
                ui::wait_for_enter();
                Ok(CommandResult::Navigated)
            }
            Err(e) => {
                ui::show_error(&e.to_string());
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
            _ => {
                ui::show_error("Already at the first lesson");
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
        }
    }
    
    /// Quit the application
    fn cmd_quit(&mut self) -> Result<CommandResult> {
        // Check if there are unsaved changes
        let lesson_id = self.state.current_lesson();
        let edited_blocks = self.state.get_edited_blocks_for_lesson(lesson_id);
        
        if !edited_blocks.is_empty() {
            let terminal = ui::Terminal::new();
            let margin = terminal.margin_str();
            println!("{}{} You have unsaved changes in this lesson", 
                margin,
                "[warn]".yellow().bold()
            );
            println!("{}{} Are you sure you want to quit? (y/N)", margin, "[question]".yellow().bold());
            
            let mut response = String::new();
            std::io::stdin().read_line(&mut response).unwrap();
            
            if response.trim().to_lowercase() != "y" {
                return Ok(CommandResult::Continue);
            }
        }
        
        Ok(CommandResult::Quit)
    }
    
    /// Show help
    fn cmd_help(&mut self) -> Result<CommandResult> {
        ui::show_help(self.config);
        ui::wait_for_enter();
        Ok(CommandResult::Help)
    }
    
    /// Run a specific code block
    fn cmd_run_block(&mut self, block_str: &str) -> Result<CommandResult> {
        let block_num = block_str.parse::<usize>()
            .map_err(|_| RustlrnError::Parse("Invalid block number".to_string()))?;
        
        self.run_block(block_num)
    }
    
    /// Run the current lesson's main block (first executable block)
    fn cmd_run_current(&mut self) -> Result<CommandResult> {
        let blocks = executor::extract_code_blocks(&self.lessons[self.state.current_lesson().as_usize()]);
        
        // Find the first block with main function
        for (i, block) in blocks.iter().enumerate() {
            if executor::has_main_function(block) {
                return self.run_block(i + 1);
            }
        }
        
        // If no main function, run the first block
        if !blocks.is_empty() {
            return self.run_block(1);
        }
        
        ui::show_error("No executable code blocks found in this lesson");
        ui::wait_for_enter();
        Ok(CommandResult::Continue)
    }
    
    /// Execute a code block
    fn run_block(&mut self, block_num: usize) -> Result<CommandResult> {
        let lesson_id = self.state.current_lesson();
        let blocks = executor::extract_code_blocks(&self.lessons[lesson_id.as_usize()]);
        
        if block_num == 0 || block_num > blocks.len() {
            return Err(RustlrnError::Parse(format!(
                "Block {} not found (max: {})",
                block_num,
                blocks.len()
            )));
        }
        
        let block_idx = block_num - 1;
        let original = &blocks[block_idx];
        let code = self.state.get_code_block(lesson_id, BlockId::new(block_idx), original);
        
        let executable_code = executor::ensure_main_wrapper(&code);
        let exec_config = ExecutionConfig {
            compile_timeout: self.config.compile_timeout,
            run_timeout: self.config.run_timeout,
            ..ExecutionConfig::default()
        };
        
        let result = executor::execute_code_with_config(&executable_code, exec_config)?;
        ui::show_execution_result(&result, self.config);
        
        // Mark lesson as completed if execution succeeded
        if result.success {
            self.state.mark_current_completed();
        }
        
        Ok(CommandResult::Executed)
    }
    
    /// Edit a specific code block
    fn cmd_edit_block(&mut self, block_str: &str) -> Result<CommandResult> {
        let block_num = block_str.parse::<usize>()
            .map_err(|_| RustlrnError::Parse("Invalid block number".to_string()))?;
        
        self.edit_block(block_num)
    }
    
    /// Edit the last edited block or create new code
    fn cmd_edit_last(&mut self) -> Result<CommandResult> {
        let lesson_id = self.state.current_lesson();
        let edited_blocks = self.state.get_edited_blocks_for_lesson(lesson_id);
        
        if let Some((block_id, _)) = edited_blocks.iter().last() {
            return self.edit_block(block_id.as_usize() + 1);
        }
        
        // No edited blocks, open scratch pad
        match ui::edit_code_with_editor("", 0, self.config) {
            Ok(code) if !code.trim().is_empty() => {
                let lesson_id = self.state.current_lesson();
                let blocks = executor::extract_code_blocks(&self.lessons[lesson_id.as_usize()]);
                let block_idx = blocks.len(); // New block
                
                self.state.update_code_block(
                    lesson_id,
                    BlockId::new(block_idx),
                    code.clone(),
                    ""
                );
                
                ui::show_success(&format!("Created new code block #{}", block_idx + 1));
                
                let executable_code = executor::ensure_main_wrapper(&code);
                let exec_config = ExecutionConfig {
                    compile_timeout: self.config.compile_timeout,
                    run_timeout: self.config.run_timeout,
                    ..ExecutionConfig::default()
                };
                
                let result = executor::execute_code_with_config(&executable_code, exec_config)?;
                ui::show_execution_result(&result, self.config);
                
                if result.success {
                    self.state.mark_current_completed();
                }
                
                Ok(CommandResult::Edited)
            }
            Ok(_) => {
                ui::show_info("No code entered");
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
            Err(e) => {
                ui::show_error(&e.to_string());
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
        }
    }
    
    /// Edit a specific code block
    fn edit_block(&mut self, block_num: usize) -> Result<CommandResult> {
        let lesson_id = self.state.current_lesson();
        let blocks = executor::extract_code_blocks(&self.lessons[lesson_id.as_usize()]);
        
        if block_num == 0 || block_num > blocks.len() {
            return Err(RustlrnError::Parse(format!(
                "Block {} not found (max: {})",
                block_num,
                blocks.len()
            )));
        }
        
        let block_idx = block_num - 1;
        let original = &blocks[block_idx];
        let current_code = self.state.get_code_block(lesson_id, BlockId::new(block_idx), original);
        
        match ui::edit_code_with_editor(&current_code, block_num, self.config) {
            Ok(edited_code) if !edited_code.trim().is_empty() && edited_code != current_code => {
                self.state.update_code_block(
                    lesson_id,
                    BlockId::new(block_idx),
                    edited_code.clone(),
                    original
                );
                
                ui::show_success(&format!("Code block #{} updated!", block_num));
                
                // Auto-run if configured
                match self.config.auto_run {
                    crate::config::AutoRun::Always | crate::config::AutoRun::OnEdit => {
                        let executable_code = executor::ensure_main_wrapper(&edited_code);
                        let exec_config = ExecutionConfig {
                            compile_timeout: self.config.compile_timeout,
                            run_timeout: self.config.run_timeout,
                            ..ExecutionConfig::default()
                        };
                        let result = executor::execute_code_with_config(&executable_code, exec_config)?;
                        ui::show_execution_result(&result, self.config);
                        
                        if result.success {
                            self.state.mark_current_completed();
                        }
                    }
                    _ => {
                        ui::wait_for_enter();
                    }
                }
                
                Ok(CommandResult::Edited)
            }
            Ok(edited_code) if !edited_code.trim().is_empty() => {
                ui::show_info("No changes made");
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
            Ok(_) => {
                ui::show_info("No code entered");
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
            Err(e) => {
                ui::show_error(&e.to_string());
                ui::wait_for_enter();
                Ok(CommandResult::Continue)
            }
        }
    }
    
    /// Reset a specific code block
    fn cmd_reset_block(&mut self, block_str: &str) -> Result<CommandResult> {
        let block_num = block_str.parse::<usize>()
            .map_err(|_| RustlrnError::Parse("Invalid block number".to_string()))?;
        
        let lesson_id = self.state.current_lesson();
        let blocks = executor::extract_code_blocks(&self.lessons[lesson_id.as_usize()]);
        
        if block_num == 0 || block_num > blocks.len() {
            return Err(RustlrnError::Parse(format!(
                "Block {} not found (max: {})",
                block_num,
                blocks.len()
            )));
        }
        
        let block_idx = block_num - 1;
        if self.state.reset_code_block(lesson_id, BlockId::new(block_idx)) {
            ui::show_success(&format!("Reset code block #{} to original", block_num));
        } else {
            ui::show_info(&format!("Code block #{} was not modified", block_num));
        }
        ui::wait_for_enter();
        Ok(CommandResult::Reset)
    }
    
    /// Reset the current lesson's changes
    fn cmd_reset_current(&mut self) -> Result<CommandResult> {
        let lesson_id = self.state.current_lesson();
        let blocks = executor::extract_code_blocks(&self.lessons[lesson_id.as_usize()]);
        let mut reset_count = 0;
        
        for i in 0..blocks.len() {
            if self.state.reset_code_block(lesson_id, BlockId::new(i)) {
                reset_count += 1;
            }
        }
        
        if reset_count > 0 {
            ui::show_success(&format!("Reset {} code blocks", reset_count));
        } else {
            ui::show_info("No code blocks were modified");
        }
        ui::wait_for_enter();
        Ok(CommandResult::Reset)
    }
    
    /// Mark the current lesson as complete
    fn cmd_complete(&mut self) -> Result<CommandResult> {
        self.state.mark_current_completed();
        ui::show_success("Lesson marked as complete!");
        ui::wait_for_enter();
        Ok(CommandResult::Navigated)
    }
    
    /// Go to a specific lesson
    fn cmd_goto(&mut self, lesson_str: &str) -> Result<CommandResult> {
        let lesson_num = lesson_str.parse::<usize>()
            .map_err(|_| RustlrnError::Parse("Invalid lesson number".to_string()))?;
        
        if lesson_num < 1 || lesson_num > self.lessons.len() {
            return Err(RustlrnError::Parse(format!(
                "Lesson {} not found (1-{})",
                lesson_num,
                self.lessons.len()
            )));
        }
        
        let lesson_id = LessonId::new(lesson_num - 1);
        self.state.navigate_to(lesson_id)?;
        
        let terminal = ui::Terminal::new();
        let margin = terminal.margin_str();
        println!("{}{} Navigated to lesson {} of {}", 
            margin,
            "[navigate]".green().bold(),
            lesson_num,
            self.lessons.len()
        );
        ui::wait_for_enter();
        Ok(CommandResult::Navigated)
    }
    
    /// Show status information
    fn cmd_status(&mut self) -> Result<CommandResult> {
        let terminal = ui::Terminal::new();
        let margin = terminal.margin_str();
        
        println!("{}{}", margin, "Status:".cyan().bold());
        println!("{}{}", margin, "─".repeat(40).dimmed());
        
        let current = self.state.current_lesson().as_usize() + 1;
        let total = self.lessons.len();
        let completed = self.state.completed_count();
        
        println!("{}{} Current lesson: {}/{}", margin, "  ".dimmed(), current, total);
        println!("{}{} Completed: {}/{} ({:.1}%)", 
            margin, "  ".dimmed(), 
            completed, total, 
            self.state.progress_percentage()
        );
        
        let edited_blocks = self.state.get_edited_blocks_for_lesson(self.state.current_lesson());
        println!("{}{} Edited blocks: {}", margin, "  ".dimmed(), edited_blocks.len());
        
        let total_warnings = self.state.total_warnings();
        if total_warnings > 0 {
            println!("{}{} Total warnings: {}", margin, "  ".dimmed(), total_warnings.to_string().yellow());
        }
        
        let time_spent = self.state.time_spent(self.state.current_lesson());
        if time_spent.as_secs() > 0 {
            println!("{}{} Time on lesson: {}s", margin, "  ".dimmed(), time_spent.as_secs());
        }
        
        ui::wait_for_enter();
        Ok(CommandResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    
    fn setup_handler() -> (CommandHandler<'static>, Vec<String>, Config) {
        let lessons = vec![
            "Lesson 1\n```rust\nfn main() { println!(\"Hello\"); }\n```".to_string(),
            "Lesson 2\n```rust\nfn main() { println!(\"World\"); }\n```".to_string(),
        ];
        let config = Config::default();
        let mut state = AppState::new(0, lessons.len(), config.clone()).unwrap();
        let handler = CommandHandler::new(&mut state, &lessons);
        (handler, lessons, config)
    }
    
    #[test]
    fn test_command_parsing_next() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("n").unwrap();
        assert_eq!(result, CommandResult::Navigated);
    }
    
    #[test]
    fn test_command_parsing_quit() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("q").unwrap();
        assert_eq!(result, CommandResult::Quit);
    }
    
    #[test]
    fn test_command_parsing_help() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("h").unwrap();
        assert_eq!(result, CommandResult::Help);
    }
    
    #[test]
    fn test_command_parsing_goto() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("g2").unwrap();
        assert_eq!(result, CommandResult::Navigated);
        assert_eq!(handler.state.current_lesson().as_usize(), 1);
    }
    
    #[test]
    fn test_command_parsing_status() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("status").unwrap();
        assert_eq!(result, CommandResult::Continue);
    }
    
    #[test]
    fn test_invalid_command() {
        let (mut handler, _, _) = setup_handler();
        let result = handler.handle("invalid");
        assert!(result.is_err());
    }
}
