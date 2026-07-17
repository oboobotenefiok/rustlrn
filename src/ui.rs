use crate::NAME;
use colored::Colorize;
use crate::executor;

// 1. The clear_screen function (both versions)
#[cfg(target_os = "windows")]
fn clear_screen() {
    std::process::Command::new("cmd")
        .args(&["/c", "cls"])
        .status()
        .unwrap();
}

#[cfg(not(target_os = "windows"))]
pub fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}

// 2. Display functions to reduce repetitions and a subtle attempt at states.
pub fn show_header() {
    println!("{}", NAME.cyan().bold());
    println!("{}", "-".repeat(40).cyan());
}

/// Display a lesson with interactive code execution
///
/// This function now detects code blocks and offers execution options.
/// Users can run code snippets directly from the lesson view.
pub fn show_lesson(content: &str) {
    // Extract all code blocks for potential execution
    let code_blocks = executor::extract_code_blocks(content);
    let mut block_index = 0;

    // Process content line by line to style specific patterns
    let mut in_code_block = false;
    let mut code_buffer: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines but print them for spacing
        if trimmed.is_empty() {
            println!();
            continue;
        }

        // Detect code block markers
        if trimmed.starts_with("```") {
            if in_code_block && !code_buffer.is_empty() {
                // Display code block with run hint
                let code_text = code_buffer.join("\n");
                println!("{}", "  ┌─ Code Block ─────────────".dimmed());
                for code_line in code_buffer.iter() {
                    println!("  │ {}", code_line.dimmed());
                }
                println!("{}", "  └─────────────────────────".dimmed());

                // Show run option if this block has a main function or we can wrap it
                let has_main = executor::has_main_function(&code_text);
                if has_main || code_text.contains("println!") {
                    let block_num = block_index + 1;
                    println!("  {} Run this code? Press 'r{}'", "[r]".green().bold(), block_num);
                }

                block_index += 1;
                code_buffer.clear();
            }

            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            // Collect code lines for later display
            code_buffer.push(line.to_string());
            continue;
        }

        // Detect section headers (ALL CAPS with dashes or colons)
        if trimmed.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c == ':' || c == '•' || c == '━' || c == '═')
            && trimmed.len() > 3 
            && !trimmed.starts_with("TIP")
            && !trimmed.starts_with("NOTE")
            && !trimmed.starts_with("Based on")
            && !trimmed.starts_with("*Credit")
        {
            // UPPERCASE = header
            println!("{}\n", trimmed.yellow().bold());
        } else if trimmed.starts_with("TIP:") {
            println!("{}", trimmed.green().bold());
        } else if trimmed.starts_with("NOTE:") {
            println!("{}", trimmed.blue().bold());
        } else if trimmed.starts_with("✓") {
            println!("  {}", trimmed.green());
        } else if trimmed.starts_with("•") || trimmed.starts_with("-") {
            println!("  {}", trimmed.cyan());
        } else if trimmed.starts_with("Example:") || trimmed.starts_with("LOCAL VARIABLE SCOPE:") || trimmed.starts_with("STACK vs. HEAP:") {
            println!("\n{}", trimmed.cyan().bold());
        } else if trimmed.starts_with("fn main()") || trimmed.starts_with("fn status()") || trimmed.starts_with("let") {
            // Code snippets - display in dim
            println!("  {}", trimmed.dimmed());
        } else if trimmed.contains("ERROR!") {
            println!("  {}", trimmed.red().bold());
        } else if trimmed.starts_with("===") || trimmed.starts_with("━━━") {
            // Decorative lines - dim
            println!("{}", trimmed.dimmed());
        } else {
            // Regular text
            println!("{}", trimmed);
        }
    }

    // Handle any remaining code block
    if !code_buffer.is_empty() {
        let code_text = code_buffer.join("\n");
        println!("{}", "  ┌─ Code Block ─────────────".dimmed());
        for code_line in code_buffer.iter() {
            println!("  │ {}", code_line.dimmed());
        }
        println!("{}", "  └─────────────────────────".dimmed());

        let has_main = executor::has_main_function(&code_text);
        if has_main || code_text.contains("println!") {
            let block_num = block_index + 1;
            println!("  {} Run this code? Press 'r{}'", "[r]".green().bold(), block_num);
        }
    }
}

/// Display controls including execution commands
pub fn show_controls() {
    println!();
    println!("{}", "-".repeat(40).dimmed());
    println!(
        "{} {}  {} {}  {} {}  {} {}",
        "[n]".green().bold(),
        "next".dimmed(),
        "[p]".yellow().bold(),
        "previous".dimmed(),
        "[r#]".cyan().bold(),
        "run block #".dimmed(),
        "[q]".red().bold(),
        "quit".dimmed()
    );
    println!();
}

/// Show execution results
pub fn show_execution_result(result: &executor::ExecutionResult, _code: &str) {
    println!();
    println!("{}", "═".repeat(40).cyan());
    println!("{}", "▶ EXECUTION RESULT".cyan().bold());
    println!("{}", "─".repeat(40).dimmed());

    if result.success {
        println!("{}", "✓ SUCCESS".green().bold());
        if !result.output.is_empty() {
            println!("\n{}", "Output:".dimmed());
            for line in result.output.lines() {
                println!("  {}", line);
            }
        } else {
            println!("  {}", "(no output)".dimmed());
        }
    } else {
        println!("{}", "✗ ERROR".red().bold());
        if !result.error.is_empty() {
            println!("\n{}", "Compiler/Runtime Error:".red().dimmed());
            for line in result.error.lines() {
                println!("  {}", line.red());
            }
        }
    }

    println!("{}", "═".repeat(40).cyan());
    println!("\n{} Press any key to continue...", "[enter]".dimmed());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

pub fn show_error(msg: &str) {
    eprintln!("{}", msg.red().bold());
}
