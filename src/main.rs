//! This is the entry point of the program and will mostly act as a 'pointer'

mod config;
mod executor;
mod lessons;
mod ui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io;
use std::io::Write;

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

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Editor { command } => {
                handle_editor_command(&command);
                return;
            }
        }
    }

    let config = config::load_config();

    let lessons = lessons::load_all_lessons();
    let mut current = (cli.lesson - 1) as usize;
    let mut warn_count = 0;

    use std::collections::HashMap;
    let mut edited_blocks: HashMap<(usize, usize), String> = HashMap::new();

    loop {
        ui::clear_screen();
        ui::show_header();
        ui::show_lesson(&lessons[current]);
        ui::show_controls();

        let warn = warn_count > 0;
        if warn {
            ui::show_error("Please type a valid input");
            warn_count = 0;
        }

        let mut input = String::new();
        print!("{} ", "rustlrn >".cyan().bold());
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input_trimmed = input.trim();

        let next = current < lessons.len() - 1;
        let previous = current > 0;

        let cached_blocks = executor::extract_code_blocks(&lessons[current]);

        match input_trimmed {
            "n" => {
                if next {
                    current += 1;
                } else {
                    warn_count += 1;
                }
            }
            "p" => {
                if previous {
                    current -= 1;
                } else {
                    warn_count += 1;
                }
            }
            cmd if cmd.starts_with("ed") && cmd.len() > 2 => {
                if let Ok(block_num) = cmd[2..].parse::<usize>() {
                    if block_num > 0 && block_num <= cached_blocks.len() {
                        let block_idx = block_num - 1;
                        let key = (current, block_idx);

                        let current_code = edited_blocks
                            .get(&key)
                            .unwrap_or(&cached_blocks[block_idx])
                            .clone();

                        match ui::edit_code_with_editor(&current_code, block_num, &config) {
                            Ok(edited_code) => {
                                if edited_code != current_code && !edited_code.trim().is_empty() {
                                    edited_blocks.insert(key, edited_code.clone());
                                    println!(
                                        "{} Code block #{} updated!",
                                        "[✓]".green().bold(),
                                        block_num
                                    );
                                    handle_code_execution(&edited_code);
                                } else {
                                    println!("{} No changes made", "[info]".blue().bold());
                                    println!(
                                        "\n{} Press any key to continue...",
                                        "[enter]".dimmed()
                                    );
                                    let mut temp = String::new();
                                    io::stdin().read_line(&mut temp).unwrap();
                                }
                            }
                            Err(e) => {
                                ui::show_error(&format!("Edit failed: {}", e));
                                println!("\n{} Press any key to continue...", "[enter]".dimmed());
                                let mut temp = String::new();
                                io::stdin().read_line(&mut temp).unwrap();
                            }
                        }
                    } else {
                        warn_count += 1;
                    }
                } else {
                    warn_count += 1;
                }
            }
            "ed0" => match ui::edit_code_with_editor("", 0, &config) {
                Ok(code) => {
                    if !code.trim().is_empty() {
                        handle_code_execution(&code);
                    } else {
                        println!("{} No code entered", "[info]".blue().bold());
                        println!("\n{} Press any key to continue...", "[enter]".dimmed());
                        let mut temp = String::new();
                        io::stdin().read_line(&mut temp).unwrap();
                    }
                }
                Err(e) => {
                    ui::show_error(&format!("Edit failed: {}", e));
                    println!("\n{} Press any key to continue...", "[enter]".dimmed());
                    let mut temp = String::new();
                    io::stdin().read_line(&mut temp).unwrap();
                }
            },
            cmd if cmd.starts_with('z') && cmd.len() > 1 => {
                if let Ok(block_num) = cmd[1..].parse::<usize>() {
                    if block_num > 0 && block_num <= cached_blocks.len() {
                        let block_idx = block_num - 1;
                        let key = (current, block_idx);

                        if edited_blocks.remove(&key).is_some() {
                            println!(
                                "{} Reset code block #{} to original",
                                "[✓]".green().bold(),
                                block_num
                            );
                        } else {
                            println!(
                                "{} Code block #{} was not modified",
                                "[info]".blue().bold(),
                                block_num
                            );
                        }

                        println!("\n{} Press any key to continue...", "[enter]".dimmed());
                        let mut temp = String::new();
                        io::stdin().read_line(&mut temp).unwrap();
                    } else {
                        warn_count += 1;
                    }
                } else {
                    warn_count += 1;
                }
            }
            cmd if cmd.starts_with('r') && cmd.len() > 1 => {
                if let Ok(block_num) = cmd[1..].parse::<usize>() {
                    if block_num > 0 && block_num <= cached_blocks.len() {
                        let block_idx = block_num - 1;
                        let key = (current, block_idx);

                        let code = edited_blocks.get(&key).unwrap_or(&cached_blocks[block_idx]);

                        let executable_code = executor::ensure_main_wrapper(code);
                        let result = executor::execute_code(&executable_code);
                        ui::show_execution_result(&result, code);
                    } else {
                        warn_count += 1;
                    }
                } else {
                    warn_count += 1;
                }
            }
            "r" => {
                println!("\n{} Commands:", "[help]".yellow().bold());
                println!(
                    "  {} r1, r2, r3    - Run specific code block",
                    "[r#]".cyan().bold()
                );
                println!(
                    "  {} ed1, ed2, ed3 - Edit specific code block",
                    "[ed#]".green().bold()
                );
                println!(
                    "  {} ed0           - Write code from scratch",
                    "[ed0]".green().bold()
                );
                println!(
                    "  {} z1, z2, z3    - Reset specific code block",
                    "[z#]".red().bold()
                );

                let cfg = config::load_config();
                if cfg.editor.is_none() {
                    println!("\n{} No editor configured!", "[warn]".yellow().bold());
                    println!(
                        "  {} Set editor: rustlrn editor <command>",
                        "[setup]".cyan().bold()
                    );
                    println!("  {} Example: rustlrn editor nano", "[example]".dimmed());
                }

                println!("{} Press any key to continue...", "[enter]".dimmed());
                let mut temp = String::new();
                io::stdin().read_line(&mut temp).unwrap();
            }
            "q" => break,
            _ => {
                warn_count += 1;
                continue;
            }
        }
    }
}

