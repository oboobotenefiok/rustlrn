use crate::config;
use crate::executor;
use crate::NAME;
use colored::Colorize;

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

fn get_terminal_width() -> usize {
    if let Some((width, _height)) = terminal_size::terminal_size() {
        width.0 as usize
    } else {
        80
    }
}

pub fn show_header() {
    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);
    
    println!("{}{}", margin_str, NAME.cyan().bold());
    println!("{}{}", margin_str, "-".repeat(40).cyan());
}

pub fn edit_code_with_editor(
    code: &str,
    block_num: usize,
    config: &config::Config,
) -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let editor_cmd = match &config.editor {
        Some(cmd) => cmd,
        None => {
            return Err("No editor configured!\n\n\
                 Please set your editor with: rustlrn editor <command>\n\
                 Examples:\n\
                   rustlrn editor nano\n\
                   rustlrn editor micro\n\
                   rustlrn editor vim\n\
                   rustlrn editor \"code --wait\"\n\
                   rustlrn editor \"subl -w\""
                .to_string());
        }
    };

    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    if !code.is_empty() {
        write!(temp_file, "{}", code)
            .map_err(|e| format!("Failed to write code to temp file: {}", e))?;
        temp_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;
    }

    let temp_path = temp_file.path().to_path_buf();
    let temp_path_display = temp_path.display();

    let block_label = if block_num > 0 {
        format!("block #{}", block_num)
    } else {
        "free-form".to_string()
    };

    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);

  
       
    println!(
        "{}{} Edit the code, save, and close the editor. After seeing a successful result, press ENTER to continue",
        margin_str,
        "[hint]".yellow().bold()
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    let editor_cmd_base = cmd_parts[0];
    let editor_args = &cmd_parts[1..];

    let status = Command::new(editor_cmd_base)
        .args(editor_args)
        .arg(&temp_path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor_cmd, e))?;

    if !status.success() {
        return Err(format!("Editor exited with error"));
    }

    let edited_code = fs::read_to_string(&temp_path)
        .map_err(|e| format!("Failed to read edited code: {}", e))?;

    if edited_code.trim().is_empty() {
        return Err("No code entered".to_string());
    }

    Ok(edited_code)
}

