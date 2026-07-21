// File: src/main.rs
//! Entry point for the RustLrn application
//!
//! This module handles command-line argument parsing, application initialization,
//! and the main event loop with proper error handling and logging.

mod config;
mod error;
mod executor;
mod lessons;
mod state;
mod ui;
mod commands;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Write};
use std::time::Instant;
use std::process;

use crate::config::Config;
use crate::error::{ErrorContext, Result, RustlrnError};
use crate::executor::ExecutionConfig;
use crate::state::{AppState, LessonId, BlockId};
use crate::commands::{CommandHandler, CommandResult};

/// Application name
const APP_NAME: &str = "rustlrn";

/// Application version
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command-line interface structure
#[derive(Parser)]
#[command(
    author = "RustLrn Team",
    version = APP_VERSION,
    about = "Rust Tutor - Learn Rust interactively right from your terminal",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Starting lesson number (1-indexed)
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=10))]
    lesson: u8,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug output
    #[arg(long, global = true)]
    debug: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Configuration file path
    #[arg(long, global = true)]
    config: Option<String>,
}

/// Subcommands for the CLI
#[derive(Subcommand)]
enum Commands {
    /// Configure the editor used for editing code blocks
    Editor {
        /// The editor command to use (e.g., nano, micro, "code --wait")
        command: String,

        /// Validate the editor command exists
        #[arg(long)]
        validate: bool,
    },

    /// Show current configuration
    Config,

    /// Reset configuration to defaults
    ConfigReset,

    /// Show lesson information
    Lessons {
        /// List all available lessons
        #[arg(long)]
        list: bool,

        /// Show specific lesson
        #[arg(long)]
        show: Option<u8>,
    },

    /// Run code without opening editor
    Run {
        /// The code to run (provided as a string)
        code: Option<String>,

        /// Run from a file
        #[arg(short, long)]
        file: Option<String>,

        /// Code block to run from current lesson
        #[arg(short, long)]
        block: Option<u8>,
    },

    /// Clear all user progress
    Clear,
}

