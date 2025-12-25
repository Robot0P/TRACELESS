//! Command utilities for cross-platform command execution
//!
//! This module provides utilities for executing system commands
//! without showing console windows on Windows.

use std::process::{Command, Output};
use std::io;

/// Extension trait for Command to hide console window on Windows
pub trait CommandExt {
    /// Configure the command to hide the console window on Windows
    fn hide_window(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    #[cfg(target_os = "windows")]
    fn hide_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt as WinCommandExt;
        // CREATE_NO_WINDOW = 0x08000000
        self.creation_flags(0x08000000);
        self
    }

    #[cfg(not(target_os = "windows"))]
    fn hide_window(&mut self) -> &mut Self {
        self
    }
}

/// Create a new command with hidden window on Windows
pub fn command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.hide_window();
    cmd
}

/// Execute a command and return its output, hiding the window on Windows
pub fn execute(program: &str, args: &[&str]) -> io::Result<Output> {
    command(program).args(args).output()
}

/// Execute a command and return stdout as string, hiding the window on Windows
pub fn execute_string(program: &str, args: &[&str]) -> io::Result<String> {
    let output = execute(program, args)?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = command("echo");
        assert!(cmd.get_program().to_str().unwrap().contains("echo"));
    }
}
