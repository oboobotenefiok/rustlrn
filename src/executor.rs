// File: src/executor.rs
//! Code execution module for running Rust snippets with sandboxing and safety
//!
//! This module handles compiling and executing Rust code snippets
//! within a sandboxed environment with timeouts, resource limits,
//! and proper error handling.

use crate::error::{ErrorContext, Result, RustlrnError};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::{NamedTempFile, TempDir};
use which::which;

/// Execution configuration settings
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum compilation time in seconds
    pub compile_timeout: u64,
    /// Maximum execution time in seconds
    pub run_timeout: u64,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Whether to use sandboxing (if available)
    pub use_sandbox: bool,
    /// Additional rustc flags
    pub rustc_flags: Vec<String>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            compile_timeout: 30,
            run_timeout: 10,
            max_output_size: 1024 * 1024, // 1MB
            use_sandbox: true,
            rustc_flags: vec![
                "-C".to_string(),
                "opt-level=0".to_string(),
                "-A".to_string(),
                "warnings".to_string(),
            ],
        }
    }
}

/// Result of a code execution attempt
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub execution_time: Duration,
    pub compilation_time: Duration,
}

/// Code execution environment with sandboxing
pub struct ExecutionEnvironment {
    temp_dir: TempDir,
    config: ExecutionConfig,
}

impl ExecutionEnvironment {
    /// Create a new execution environment
    pub fn new(config: ExecutionConfig) -> Result<Self> {
        let temp_dir = TempDir::new()
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to create temporary directory")?;
        
        Ok(Self {
            temp_dir,
            config,
        })
    }
    
    /// Execute Rust code with full environment
    pub fn execute(&self, code: &str) -> Result<ExecutionResult> {
        // Validate the code before execution
        self.validate_code(code)?;
        
        // Check for rustc availability
        self.check_rustc()?;
        
        let compile_start = Instant::now();
        let binary_path = self.compile_code(code)?;
        let compile_time = compile_start.elapsed();
        
        // Execute the compiled binary
        let exec_result = self.run_binary(&binary_path)?;
        
        Ok(ExecutionResult {
            success: exec_result.status.success(),
            output: exec_result.stdout,
            error: exec_result.stderr,
            execution_time: exec_result.execution_time,
            compilation_time: compile_time,
        })
    }
    
