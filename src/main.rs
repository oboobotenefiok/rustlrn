//! This is the entry point of the program and will mostly act as a 'pointer'

mod config;
mod executor;
mod lessons;
mod ui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Write};

const NAME: &str = "rustlrn";

#[derive(Parser)]
#[command(author, version, about = "Rust Tutor - Learn Rust interactively right from your terminal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Starting lesson number (1-5)
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
    lesson: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure the editor used for editing code blocks
    Editor {
        /// The editor command to use (e.g., nano, micro, "code --wait")
        command: String,
    },
}

struct AppState {
    current_lesson: usize,
    warn_count: usize,
    edited_blocks: std::collections::HashMap<(usize, usize), String>,
    config: config::Config,
}

impl AppState {
    fn new(lesson: u8) -> Self {
        Self {
            current_lesson: (lesson - 1) as usize,
            warn_count: 0,
            edited_blocks: std::collections::HashMap::new(),
            config: config::load_config(),
        }
    }

    fn increment_warning(&mut self) {
        self.warn_count += 1;
    }

    fn reset_warning(&mut self) {
        self.warn_count = 0;
    }

    fn has_warning(&self) -> bool {
        self.warn_count > 0
    }

    fn navigate_next(&mut self, total_lessons: usize) -> bool {
        if self.current_lesson < total_lessons - 1 {
            self.current_lesson += 1;
            true
        } else {
            false
        }
    }

    fn navigate_previous(&mut self) -> bool {
        if self.current_lesson > 0 {
            self.current_lesson -= 1;
            true
        } else {
            false
        }
    }

    fn get_cached_or_original<'a>(&'a self, lesson_idx: usize, block_idx: usize, original: &'a str) -> &'a str {
        self.edited_blocks
            .get(&(lesson_idx, block_idx))
            .map_or(original, |s| s.as_str())
    }

    fn update_code_block(&mut self, lesson_idx: usize, block_idx: usize, code: String) {
        self.edited_blocks.insert((lesson_idx, block_idx), code);
    }

    fn reset_code_block(&mut self, lesson_idx: usize, block_idx: usize) -> bool {
        self.edited_blocks.remove(&(lesson_idx, block_idx)).is_some()
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Editor { command }) = cli.command {
        handle_editor_command(&command);
        return;
    }

    let lessons = lessons::load_all_lessons();
    let mut state = AppState::new(cli.lesson);

    loop {
        ui::clear_screen();
        ui::show_header();
        ui::show_lesson(&lessons[state.current_lesson]);
        ui::show_controls();

        if state.has_warning() {
            ui::show_error("Please type a valid input");
            state.reset_warning();
        }

        let input = read_user_input();
        let input_trimmed = input.trim();

        if handle_command(input_trimmed, &mut state, &lessons).unwrap_or(false) {
            break;
        }
    }
}