/// Main entry point
fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Configure colored output
    if cli.no_color {
        colored::control::set_override(false);
    }
    
    // Initialize logging
    let log_level = if cli.debug {
        log::LevelFilter::Debug
    } else if cli.verbose {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Warn
    };
    
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_millis()
        .init();
    
    log::info!("Starting {} v{}", APP_NAME, APP_VERSION);
    
    // Load configuration
    let config = match load_configuration(&cli) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{} Failed to load configuration: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };
    
    // Handle subcommands
    if let Some(command) = cli.command {
        let result = handle_subcommand(command, &config, cli.lesson);
        if let Err(e) = result {
            eprintln!("{} {}", "✗".red().bold(), e);
            process::exit(1);
        }
        return;
    }
    
    // Load lessons
    let lessons = match lessons::load_all_lessons() {
        Ok(lessons) => lessons,
        Err(e) => {
            eprintln!("{} Failed to load lessons: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };
    
    if lessons.is_empty() {
        eprintln!("{} No lessons found!", "✗".red().bold());
        process::exit(1);
    }
    
    // Create application state
    let start_lesson = (cli.lesson - 1) as usize;
    let state = match AppState::new(start_lesson, lessons.len(), config) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("{} Failed to initialize application: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };
    
    // Run the main application loop
    if let Err(e) = run_app(state, &lessons) {
        log::error!("Application error: {}", e);
        
        // Save progress before exiting
        if let Err(save_err) = state.save_progress() {
            log::error!("Failed to save progress: {}", save_err);
        }
        
        // Show user-friendly error message
        let terminal = ui::Terminal::new();
        let margin = terminal.margin_str();
        eprintln!(
            "{}{} An error occurred: {}",
            margin,
            "✗".red().bold(),
            error::ui_errors::display_error(&e)
        );
        
        // Suggest recovery if available
        if let Some(suggestion) = error::ui_errors::suggest_recovery(&e) {
            eprintln!("{}{} {}", margin, "💡".yellow().bold(), suggestion);
        }
        
        process::exit(1);
    }
}

/// Load configuration with command-line overrides
fn load_configuration(cli: &Cli) -> Result<Config> {
    let mut config = if let Some(path) = &cli.config {
        // Load from custom path
        let content = std::fs::read_to_string(path)
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to read config file")?;
        
        toml::from_str(&content)
            .map_err(|e| RustlrnError::Config(format!("Invalid config file: {}", e)))?
    } else {
        // Load from default path
        Config::load()?
    };
    
    // Override verbosity from CLI
    if cli.verbose {
        config.verbosity = config::Verbosity::Verbose;
    }
    if cli.debug {
        config.verbosity = config::Verbosity::Debug;
    }
    
    Ok(config)
}

/// Handle subcommands
fn handle_subcommand(command: Commands, config: &Config, lesson: u8) -> Result<()> {
    match command {
        Commands::Editor { command, validate } => {
            handle_editor_command(&command, validate, config)
        }
        Commands::Config => {
            handle_config_command(config)
        }
        Commands::ConfigReset => {
            handle_config_reset()
        }
        Commands::Lessons { list, show } => {
            handle_lessons_command(list, show)
        }
        Commands::Run { code, file, block } => {
            handle_run_command(code, file, block, config)
        }
        Commands::Clear => {
            handle_clear_command()
        }
    }
}

/// Handle the editor subcommand
fn handle_editor_command(command: &str, validate: bool, config: &Config) -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    println!("{}{} Setting editor to: {}", margin, "[config]".cyan().bold(), command.cyan());
    
    if validate || config::validate_editor(command) {
        if !config::validate_editor(command) {
            let cmd_parts: Vec<&str> = command.split_whitespace().collect();
            let cmd_name = cmd_parts.first().unwrap_or(&"");
            return Err(RustlrnError::Config(format!(
                "Editor '{}' not found in PATH",
                cmd_name
            )));
        }
        println!("{}{} Editor validated successfully", margin, "✓".green().bold());
    } else {
        println!(
            "{}{} Skipping validation (use --validate to check)",
            margin,
            "[info]".blue().bold()
        );
    }
    
    if !config::is_blocking_editor(command) {
        println!(
            "{}{} Note: '{}' may not block the terminal.",
            margin,
            "[info]".blue().bold(),
            command
        );
        println!(
            "{}{} For GUI editors, add --wait or -w flag.",
            margin,
            "[hint]".yellow().bold()
        );
        println!(
            "{}{} Example: rustlrn editor \"code --wait\"",
            margin,
            "[hint]".yellow().bold()
        );
    }
    
    // Save the configuration
    let mut new_config = config.clone();
    new_config.set_editor(command.to_string())?;
    new_config.save()?;
    
    println!("{}{} Editor configured successfully!", margin, "✓".green().bold());
    Ok(())
}

/// Handle the config subcommand
fn handle_config_command(config: &Config) -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    println!("{}{}", margin, "Current Configuration:".cyan().bold());
    println!("{}{}", margin, "─".repeat(40).dimmed());
    
    println!(
        "{}{} Editor: {}",
        margin,
        "  Editor".dimmed(),
        config.get_editor().unwrap_or("(not set)").yellow()
    );
    println!(
        "{}{} Theme: {}",
        margin,
        "  Theme".dimmed(),
        format!("{:?}", config.theme).cyan()
    );
    println!(
        "{}{} Verbosity: {}",
        margin,
        "  Verbosity".dimmed(),
        format!("{:?}", config.verbosity).cyan()
    );
    println!(
        "{}{} Auto-run: {}",
        margin,
        "  Auto-run".dimmed(),
        format!("{:?}", config.auto_run).cyan()
    );
    println!(
        "{}{} Code Display: {}",
        margin,
        "  Code Display".dimmed(),
        format!("{:?}", config.code_display).cyan()
    );
    println!(
        "{}{} Line Numbers: {}",
        margin,
        "  Line Numbers".dimmed(),
        if config.show_line_numbers { "enabled".green() } else { "disabled".red() }
    );
    println!(
        "{}{} Max Retries: {}",
        margin,
        "  Max Retries".dimmed(),
        config.max_retries.to_string().yellow()
    );
    println!(
        "{}{} Compile Timeout: {}s",
        margin,
        "  Compile Timeout".dimmed(),
        config.compile_timeout.to_string().yellow()
    );
    println!(
        "{}{} Run Timeout: {}s",
        margin,
        "  Run Timeout".dimmed(),
        config.run_timeout.to_string().yellow()
    );
    
    if !config.custom_settings.is_empty() {
        println!("\n{}{}", margin, "Custom Settings:".dimmed());
        for (key, value) in &config.custom_settings {
            println!("{}{} {} = {}", margin, "  ", key.dimmed(), value.to_string().cyan());
        }
    }
    
    Ok(())
}

/// Handle the config-reset subcommand
fn handle_config_reset() -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    print!(
        "{}{} Reset all configuration to defaults? (y/N): ",
        margin,
        "[question]".yellow().bold()
    );
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if input.trim().to_lowercase() == "y" {
        let default_config = Config::default();
        default_config.save()?;
        println!("{}{} Configuration reset to defaults", margin, "✓".green().bold());
    } else {
        println!("{}{} Reset cancelled", margin, "[info]".blue().bold());
    }
    
    Ok(())
}