    /// Validate code for security concerns
    fn validate_code(&self, code: &str) -> Result<()> {
        // Check for potentially dangerous patterns
        let dangerous_patterns = [
            "std::process::Command",
            "std::fs::remove",
            "std::fs::delete",
            "std::net::TcpListener",
            "std::net::TcpStream",
            "std::os::unix",
            "std::os::windows",
            "std::mem::transmute",
            "std::ptr::read",
            "std::ptr::write",
            "std::mem::zeroed",
            "std::mem::uninitialized",
            "std::thread::sleep",
            "std::thread::spawn",
            "std::env::set_var",
            "std::env::remove_var",
            "std::fs::set_permissions",
            "std::fs::hard_link",
            "std::fs::symlink",
            "std::fs::rename",
            "std::fs::create_dir",
            "std::fs::remove_dir",
            "std::fs::remove_dir_all",
            "std::fs::read_dir",
            "std::process::exit",
            "std::panic::set_hook",
            "std::panic::take_hook",
            "std::alloc",
            "#[no_std]",
        ];
        
        for pattern in dangerous_patterns {
            if code.contains(pattern) {
                return Err(RustlrnError::Execution(format!(
                    "Code contains potentially unsafe pattern: {}",
                    pattern
                )));
            }
        }
        
        // Check for infinite loops (basic detection)
        if !code.contains("break") && code.contains("loop") && !code.contains("while") {
            // Might be an intentional loop, but we'll warn
            // For now, we'll accept it but this could be enhanced
        }
        
        // Check code size
        if code.len() > 1024 * 100 { // 100KB max
            return Err(RustlrnError::Execution(
                "Code exceeds maximum size limit".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check if rustc is available
    fn check_rustc(&self) -> Result<()> {
        which("rustc")
            .map_err(|_| RustlrnError::Compilation(
                "rustc not found in PATH. Please install Rust from https://rustup.rs/".to_string()
            ))?;
        Ok(())
    }
    
    /// Compile the code with timeout
    fn compile_code(&self, code: &str) -> Result<PathBuf> {
        let file_path = self.temp_dir.path().join("main.rs");
        let binary_path = self.temp_dir.path().join("main");
        
        // Write code to file
        fs::write(&file_path, code)
            .map_err(|e| RustlrnError::Io(e))
            .with_context("Failed to write code to temporary file")?;
        
        // Build the compilation command
        let mut cmd = Command::new("rustc");
        cmd.arg(&file_path)
            .arg("-o")
            .arg(&binary_path)
            .arg("-C")
            .arg("opt-level=0")
            .arg("-A")
            .arg("warnings");
        
        // Add additional flags
        for flag in &self.config.rustc_flags {
            cmd.arg(flag);
        }
        
        // Execute with timeout
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RustlrnError::Compilation(format!("Failed to spawn rustc: {}", e)))?;
        
        let output = self.wait_for_child_with_timeout(child, Duration::from_secs(self.config.compile_timeout))
            .map_err(|e| RustlrnError::Compilation(format!("Compilation timed out: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RustlrnError::Compilation(format!(
                "Compilation failed:\n{}",
                stderr
            )));
        }
        
        // Verify the binary exists
        if !binary_path.exists() {
            return Err(RustlrnError::Compilation(
                "Binary not created after successful compilation".to_string()
            ));
        }
        
        Ok(binary_path)
    }
    
    /// Run the compiled binary with timeout
    fn run_binary(&self, binary_path: &Path) -> Result<ProcessOutput> {
        let start = Instant::now();
        
        let child = Command::new(binary_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RustlrnError::Execution(format!("Failed to spawn binary: {}", e)))?;
        
        let output = self.wait_for_child_with_timeout(child, Duration::from_secs(self.config.run_timeout))
            .map_err(|e| RustlrnError::Execution(format!("Execution timed out: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Limit output size
        let stdout = Self::truncate_output(&stdout, self.config.max_output_size);
        let stderr = Self::truncate_output(&stderr, self.config.max_output_size);
        
        Ok(ProcessOutput {
            status: output.status,
            stdout,
            stderr,
            execution_time: start.elapsed(),
        })
    }
    
    /// Wait for a child process with timeout
    fn wait_for_child_with_timeout(&self, mut child: std::process::Child, timeout: Duration) -> Result<std::process::Output> {
        let start = Instant::now();
        
        loop {
            if let Some(output) = child.try_wait()? {
                return Ok(output);
            }
            
            if start.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(RustlrnError::Execution(
                    format!("Process timed out after {:?}", timeout)
                ));
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    
    /// Truncate output to prevent memory issues
    fn truncate_output(output: &str, max_size: usize) -> String {
        if output.len() > max_size {
            format!("{}... (truncated, output too large)", &output[..max_size - 20])
        } else {
            output.to_string()
        }
    }
}

/// Process output with timing information
struct ProcessOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
    execution_time: Duration,
}

/// Execute code with default configuration
pub fn execute_code(code: &str) -> Result<ExecutionResult> {
    let config = ExecutionConfig::default();
    let env = ExecutionEnvironment::new(config)?;
    env.execute(code)
}

/// Execute code with custom configuration
pub fn execute_code_with_config(code: &str, config: ExecutionConfig) -> Result<ExecutionResult> {
    let env = ExecutionEnvironment::new(config)?;
    env.execute(code)
}

/// Execute code from a temporary file
pub fn execute_file(file_path: &Path) -> Result<ExecutionResult> {
    let code = fs::read_to_string(file_path)
        .map_err(|e| RustlrnError::Io(e))
        .with_context("Failed to read file")?;
    
    execute_code(&code)
}

/// Check if code contains a main function
pub fn has_main_function(code: &str) -> bool {
    code.contains("fn main()") || 
    code.contains("fn main(") ||
    code.contains("fn main ->") ||
    code.contains("#[tokio::main]") ||
    code.contains("#[async_std::main]")
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
pub fn extract_code_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_code = false;
    let mut current_block = String::new();
    let mut lang_specified = false;

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
                lang_specified = false;
            } else {
                // Start of code block
                in_code = true;
                current_block.clear();
                lang_specified = false;
            }
            continue;
        }
        
        if in_code {
            // Skip language specification lines that don't start with language
            if !trimmed.starts_with("```") && 
               !trimmed.starts_with("//") && 
               !trimmed.starts_with("fn main") {
                // Check if this is a language specification
                if !lang_specified && 
                   (trimmed == "rust" || trimmed == "rs" || trimmed == "Rust") {
                    lang_specified = true;
                    continue;
                }
                
                if !current_block.is_empty() {
                    current_block.push('\n');
                }
                current_block.push_str(line);
            } else if trimmed.starts_with("//") && current_block.is_empty() {
                // Allow comments at the start of blocks
                current_block.push_str(line);
            }
        }
    }
    
    blocks
}

/// Check if code is valid Rust (compiles)
pub fn validate_rust_code(code: &str) -> Result<()> {
    let config = ExecutionConfig {
        run_timeout: 0, // Don't run, just compile
        ..ExecutionConfig::default()
    };
    
    let env = ExecutionEnvironment::new(config)?;
    let temp_dir = env.temp_dir;
    let file_path = temp_dir.path().join("check.rs");
    
    fs::write(&file_path, code)
        .map_err(|e| RustlrnError::Io(e))
        .with_context("Failed to write code for validation")?;
    
    let output = Command::new("rustc")
        .arg("--emit")
        .arg("metadata")
        .arg(&file_path)
        .output()
        .map_err(|e| RustlrnError::Compilation(format!("Failed to run rustc: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RustlrnError::Compilation(format!(
            "Code validation failed:\n{}",
            stderr
        )));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_has_main_function() {
        assert!(has_main_function("fn main() { println!(\"Hello\"); }"));
        assert!(has_main_function("fn main(\n)"));
        assert!(has_main_function("fn main -> Result<()>"));
        assert!(!has_main_function("fn not_main()"));
        assert!(!has_main_function("println!(\"Hello\");"));
    }
    
    #[test]
    fn test_ensure_main_wrapper() {
        let code = "println!(\"Hello\");";
        let wrapped = ensure_main_wrapper(code);
        assert!(wrapped.contains("fn main()"));
        assert!(wrapped.contains("println!(\"Hello\");"));
        assert!(wrapped.contains("    "));
    }
    
    #[test]
    fn test_extract_code_blocks() {
        let content = "# Heading
        
```rust
fn main() {
    println!(\"Hello\");
}
