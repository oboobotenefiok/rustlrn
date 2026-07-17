use crate::NAME;
use colored::Colorize;

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

pub fn show_lesson(content: &str) {
    // Process content line by line to style specific patterns
    let mut in_code_block = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines but print them for spacing
        if trimmed.is_empty() {
            println!();
            continue;
        }
        
        // Detect code block markers
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        
        if in_code_block {
            // Code block - display in dim
            println!("  {}", line.dimmed());
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
}

pub fn show_controls() {
    println!();
    println!("{}", "-".repeat(40).dimmed());
    println!(
        "{} {}  {} {}  {} {}",
        "[n]".green().bold(),
        "next".dimmed(),
        "[p]".yellow().bold(),
        "previous".dimmed(),
        "[q]".red().bold(),
        "quit".dimmed()
    );
    println!();
}

pub fn show_error(msg: &str) {
    eprintln!("{}", msg.red().bold());
}
