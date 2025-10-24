/// Core types for the fasterthefuck command correction engine.

use std::fmt;

/// Represents a shell command that needs correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The original shell script/command
    pub script: String,
    /// The output/error from running the command
    pub output: String,
    /// Exit code from the command execution
    pub exit_code: i32,
}

impl Command {
    /// Creates a new Command instance.
    pub fn new(script: impl Into<String>, output: impl Into<String>, exit_code: i32) -> Self {
        Self {
            script: script.into(),
            output: output.into(),
            exit_code,
        }
    }

    /// Gets the command parts by splitting on whitespace.
    pub fn script_parts(&self) -> Vec<&str> {
        self.script.split_whitespace().collect()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Command(script={}, exit_code={}, output_len={})",
            self.script,
            self.exit_code,
            self.output.len()
        )
    }
}

/// A corrected command with metadata about which rule suggested it.
#[derive(Debug, Clone)]
pub struct CorrectedCommand {
    /// The corrected shell command/script
    pub script: String,
    /// Priority for sorting results (lower = higher priority)
    pub priority: i32,
    /// Optional side effect function name (for executing pre/post hooks)
    pub side_effect: Option<String>,
}

impl CorrectedCommand {
    /// Creates a new CorrectedCommand.
    pub fn new(script: impl Into<String>, priority: i32) -> Self {
        Self {
            script: script.into(),
            priority,
            side_effect: None,
        }
    }

    /// Creates a CorrectedCommand with a side effect.
    pub fn with_side_effect(
        script: impl Into<String>,
        priority: i32,
        side_effect: impl Into<String>,
    ) -> Self {
        Self {
            script: script.into(),
            priority,
            side_effect: Some(side_effect.into()),
        }
    }
}

impl PartialEq for CorrectedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.script == other.script && self.side_effect == other.side_effect
    }
}

impl Eq for CorrectedCommand {}

impl PartialOrd for CorrectedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CorrectedCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// Trait that all rules must implement.
pub trait Rule: Send + Sync {
    /// The name of the rule (e.g., "git_branch_delete")
    fn name(&self) -> &str;

    /// Returns true if this rule matches the given command.
    fn matches(&self, command: &Command) -> bool;

    /// Returns a list of corrected commands for the given command.
    fn get_new_commands(&self, command: &Command) -> Vec<String>;

    /// Whether this rule is enabled by default.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Priority for this rule (lower = higher priority). Default is 1000.
    fn priority(&self) -> i32 {
        1000
    }

    /// Whether this rule requires command output to match.
    fn requires_output(&self) -> bool {
        true
    }

    /// Gets corrected commands with priority and metadata.
    fn get_corrected_commands(&self, command: &Command) -> Vec<CorrectedCommand> {
        self.get_new_commands(command)
            .into_iter()
            .enumerate()
            .map(|(i, script)| {
                CorrectedCommand::new(script, (i as i32 + 1) * self.priority())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("ls -la", "error message", 1);
        assert_eq!(cmd.script, "ls -la");
        assert_eq!(cmd.output, "error message");
        assert_eq!(cmd.exit_code, 1);
    }

    #[test]
    fn test_command_script_parts() {
        let cmd = Command::new("git branch -d feature", "output", 0);
        assert_eq!(
            cmd.script_parts(),
            vec!["git", "branch", "-d", "feature"]
        );
    }

    #[test]
    fn test_corrected_command_priority() {
        let cmd1 = CorrectedCommand::new("cmd1", 100);
        let cmd2 = CorrectedCommand::new("cmd2", 50);
        assert!(cmd2 < cmd1);
    }
}
