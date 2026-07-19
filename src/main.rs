//! This is the entry point of the program and will mostly act as a 'pointer' :-)

//  //! means comment on this file(module)
//  /// means documentation comment
// Well, just an inline
/* */
// <--And That One Above Is Multi-line comment

mod config;
mod executor;
mod lessons;
mod ui; // Import the UI module and his brothers.

// You must declare the type of constants, and also best practice to keep it capital.
// My major use case for constants is for source of truth otherwise you'll almost NEVER see me use it.
// Quick one is, you cannot use the to_string() or  string::from("") on CONST cause of compile time needs. We'll use &str.

// JUST KNOW THAT HEAP ALLOCATION CAN'T HAPPEN AT COMPILE TIME.
const NAME: &str = "rustlrn";
/* Spectra, Obot here... feel we can make this a CLI tool for learning Rust so I'll add actual stuff to main and we can iterate from there. Think it will be cool that way? I'm not versed in the contribution culture but for now this comment should be fine :-) We can learn a lot from implementing this and fuse it to an app or web later when we understand better...*/

// That reminds me, this binary will handroll its own interface. and many other features.
use clap::{Parser, Subcommand};

/// In the last one I used
/// `use colored::*`
// But that led to errors. I had to change it to Colorize. Note that it begins with capital C
use colored::Colorize;
/// The main function will basically be kept as empty as possible and we'll route several things to other modules.
/// This will serve as one of the evidence of understanding handrolling.
// At first, all we have to do is make the program move from one step to another, like PREVIOUS, NEXT!
// Then we make it feel as instant and re-filling as possible.
// That will be enough proof-of-work!!!!
// I would have used the terminal ANSI colors but since we are here for Rust, we'll......PLEASE NOTE THAT THE COMMENTS IN HERE WILL SWITCH BETWEEN WE, I, OUR, ME, MY , US, INTERCHANGEABLY...
// Like I was saying, we are going to use the colored crate instead of ANSI.

// We bring it in because it is not in the PRELUDE exposed by the compiler.

// A PRELUDE is the ATMOSPHERE of the compiler you are experiencing right now. If you still don't understand that, come back to this comment in future.
use std::io;
use std::io::Write;
// This derive gave me headache

/// Trait for clap needs derive to be implemented for certain stuff. Be sure to keep an eye at Cargo.toml.
// I had to run `cargo add clap --features derive` to get it.
// Also there are a lot of ambiguities in crate versions. It works for now so no problem.
// The syntax you see here is referred to as 'attribute'
#[derive(Parser)]
#[command(author, version, about = "Rust Tutor - Learn Rust interactively right from your terminal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Starting lesson number (1-5)
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
    lesson: u8, // short means we can pass 'l' but can't add short for any other subcommand.
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

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Editor { command } => {
                handle_editor_command(&command);
                return;
            }
        }
    }

    // Load config
    let config = config::load_config();

    use crate::lessons;
    let lessons = [
        lessons::intermediate::ownership::obot(),
        "Lesson 2: Variables - let x = 5;",
        "Lesson 3: Functions - fn add(a: i32, b: i32) -> i32 { a + b }",
        "Lesson 4: If/Else - if x > 0 { println!(\"Positive\"); }",
        "Lesson 5: Loops - for i in 0..5 { println!(\"{}\", i); }",
    ];

    let mut current = (cli.lesson - 1) as usize;
    let mut warn_count = 0;

    // Track edited code blocks
    use std::collections::HashMap;
    let mut edited_blocks: HashMap<(usize, usize), String> = HashMap::new();

    loop {
        ui::clear_screen();
        ui::show_header();
        ui::show_lesson(lessons[current]);
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

        // Get fresh code blocks for the current lesson
        let cached_blocks = executor::extract_code_blocks(lessons[current]);

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
            // Edit command: ed1, ed2, ed3, etc.
            cmd if cmd.starts_with("ed") && cmd.len() > 2 => {
                if let Ok(block_num) = cmd[2..].parse::<usize>() {
                    if block_num > 0 && block_num <= cached_blocks.len() {
                        let block_idx = block_num - 1;
                        let key = (current, block_idx);

                        // Get the current code (either original or edited)
                        let current_code = edited_blocks
                            .get(&key)
                            .unwrap_or(&cached_blocks[block_idx])
                            .clone();

                        // Try to edit with the configured editor
                        match ui::edit_code_with_editor(&current_code, block_num, &config) {
                            Ok(edited_code) => {
                                if edited_code != current_code && !edited_code.trim().is_empty() {
                                    edited_blocks.insert(key, edited_code.clone());
                                    println!(
                                        "{} Code block #{} updated!",
                                        "[✓]".green().bold(),
                                        block_num
                                    );

                                    // Auto-execute with retry on error
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
            // Free-form code editing mode: ed0
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
            // Reset command: z1, z2, z3, etc.
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
            // Handle code execution: r1, r2, r3, etc.
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

                // Check if editor is configured
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

/// Handle the editor configuration command
fn handle_editor_command(command: &str) {
    println!(
        "{} Setting editor to: {}",
        "[config]".cyan().bold(),
        command.cyan()
    );

    // Validate the editor
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

    // Check if it's a non-blocking GUI editor
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

/// Handle code execution with retry on error
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

            // Reload config
            let config = config::load_config();

            // Re-edit the code
            match ui::edit_code_with_editor(&current_code, 0, &config) {
                Ok(new_code) => {
                    if new_code.trim().is_empty() || new_code == current_code {
                        println!("{} No changes made. Exiting.", "[info]".blue().bold());
                        break;
                    }
                    current_code = new_code;
                    // Continue the loop
                }
                Err(e) => {
                    ui::show_error(&format!("Edit failed: {}", e));
                    break;
                }
            }
        }
    }
}
