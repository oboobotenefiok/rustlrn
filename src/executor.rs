//! Code execution module for running Rust snippets
//!
//! This module handles compiling and executing Rust code snippets
//! within the learning environment. It's designed for internal use
//! with trusted users, so sandboxing is minimal.

use std::fs;
use std::process::Command;
use tempfile::tempdir;

/// Result of a code execution attempt
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: String,
}

/// Execute a Rust code snippet
///
/// # Arguments
/// * `code` - The Rust code to compile and run
///
/// # Returns
/// * `ExecutionResult` - Contains success status, stdout, and stderr
///
/// # Example
/// ```rust
/// let result = execute_code(r#"fn main() { println!("Hello"); }"#);
/// assert!(result.success);
/// assert_eq!(result.output.trim(), "Hello");
/// ```
pub fn execute_code(code: &str) -> ExecutionResult {
    // Create a temporary directory for compilation
    let temp_dir = match tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return ExecutionResult {
                success: false,
                output: String::new(),
                error: format!("Failed to create temp directory: {}", e),
            };
        }
    };

    let file_path = temp_dir.path().join("main.rs");
    let binary_path = temp_dir.path().join("main");

    // Write the code to a file
    if let Err(e) = fs::write(&file_path, code) {
        return ExecutionResult {
            success: false,
            output: String::new(),
            error: format!("Failed to write code: {}", e),
        };
    }

    // Compile the code
    let compile_output = Command::new("rustc")
        .arg(&file_path)
        .arg("-o")
        .arg(&binary_path)
        .output();

    let compile_output = match compile_output {
        Ok(output) => output,
        Err(e) => {
            return ExecutionResult {
                success: false,
                output: String::new(),
                error: format!("Failed to run rustc: {}", e),
            };
        }
    };

    // Check if compilation succeeded
    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return ExecutionResult {
            success: false,
            output: String::new(),
            error: stderr.to_string(),
        };
    }

    // Run the compiled binary
    let run_output = Command::new(&binary_path).output();

    let run_output = match run_output {
        Ok(output) => output,
        Err(e) => {
            return ExecutionResult {
                success: false,
                output: String::new(),
                error: format!("Failed to run binary: {}", e),
            };
        }
    };

    // Return the results
    ExecutionResult {
        success: run_output.status.success(),
        output: String::from_utf8_lossy(&run_output.stdout).to_string(),
        error: String::from_utf8_lossy(&run_output.stderr).to_string(),
    }
}

/// Check if code contains a main function
pub fn has_main_function(code: &str) -> bool {
    code.contains("fn main()") || code.contains("fn main(")
}

/// Wrap code in a main function if it doesn't have one
pub fn ensure_main_wrapper(code: &str) -> String {
    if has_main_function(code) {
        code.to_string()
    } else {
        format!(
            "fn main() {{\n    {}\n}}",
            code.lines()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Extract code blocks from a lesson text
///
/// This parses the lesson content and returns all code blocks
/// marked with triple backticks.
pub fn extract_code_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_code = false;
    let mut current_block = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code {
                // End of code block
                if !current_block.is_empty() {
                    blocks.push(current_block.clone());
                    current_block.clear();
                }
                in_code = false;
            } else {
                // Start of code block
                in_code = true;
                current_block.clear();
            }
            continue;
        }

        if in_code {
            // Skip language annotation lines
            if !trimmed.starts_with("```") && trimmed != "rust" && !trimmed.starts_with("//") {
                if !current_block.is_empty() {
                    current_block.push('\n');
                }
                current_block.push_str(line);
            }
        }
    }

    blocks
}