/// Handle the lessons subcommand
fn handle_lessons_command(list: bool, show: Option<u8>) -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    let lessons = lessons::load_all_lessons()?;
    
    if list {
        println!("{}{}", margin, "Available Lessons:".cyan().bold());
        println!("{}{}", margin, "─".repeat(40).dimmed());
        
        for (i, content) in lessons.iter().enumerate() {
            let title = content.lines()
                .find(|line| line.starts_with("# ") || line.starts_with("## "))
                .unwrap_or(&format!("Lesson {}", i + 1))
                .trim_start_matches('#')
                .trim();
            
            let num_blocks = executor::extract_code_blocks(content).len();
            let has_main = content.contains("fn main()");
            
            println!(
                "{}{} {}: {} {}",
                margin,
                format!("[{}]", i + 1).cyan(),
                title,
                if has_main { "[executable]".green() } else { "[reference]".dimmed() },
                format!("({} blocks)", num_blocks).dimmed()
            );
        }
        return Ok(());
    }
    
    if let Some(lesson_num) = show {
        let idx = (lesson_num - 1) as usize;
        if idx >= lessons.len() {
            return Err(RustlrnError::Lesson(format!(
                "Lesson {} not found (max: {})",
                lesson_num,
                lessons.len()
            )));
        }
        
        ui::show_header(APP_NAME);
        ui::show_lesson(&lessons[idx], &Config::default(), true);
        return Ok(());
    }
    
    // Default: show lesson count
    println!(
        "{}{} {} lessons available",
        margin,
        "[info]".cyan().bold(),
        lessons.len()
    );
    println!(
        "{}{} Use --list to see all lessons",
        margin,
        "[hint]".dimmed()
    );
    println!(
        "{}{} Use --show <number> to view a lesson",
        margin,
        "[hint]".dimmed()
    );
    
    Ok(())
}

/// Handle the run subcommand
fn handle_run_command(code: Option<String>, file: Option<String>, block: Option<u8>, config: &Config) -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    // Get the code to run
    let code_to_run = if let Some(code_str) = code {
        code_str
    } else if let Some(file_path) = file {
        std::fs::read_to_string(&file_path)
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to read file")?
    } else if let Some(block_num) = block {
        // Load the current lesson and extract the block
        let lessons = lessons::load_all_lessons()?;
        let idx = 0; // Default to first lesson for now
        if idx >= lessons.len() {
            return Err(RustlrnError::Lesson("No lessons available".to_string()));
        }
        
        let blocks = executor::extract_code_blocks(&lessons[idx]);
        let block_idx = (block_num - 1) as usize;
        if block_idx >= blocks.len() {
            return Err(RustlrnError::Lesson(format!(
                "Block {} not found in lesson (max: {})",
                block_num,
                blocks.len()
            )));
        }
        blocks[block_idx].clone()
    } else {
        return Err(RustlrnError::Parse(
            "Please provide code, a file, or a block number".to_string()
        ));
    };
    
    // Execute the code
    let code_with_main = executor::ensure_main_wrapper(&code_to_run);
    let exec_config = ExecutionConfig {
        compile_timeout: config.compile_timeout,
        run_timeout: config.run_timeout,
        ..ExecutionConfig::default()
    };
    
    let result = executor::execute_code_with_config(&code_with_main, exec_config)?;
    ui::show_execution_result(&result, config);
    
    Ok(())
}