pub fn show_lesson(content: &str) {
    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);
    let code_margin = " ".repeat(margin + 2);
    
    let code_blocks = executor::extract_code_blocks(content);
    let mut block_index = 0;

    let mut in_code_block = false;
    let mut code_buffer: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            println!();
            continue;
        }

        if trimmed.starts_with("```") {
            if in_code_block && !code_buffer.is_empty() {
                let code_text = code_buffer.join("\n");
                let block_num = block_index + 1;

                println!("{}{}", code_margin, "┌─ Code Block ─────────────".dimmed());
                for code_line in code_buffer.iter() {
                    println!("{}{}", code_margin, format!("│ {}", code_line).dimmed());
                }
                println!("{}{}", code_margin, "└─────────────────────────".dimmed());

                let has_main = executor::has_main_function(&code_text);
                if has_main || code_text.contains("println!") {
                    println!(
                        "{}{} '{}'",
                        margin_str,
                        "[ID]".green().bold(),
                        block_num
                    );
                }

                block_index += 1;
                code_buffer.clear();
            }

            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            code_buffer.push(line.to_string());
            continue;
        }

        if trimmed.chars().all(|c| {
            c.is_uppercase() || c.is_whitespace() || c == ':' || c == '•' || c == '━' || c == '═'
        }) && trimmed.len() > 3
            && !trimmed.starts_with("TIP")
            && !trimmed.starts_with("NOTE")
            && !trimmed.starts_with("Based on")
            && !trimmed.starts_with("*Credit")
            && !trimmed.starts_with("|")
        {
            println!("{}{}\n", margin_str, trimmed.yellow().bold());
        } else if trimmed.starts_with("TIP:") {
            println!("{}{}", margin_str, trimmed.green().bold());
        } else if trimmed.starts_with("NOTE:") {
            println!("{}{}", margin_str, trimmed.blue().bold());
        } else if trimmed.starts_with("✓") {
            println!("{}{}", margin_str, format!("  {}", trimmed).green());
        } else if trimmed.starts_with("•") || trimmed.starts_with("-") {
            println!("{}{}", margin_str, format!("  {}", trimmed).cyan());
        } else if trimmed.starts_with("Example:")
            || trimmed.starts_with("LOCAL VARIABLE SCOPE:")
            || trimmed.starts_with("STACK vs. HEAP:")
        {
            println!("\n{}{}", margin_str, trimmed.cyan().bold());
        } else if trimmed.starts_with("fn main()")
            || trimmed.starts_with("fn status()")
            || trimmed.starts_with("let")
        {
            println!("{}{}", margin_str, format!("  {}", trimmed).dimmed());
        } else if trimmed.contains("ERROR!") {
            println!("{}{}", margin_str, format!("  {}", trimmed).red().bold());
        } else if trimmed.starts_with("===") || trimmed.starts_with("━━━") {
            println!("{}{}", margin_str, trimmed.dimmed());
        } else if trimmed.starts_with("#") && !trimmed.starts_with("##") {
            println!("\n{}{}\n", margin_str, trimmed[1..].trim().cyan().bold().underline());
        } else if trimmed.starts_with("##") {
            println!("\n{}{}\n", margin_str, trimmed[2..].trim().yellow().bold());
        } else if trimmed.starts_with("###") {
            println!("{}{}\n", margin_str, trimmed[3..].trim().white().bold());
        } else if trimmed.starts_with("|") && trimmed.contains("|") {
            println!("{}{}", margin_str, trimmed.dimmed());
        } else {
            println!("{}{}", margin_str, trimmed);
        }
    }

    if !code_buffer.is_empty() {
        let code_text = code_buffer.join("\n");
        let block_num = block_index + 1;

        println!("{}{}", code_margin, "┌─ Code Block ─────────────".dimmed());
        for code_line in code_buffer.iter() {
            println!("{}{}", code_margin, format!("│ {}", code_line).dimmed());
        }
        println!("{}{}", code_margin, "└─────────────────────────".dimmed());

        let has_main = executor::has_main_function(&code_text);
        if has_main || code_text.contains("println!") {
            println!(
                "{}{} Run this code? Press 'r{}'",
                margin_str,
                "[r]".green().bold(),
                block_num
            );
        }
        println!(
            "{}{} Edit this code? Press 'ed{}'",
            margin_str,
            "[ed]".cyan().bold(),
            block_num
        );
        println!(
            "{}{} Reset to original? Press 'z{}'",
            margin_str,
            "[z]".red().bold(),
            block_num
        );
    }
}

pub fn show_controls() {
    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);
    
    println!();
    println!("{}{}", margin_str, "-".repeat(40).dimmed());
    println!(
        "{}{} {}  {} {}  {} {}  {} {}  {} {}  {} {}",
        margin_str,
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

    let config = config::load_config();
    if config.editor.is_none() {
        println!(
            "{}{} No editor set. Run: rustlrn editor <command>",
            margin_str,
            "[!]".yellow().bold()
        );
    }

    println!();
}

pub fn show_execution_result(result: &executor::ExecutionResult, _code: &str) {
    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);
    
    println!();
    println!("{}{}", margin_str, "═".repeat(40).cyan());
    println!("{}{}", margin_str, "▶ EXECUTION RESULT".cyan().bold());
    println!("{}{}", margin_str, "─".repeat(40).dimmed());

    if result.success {
        println!("{}{}", margin_str, "✓ SUCCESS".green().bold());
        if !result.output.is_empty() {
            println!("\n{}{}", margin_str, "Output:".dimmed());
            for line in result.output.lines() {
                println!("{}{}", margin_str, format!("  {}", line));
            }
        } else {
            println!("{}{}", margin_str, "  (no output)".dimmed());
        }
    } else {
        println!("{}{}", margin_str, "✗ ERROR".red().bold());
        if !result.error.is_empty() {
            println!("\n{}{}", margin_str, "Compiler/Runtime Error:".red().dimmed());
            for line in result.error.lines() {
                println!("{}{}", margin_str, format!("  {}", line).red());
            }
        }
    }

    println!("{}{}", margin_str, "═".repeat(40).cyan());
    println!("\n{}{} Press any key to continue...", margin_str, "[enter]".dimmed());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

pub fn show_error(msg: &str) {
    let term_width = get_terminal_width();
    let margin = if term_width > 80 { 2 } else { 0 };
    let margin_str = " ".repeat(margin);
    
    eprintln!("{}{}", margin_str, msg.red().bold());
}
