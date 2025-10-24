/// Shell abstraction layer for executing commands and capturing output.
///
/// This module provides a trait-based abstraction over shell implementations,
/// allowing the correction engine to work with different shells (Bash, Zsh, etc.)
/// while maintaining a consistent interface.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

/// Result of executing a command in the shell.
#[derive(Debug, Clone)]
pub struct ShellOutput {
    /// The command that was executed
    pub command: String,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command succeeded
    pub success: bool,
}

impl ShellOutput {
    /// Creates a new ShellOutput from execution results.
    pub fn new(
        command: String,
        stdout: String,
        stderr: String,
        exit_code: i32,
    ) -> Self {
        let success = exit_code == 0;
        Self {
            command,
            stdout,
            stderr,
            exit_code,
            success,
        }
    }
}

/// Trait for shell implementations.
///
/// This trait abstracts away shell-specific behavior, allowing the correction
/// engine to work with different shells while maintaining a consistent API.
pub trait Shell: Send + Sync {
    /// Gets the name of the shell (e.g., "bash", "zsh")
    fn name(&self) -> &str;

    /// Executes a command and captures its output.
    fn execute(&self, command: &str) -> crate::Result<ShellOutput>;

    /// Gets the current working directory
    fn cwd(&self) -> crate::Result<PathBuf>;

    /// Sets the working directory for subsequent commands
    fn set_cwd(&mut self, path: PathBuf) -> crate::Result<()>;

    /// Gets an environment variable
    fn env(&self, key: &str) -> Option<String>;

    /// Sets an environment variable
    fn set_env(&mut self, key: String, value: String) -> crate::Result<()>;

    /// Gets the user's shell history (if available)
    /// Returns commands in reverse chronological order (most recent first)
    fn history(&self) -> crate::Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Checks if a command exists in PATH
    fn command_exists(&self, command: &str) -> crate::Result<bool>;
}

/// Bash shell implementation.
pub struct BashShell {
    cwd: PathBuf,
    env: HashMap<String, String>,
}

impl BashShell {
    /// Creates a new Bash shell instance.
    pub fn new() -> crate::Result<Self> {
        let cwd = std::env::current_dir()?;
        let env = std::env::vars().collect();

        Ok(Self { cwd, env })
    }
}

impl Default for BashShell {
    fn default() -> Self {
        Self::new().expect("Failed to initialize BashShell")
    }
}

impl Shell for BashShell {
    fn name(&self) -> &str {
        "bash"
    }

    fn execute(&self, command: &str) -> crate::Result<ShellOutput> {
        let output = StdCommand::new("bash")
            .arg("-c")
            .arg(command)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(1);

        Ok(ShellOutput::new(
            command.to_string(),
            stdout,
            stderr,
            exit_code,
        ))
    }

    fn cwd(&self) -> crate::Result<PathBuf> {
        Ok(self.cwd.clone())
    }

    fn set_cwd(&mut self, path: PathBuf) -> crate::Result<()> {
        std::env::set_current_dir(&path)?;
        self.cwd = path;
        Ok(())
    }

    fn env(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned()
    }

    fn set_env(&mut self, key: String, value: String) -> crate::Result<()> {
        std::env::set_var(&key, &value);
        self.env.insert(key, value);
        Ok(())
    }

    fn history(&self) -> crate::Result<Vec<String>> {
        // Try to read from HISTFILE if set, otherwise use ~/.bash_history
        let hist_file = self
            .env("HISTFILE")
            .unwrap_or_else(|| {
                format!(
                    "{}/.bash_history",
                    self.env("HOME").unwrap_or_else(|| ".".to_string())
                )
            });

        if let Ok(contents) = std::fs::read_to_string(&hist_file) {
            let lines: Vec<String> = contents
                .lines()
                .map(|s| s.to_string())
                .collect();
            Ok(lines.into_iter().rev().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn command_exists(&self, command: &str) -> crate::Result<bool> {
        let output = self.execute(&format!("command -v {}", command))?;
        Ok(output.success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_shell_creation() {
        let shell = BashShell::new();
        assert!(shell.is_ok());
    }

    #[test]
    fn test_bash_shell_name() {
        let shell = BashShell::new().unwrap();
        assert_eq!(shell.name(), "bash");
    }

    #[test]
    fn test_bash_shell_execute_success() {
        let shell = BashShell::new().unwrap();
        let output = shell.execute("echo 'hello'").unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("hello"));
    }

    #[test]
    fn test_bash_shell_execute_failure() {
        let shell = BashShell::new().unwrap();
        let output = shell.execute("exit 1").unwrap();
        assert!(!output.success);
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_bash_shell_cwd() {
        let shell = BashShell::new().unwrap();
        let cwd = shell.cwd().unwrap();
        assert!(cwd.exists());
    }

    #[test]
    fn test_bash_shell_env() {
        let mut shell = BashShell::new().unwrap();
        shell.set_env("TEST_VAR".to_string(), "test_value".to_string()).unwrap();
        assert_eq!(shell.env("TEST_VAR"), Some("test_value".to_string()));
    }

    #[test]
    fn test_bash_shell_command_exists_true() {
        let shell = BashShell::new().unwrap();
        let exists = shell.command_exists("echo").unwrap();
        assert!(exists);
    }

    #[test]
    fn test_bash_shell_command_exists_false() {
        let shell = BashShell::new().unwrap();
        let exists = shell.command_exists("nonexistent_command_xyz_12345").unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_shell_output_success() {
        let output = ShellOutput::new("test".to_string(), "output".to_string(), "".to_string(), 0);
        assert!(output.success);
    }

    #[test]
    fn test_shell_output_failure() {
        let output = ShellOutput::new("test".to_string(), "".to_string(), "error".to_string(), 1);
        assert!(!output.success);
    }
}