/// Handle the clear subcommand
fn handle_clear_command() -> Result<()> {
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    
    print!(
        "{}{} Clear all user progress? (y/N): ",
        margin,
        "[question]".yellow().bold()
    );
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if input.trim().to_lowercase() == "y" {
        let progress_path = state::UserProgress::progress_path();
        if progress_path.exists() {
            std::fs::remove_file(&progress_path)
                .map_err(|e| RustlrnError::Io(e))?;
        }
        println!("{}{} Progress cleared successfully", margin, "✓".green().bold());
    } else {
        println!("{}{} Clear cancelled", margin, "[info]".blue().bold());
    }
    
    Ok(())
}

/// Run the main application loop
fn run_app(mut state: AppState, lessons: &[String]) -> Result<()> {
    let start_time = Instant::now();
    let config = state.config().clone();
    
    // Create command handler
    let mut command_handler = CommandHandler::new(&mut state, lessons);
    
    loop {
        // Clear screen and show UI
        ui::clear_screen();
        ui::show_header(APP_NAME);
        
        // Show progress
        let progress = state.progress_percentage();
        if progress > 0.0 {
            ui::show_progress_bar(progress, &format!("{}% complete", progress.round()));
        }
        
        // Show current lesson
        let current_idx = state.current_lesson().as_usize();
        ui::show_lesson(&lessons[current_idx], config, config.show_line_numbers);
        
        // Show completion status
        if state.is_lesson_completed(state.current_lesson()) {
            let terminal = ui::Terminal::new();
            let margin = terminal.margin_str();
            println!("{}{}", margin, "✓ Lesson completed!".green().bold());
        }
        
        ui::show_controls(config);
        
        // Show warning if any
        if state.has_warning() {
            let remaining = state.max_warnings - state.warning_count();
            ui::show_warning(&format!(
                "Invalid input. Please use a valid command. ({} remaining)",
                remaining
            ));
            state.reset_warning();
        }
        
        // Read user input
        let input = read_user_input();
        let input_trimmed = input.trim();
        
        // Handle the command
        let (should_break, should_save) = match command_handler.handle(input_trimmed) {
            Ok(CommandResult::Quit) => {
                log::info!("User quit application");
                (true, true)
            }
            Ok(CommandResult::Continue) => {
                log::debug!("Continuing...");
                (false, false)
            }
            Ok(CommandResult::Navigated) => {
                log::debug!("Navigated to lesson {}", state.current_lesson().as_usize());
                (false, true)
            }
            Ok(CommandResult::Executed) => {
                log::debug!("Code executed");
                (false, true)
            }
            Ok(CommandResult::Edited) => {
                log::debug!("Code edited");
                (false, true)
            }
            Ok(CommandResult::Reset) => {
                log::debug!("Code reset");
                (false, true)
            }
            Ok(CommandResult::Help) => {
                ui::show_help(config);
                ui::wait_for_enter();
                (false, false)
            }
            Err(e) => {
                log::debug!("Command error: {}", e);
                state.increment_warning();
                let terminal = ui::Terminal::new();
                let margin = terminal.margin_str();
                eprintln!("{}{}", margin, e.red().bold());
                ui::wait_for_enter();
                (false, false)
            }
        };
        
        // Save progress if needed
        if should_save {
            if let Err(e) = state.save_progress() {
                log::error!("Failed to save progress: {}", e);
            }
        }
        
        // Track time spent
        if !should_break {
            let elapsed = start_time.elapsed();
            state.track_time(state.current_lesson(), elapsed);
        }
        
        if should_break {
            break;
        }
    }
    
    // Save final progress
    state.save_progress()?;
    
    // Show goodbye message
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    println!("\n{}{} Thanks for learning Rust with {}!", 
        margin,
        "👋".green().bold(),
        APP_NAME.cyan().bold()
    );
    
    // Show summary
    let completed = state.completed_count();
    let total = state.total_lessons();
    let time_spent = start_time.elapsed();
    
    println!(
        "{}{} Completed {} of {} lessons ({:.1}%) in {:.1}s",
        margin,
        "[stats]".dimmed(),
        completed,
        total,
        state.progress_percentage(),
        time_spent.as_secs_f64()
    );
    
    Ok(())
}

/// Read user input from stdin
fn read_user_input() -> String {
    let mut input = String::new();
    let terminal = ui::Terminal::new();
    let margin = terminal.margin_str();
    print!("{}{} ", margin, format!("{} >", APP_NAME).cyan().bold());
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input
}
