use crate::config;
use crate::executor;
use crate::NAME;
use colored::Colorize;

#[cfg(target_os = "windows")]
pub fn clear_screen() {
    let _ = std::process::Command::new("cmd")
        .args(&["/c", "cls"])
        .status();
}

#[cfg(not(target_os = "windows"))]
pub fn clear_screen() {
    let _ = std::process::Command::new("clear").status();
}

fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(width, _)| width.0 as usize)
        .unwrap_or(80)
}

fn margin_str() -> String {
    let width = terminal_width();
    let margin = if width > 80 { 2 } else { 0 };
    " ".repeat(margin)
}

/// Wait for the user to press Enter before continuing
pub fn wait_for_enter() {
    let margin = margin_str();
    println!("\n{}{}", margin, "[enter] Press Enter to continue...".dimmed());
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

pub fn show_header() {
    let margin = margin_str();
    println!("{}{}", margin, NAME.cyan().bold());
    println!("{}{}", margin, "-".repeat(40).cyan());
}

pub fn edit_code_with_editor(
    code: &str,
    _block_num: usize,
    config: &config::Config,
) -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let editor_cmd = config
        .editor
        .as_ref()
        .ok_or_else(|| {
            format!(
                "No editor configured!\n\n\
                 Please set your editor with: rustlrn editor <command>\n\
                 Examples:\n\
                   rustlrn editor nano\n\
                   rustlrn editor micro\n\
                   rustlrn editor vim\n\
                   rustlrn editor \"code --wait\"\n\
                   rustlrn editor \"subl -w\""
            )
        })?;

    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    if !code.is_empty() {
        write!(temp_file, "{}", code)
            .map_err(|e| format!("Failed to write code to temp file: {}", e))?;
        temp_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;
    }

    let temp_path = temp_file.path();

    let cmd_parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    let (editor_cmd_base, editor_args) = cmd_parts.split_first()
        .ok_or_else(|| format!("Empty editor command: '{}'", editor_cmd))?;

    let margin = margin_str();
    println!(
        "{}{} Opening editor... Edit the code, save, and close the editor when done.",
        margin,
        "[hint]".yellow().bold()
    );

    let status = Command::new(editor_cmd_base)
        .args(editor_args)
        .arg(temp_path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor_cmd, e))?;

    if !status.success() {
        return Err("Editor exited with error".to_string());
    }

    fs::read_to_string(temp_path)
        .map_err(|e| format!("Failed to read edited code: {}", e))
        .and_then(|edited_code| {
            if edited_code.trim().is_empty() {
                Err("No code entered".to_string())
            } else {
                Ok(edited_code)
            }
        })
}

pub fn show_lesson(content: &str) {
    let margin = margin_str();
    let code_margin = " ".repeat(margin.len() + 2);

    let mut in_code_block = false;
    let mut code_buffer: Vec<String> = Vec::new();
    let mut block_index = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            println!();
            continue;
        }

        if trimmed.starts_with("```") {
            if in_code_block && !code_buffer.is_empty() {
                render_code_block(&code_buffer, block_index + 1, &margin, &code_margin);
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

        render_text_line(trimmed, &margin);
    }

    if !code_buffer.is_empty() {
        render_code_block(&code_buffer, block_index + 1, &margin, &code_margin);
    }
}

fn render_code_block(code_buffer: &[String], block_num: usize, margin: &str, code_margin: &str) {
    let code_text = code_buffer.join("\n");
    
    println!("{}{}", code_margin, "┌─ Code Block ─────────────".dimmed());
    for code_line in code_buffer {
        println!("{}{}", code_margin, format!("│ {}", code_line).dimmed());
    }
    println!("{}{}", code_margin, "└─────────────────────────".dimmed());

    let has_main = executor::has_main_function(&code_text);
    if has_main || code_text.contains("println!") {
        println!("{}{} '{}'", margin, "[ID]".green().bold(), block_num);
    }
}

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
        if trimmed.starts_with("TIP:") {
            LineType::Tip
        } else if trimmed.starts_with("NOTE:") {
            LineType::Note
        } else if trimmed.starts_with('✓') {
            LineType::Checkmark
        } else if trimmed.starts_with('•') || trimmed.starts_with('-') {
            LineType::Bullet
        } else if trimmed.starts_with("Example:")
            || trimmed.starts_with("LOCAL VARIABLE SCOPE:")
            || trimmed.starts_with("STACK vs. HEAP:")
        {
            LineType::SectionHeader
        } else if trimmed.starts_with("fn main()")
            || trimmed.starts_with("fn status()")
            || trimmed.starts_with("let")
        {
            LineType::CodeLine
        } else if trimmed.contains("ERROR!") {
            LineType::Error
        } else if trimmed.starts_with("===") || trimmed.starts_with("━━━") {
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
            LineType::SectionHeader => format!("\n{}{}", margin, trimmed.cyan().bold()),
            LineType::CodeLine => format!("{}{}", margin, format!("  {}", trimmed).dimmed()),
            LineType::Error => format!("{}{}", margin, format!("  {}", trimmed).red().bold()),
            LineType::Separator => format!("{}{}", margin, trimmed.dimmed()),
            LineType::MarkdownH1 => format!("\n{}{}\n", margin, trimmed[1..].trim().cyan().bold().underline()),
            LineType::MarkdownH2 => format!("\n{}{}\n", margin, trimmed[2..].trim().yellow().bold()),
            LineType::MarkdownH3 => format!("{}{}\n", margin, trimmed[3..].trim().white().bold()),
            LineType::TableRow => format!("{}{}", margin, trimmed.dimmed()),
            LineType::Plain => format!("{}{}", margin, trimmed),
        }
    }
}

fn is_heading(text: &str) -> bool {
    text.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c == ':' || c == '•' || c == '━' || c == '═')
        && text.len() > 3
        && !text.starts_with("TIP")
        && !text.starts_with("NOTE")
        && !text.starts_with("Based on")
        && !text.starts_with("*Credit")
        && !text.starts_with('|')
}

fn render_text_line(trimmed: &str, margin: &str) {
    let line_type = LineType::classify(trimmed);
    let rendered = line_type.render(trimmed, margin);
    print!("{}", rendered);
}

pub fn show_controls() {
    let margin = margin_str();

    println!();
    println!("{}{}", margin, "-".repeat(40).dimmed());
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

    let config = config::load_config();
    if config.editor.is_none() {
        println!(
            "{}{} No editor set. Run: rustlrn editor <command>",
            margin,
            "[!]".yellow().bold()
        );
    }

    println!();
}

pub fn show_execution_result(result: &executor::ExecutionResult, _code: &str) {
    let margin = margin_str();

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

    println!("{}{}", margin, "═".repeat(40).cyan());
    wait_for_enter();
}

pub fn show_error(msg: &str) {
    let margin = margin_str();
    eprintln!("{}{}", margin, msg.red().bold());
}
