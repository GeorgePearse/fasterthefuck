/// Simple rule builder for rules with basic string matching and replacement.

use crate::{Command, Rule};

/// A simple rule builder for string-based matching and replacement.
pub struct SimpleRuleBuilder {
    name: String,
    match_cmd: Option<String>,
    match_out: Option<String>,
    replacement: Option<(String, String)>,
    priority: i32,
}

impl SimpleRuleBuilder {
    /// Creates a new SimpleRuleBuilder with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            match_cmd: None,
            match_out: None,
            replacement: None,
            priority: 1000,
        }
    }

    /// Sets the command substring to match.
    pub fn match_command(mut self, pattern: impl Into<String>) -> Self {
        self.match_cmd = Some(pattern.into());
        self
    }

    /// Sets the output substring to match.
    pub fn match_output(mut self, pattern: impl Into<String>) -> Self {
        self.match_out = Some(pattern.into());
        self
    }

    /// Sets the priority (lower = higher priority).
    pub fn priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    /// Builds a simple rule with a fixed string replacement.
    pub fn replace(mut self, old: impl Into<String>, new: impl Into<String>) -> Box<dyn Rule> {
        self.replacement = Some((old.into(), new.into()));
        Box::new(SimpleRule {
            name: self.name,
            match_cmd: self.match_cmd,
            match_out: self.match_out,
            replacement: self.replacement,
            priority: self.priority,
        })
    }
}

/// Simple rule struct that matches on command/output and applies a correction.
struct SimpleRule {
    name: String,
    match_cmd: Option<String>,
    match_out: Option<String>,
    replacement: Option<(String, String)>,
    priority: i32,
}

impl Rule for SimpleRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, command: &Command) -> bool {
        let cmd_match = self
            .match_cmd
            .as_ref()
            .map(|p| command.script.contains(p))
            .unwrap_or(true);

        let out_match = self
            .match_out
            .as_ref()
            .map(|p| command.output.contains(p))
            .unwrap_or(true);

        cmd_match && out_match
    }

    fn get_new_commands(&self, command: &Command) -> Vec<String> {
        if let Some((old, new)) = &self.replacement {
            vec![command.script.replace(old, new)]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule_builder_with_replace() {
        let rule = SimpleRuleBuilder::new("test_rule")
            .match_command("git branch -d")
            .match_output("If you are sure")
            .replace("-d", "-D");

        let cmd = Command::new("git branch -d feature", "If you are sure to delete it", 1);
        assert!(rule.matches(&cmd));

        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert_eq!(corrections[0], "git branch -D feature");
    }

    #[test]
    fn test_simple_rule_builder_multiple_replacements() {
        let rule = SimpleRuleBuilder::new("test_rule")
            .match_command("test")
            .match_output("error")
            .replace("test", "test2");

        let cmd = Command::new("test test command", "error occurred", 1);
        assert!(rule.matches(&cmd));

        let corrections = rule.get_new_commands(&cmd);
        assert_eq!(corrections[0], "test2 test2 command");
    }

    #[test]
    fn test_simple_rule_priority() {
        let rule = SimpleRuleBuilder::new("test")
            .priority(500)
            .replace("a", "b");

        assert_eq!(rule.priority(), 500);
    }
}