fn handle_editor_command(command: &str) {
    println!(
        "{} Setting editor to: {}",
        "[config]".cyan().bold(),
        command.cyan()
    );

    if !config::validate_editor(command) {
        println!(
            "{} Warning: Could not find '{}'. Please ensure it's installed.",
            "[warn]".yellow().bold(),
            command.split_whitespace().next().unwrap_or(command)
        );
        println!(
            "{} You can still proceed, but the editor may not work.",
            "[info]".blue().bold()
        );
    }

    if !config::is_blocking_editor(command) {
        println!(
            "{} Note: '{}' may not block the terminal.",
            "[info]".blue().bold(),
            command
        );
        println!(
            "{} For GUI editors, add --wait or -w flag.",
            "[hint]".yellow().bold()
        );
        println!(
            "{} Example: rustlrn editor \"code --wait\"",
            "[hint]".yellow().bold()
        );
    }

    let new_config = config::Config {
        editor: Some(command.to_string()),
    };

    config::save_config(&new_config);
    println!("{} Editor configured successfully!", "[✓]".green().bold());
}

fn handle_code_execution(code: &str) {
    let mut retry_count = 0;
    let max_retries = 3;
    let mut current_code = code.to_string();

    loop {
        let executable_code = executor::ensure_main_wrapper(&current_code);
        let result = executor::execute_code(&executable_code);

        if result.success {
            ui::show_execution_result(&result, &current_code);
            break;
        } else {
            ui::show_execution_result(&result, &current_code);

            if retry_count >= max_retries {
                println!(
                    "\n{} Maximum retries exceeded. Please fix the code manually.",
                    "[error]".red().bold()
                );
                println!("{} Press any key to continue...", "[enter]".dimmed());
                let mut temp = String::new();
                io::stdin().read_line(&mut temp).unwrap();
                break;
            }

            println!("\n{} Edit again? (y/n)", "[question]".yellow().bold());

            let mut response = String::new();
            io::stdin().read_line(&mut response).unwrap();

            if response.trim().to_lowercase() != "y" {
                println!("{} Skipping retry.", "[info]".blue().bold());
                break;
            }

            retry_count += 1;

            let config = config::load_config();

            match ui::edit_code_with_editor(&current_code, 0, &config) {
                Ok(new_code) => {
                    if new_code.trim().is_empty() || new_code == current_code {
                        println!("{} No changes made. Exiting.", "[info]".blue().bold());
                        break;
                    }
                    current_code = new_code;
                }
                Err(e) => {
                    ui::show_error(&format!("Edit failed: {}", e));
                    break;
                }
            }
        }
    }
}