fn read_user_input() -> String {
    let mut input = String::new();
    print!("{} ", "rustlrn >".cyan().bold());
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn handle_command(input: &str, state: &mut AppState, lessons: &[String]) -> Option<bool> {
    match input {
        "n" => {
            if !state.navigate_next(lessons.len()) {
                state.increment_warning();
            }
            None
        }
        "p" => {
            if !state.navigate_previous() {
                state.increment_warning();
            }
            None
        }
        "q" => Some(true),
        "r" => {
            show_help();
            ui::wait_for_enter();
            None
        }
        cmd if cmd.starts_with("ed0") => {
            handle_ed0_command(state);
            None
        }
        cmd if cmd.starts_with("ed") && cmd.len() > 2 => {
            handle_edit_command(cmd, state, lessons);
            None
        }
        cmd if cmd.starts_with('z') && cmd.len() > 1 => {
            handle_reset_command(cmd, state, lessons);
            None
        }
        cmd if cmd.starts_with('r') && cmd.len() > 1 => {
            handle_run_command(cmd, state, lessons);
            None
        }
        _ => {
            state.increment_warning();
            None
        }
    }
}

fn handle_ed0_command(state: &mut AppState) {
    match ui::edit_code_with_editor("", 0, &state.config) {
        Ok(code) if !code.trim().is_empty() => {
            handle_code_execution(&code, &state.config);
        }
        Ok(_) => {
            println!("{} No code entered", "[info]".blue().bold());
            ui::wait_for_enter();
        }
        Err(e) => {
            ui::show_error(&format!("Edit failed: {}", e));
            ui::wait_for_enter();
        }
    }
}

fn handle_edit_command(cmd: &str, state: &mut AppState, lessons: &[String]) {
    if let Ok(block_num) = cmd[2..].parse::<usize>() {
        let cached_blocks = executor::extract_code_blocks(&lessons[state.current_lesson]);
        
        if block_num > 0 && block_num <= cached_blocks.len() {
            let block_idx = block_num - 1;
            let current_code = state
                .get_cached_or_original(state.current_lesson, block_idx, &cached_blocks[block_idx])
                .to_string();

            match ui::edit_code_with_editor(&current_code, block_num, &state.config) {
                Ok(edited_code) if !edited_code.trim().is_empty() && edited_code != current_code => {
                    state.update_code_block(state.current_lesson, block_idx, edited_code.clone());
                    println!("{} Code block #{} updated!", "[✓]".green().bold(), block_num);
                    handle_code_execution(&edited_code, &state.config);
                }
                Ok(edited_code) if !edited_code.trim().is_empty() => {
                    println!("{} No changes made", "[info]".blue().bold());
                    ui::wait_for_enter();
                }
                Ok(_) => {
                    println!("{} No code entered", "[info]".blue().bold());
                    ui::wait_for_enter();
                }
                Err(e) => {
                    ui::show_error(&format!("Edit failed: {}", e));
                    ui::wait_for_enter();
                }
            }
        } else {
            state.increment_warning();
        }
    } else {
        state.increment_warning();
    }
}

fn handle_reset_command(cmd: &str, state: &mut AppState, lessons: &[String]) {
    if let Ok(block_num) = cmd[1..].parse::<usize>() {
        let cached_blocks = executor::extract_code_blocks(&lessons[state.current_lesson]);
        
        if block_num > 0 && block_num <= cached_blocks.len() {
            let block_idx = block_num - 1;
            if state.reset_code_block(state.current_lesson, block_idx) {
                println!("{} Reset code block #{} to original", "[✓]".green().bold(), block_num);
            } else {
                println!("{} Code block #{} was not modified", "[info]".blue().bold(), block_num);
            }
            ui::wait_for_enter();
        } else {
            state.increment_warning();
        }
    } else {
        state.increment_warning();
    }
}

fn handle_run_command(cmd: &str, state: &mut AppState, lessons: &[String]) {
    if let Ok(block_num) = cmd[1..].parse::<usize>() {
        let cached_blocks = executor::extract_code_blocks(&lessons[state.current_lesson]);
        
        if block_num > 0 && block_num <= cached_blocks.len() {
            let block_idx = block_num - 1;
            let code = state
                .get_cached_or_original(state.current_lesson, block_idx, &cached_blocks[block_idx]);

            let executable_code = executor::ensure_main_wrapper(code);
            let result = executor::execute_code(&executable_code);
            ui::show_execution_result(&result, code);
        } else {
            state.increment_warning();
        }
    } else {
        state.increment_warning();
    }
}

fn show_help() {
    println!("\n{} Commands:", "[help]".yellow().bold());
    println!("  {} r1, r2, r3    - Run specific code block", "[r#]".cyan().bold());
    println!("  {} ed1, ed2, ed3 - Edit specific code block", "[ed#]".green().bold());
    println!("  {} ed0           - Write code from scratch", "[ed0]".green().bold());
    println!("  {} z1, z2, z3    - Reset specific code block", "[z#]".red().bold());

    let cfg = config::load_config();
    if cfg.editor.is_none() {
        println!("\n{} No editor configured!", "[warn]".yellow().bold());
        println!("  {} Set editor: rustlrn editor <command>", "[setup]".cyan().bold());
        println!("  {} Example: rustlrn editor nano", "[example]".dimmed());
    }
}

fn handle_editor_command(command: &str) {
    println!("{} Setting editor to: {}", "[config]".cyan().bold(), command.cyan());

    if !config::validate_editor(command) {
        let cmd_name = command.split_whitespace().next().unwrap_or(command);
        println!("{} Warning: Could not find '{}'. Please ensure it's installed.", "[warn]".yellow().bold(), cmd_name);
        println!("{} You can still proceed, but the editor may not work.", "[info]".blue().bold());
    }

    if !config::is_blocking_editor(command) {
        println!("{} Note: '{}' may not block the terminal.", "[info]".blue().bold(), command);
        println!("{} For GUI editors, add --wait or -w flag.", "[hint]".yellow().bold());
        println!("{} Example: rustlrn editor \"code --wait\"", "[hint]".yellow().bold());
    }

    let new_config = config::Config {
        editor: Some(command.to_string()),
    };

    config::save_config(&new_config);
    println!("{} Editor configured successfully!", "[✓]".green().bold());
}

fn handle_code_execution(initial_code: &str, config: &config::Config) {
    let mut retry_count = 0;
    let max_retries = 3;
    let mut current_code = initial_code.to_string();

    loop {
        let executable_code = executor::ensure_main_wrapper(&current_code);
        let result = executor::execute_code(&executable_code);

        if result.success {
            ui::show_execution_result(&result, &current_code);
            break;
        }

        ui::show_execution_result(&result, &current_code);

        if retry_count >= max_retries {
            println!("\n{} Maximum retries exceeded. Please fix the code manually.", "[error]".red().bold());
            ui::wait_for_enter();
            break;
        }

        if !prompt_retry() {
            println!("{} Skipping retry.", "[info]".blue().bold());
            break;
        }

        retry_count += 1;

        match ui::edit_code_with_editor(&current_code, 0, config) {
            Ok(new_code) if !new_code.trim().is_empty() && new_code != current_code => {
                current_code = new_code;
            }
            Ok(_) => {
                println!("{} No changes made. Exiting.", "[info]".blue().bold());
                break;
            }
            Err(e) => {
                ui::show_error(&format!("Edit failed: {}", e));
                break;
            }
        }
    }
}

fn prompt_retry() -> bool {
    println!("\n{} Edit again? (y/n)", "[question]".yellow().bold());
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    response.trim().to_lowercase() == "y"
}
