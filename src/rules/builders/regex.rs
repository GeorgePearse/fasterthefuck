/// Regex-based rule builder for pattern matching and replacement with capture groups.

use crate::{Command, Rule};
use regex::Regex;

/// A rule builder for regex-based pattern matching with capture group support.
pub struct RegexRuleBuilder {
    name: String,
    command_pattern: Option<Regex>,
    output_pattern: Option<Regex>,
    replacement_fn: Option<Box<dyn Fn(&str, &regex::Captures) -> Vec<String> + Send + Sync>>,
    priority: i32,
}

impl RegexRuleBuilder {
    /// Creates a new RegexRuleBuilder with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command_pattern: None,
            output_pattern: None,
            replacement_fn: None,
            priority: 1000,
        }
    }

    /// Sets a regex pattern to match against the command.
    /// Returns an error if the regex is invalid.
    pub fn match_command_regex(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.command_pattern = Some(Regex::new(pattern)?);
        Ok(self)
    }

    /// Sets a regex pattern to match against the command output.
    /// Returns an error if the regex is invalid.
    pub fn match_output_regex(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.output_pattern = Some(Regex::new(pattern)?);
        Ok(self)
    }

    /// Sets the priority (lower = higher priority).
    pub fn priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    /// Sets a replacement function that generates corrections based on the matched command and capture groups.
    /// The function receives the original command and the regex captures.
    pub fn replace_with<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &regex::Captures) -> Vec<String> + Send + Sync + 'static,
    {
        self.replacement_fn = Some(Box::new(f));
        self
    }

    /// Builds the regex rule.
    pub fn build(self) -> Result<Box<dyn Rule>, String> {
        if self.command_pattern.is_none() && self.output_pattern.is_none() {
            return Err("RegexRuleBuilder: At least one pattern (command or output) must be set"
                .to_string());
        }

        if self.replacement_fn.is_none() {
            return Err("RegexRuleBuilder: replacement_fn must be set".to_string());
        }

        Ok(Box::new(RegexRule {
            name: self.name,
            command_pattern: self.command_pattern,
            output_pattern: self.output_pattern,
            replacement_fn: self.replacement_fn.unwrap(),
            priority: self.priority,
        }))
    }

    /// Convenience method: builds a rule with a simple string replacement.
    /// The replacement string can reference capture groups using $1, $2, etc.
    pub fn replace_simple(self, replacement_template: impl Into<String>) -> Result<Box<dyn Rule>, String> {
        let template = replacement_template.into();
        self.replace_with(move |original, captures| {
            let mut result = template.clone();

            // Replace capture group references: $1, $2, etc.
            for i in 0..captures.len() {
                if let Some(m) = captures.get(i) {
                    let placeholder = format!("${}", i);
                    result = result.replace(&placeholder, m.as_str());
                }
            }

            // If no replacements occurred, use original
            if result == template {
                vec![original.to_string()]
            } else {
                vec![result]
            }
        }).build()
    }
}

/// A regex-based rule that uses pattern matching and capture groups for corrections.
struct RegexRule {
    name: String,
    command_pattern: Option<Regex>,
    output_pattern: Option<Regex>,
    replacement_fn: Box<dyn Fn(&str, &regex::Captures) -> Vec<String> + Send + Sync>,
    priority: i32,
}

impl Rule for RegexRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, command: &Command) -> bool {
        let cmd_matches = if let Some(ref pattern) = self.command_pattern {
            pattern.is_match(&command.script)
        } else {
            true
        };

        let out_matches = if let Some(ref pattern) = self.output_pattern {
            pattern.is_match(&command.output)
        } else {
            true
        };

        cmd_matches && out_matches
    }

    fn get_new_commands(&self, command: &Command) -> Vec<String> {
        if let Some(ref pattern) = self.command_pattern {
            if let Some(captures) = pattern.captures(&command.script) {
                return (self.replacement_fn)(&command.script, &captures);
            }
        }

        // If no command pattern, try output pattern
        if let Some(ref pattern) = self.output_pattern {
            if let Some(captures) = pattern.captures(&command.output) {
                return (self.replacement_fn)(&command.script, &captures);
            }
        }

        Vec::new()
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_rule_builder_creation() {
        let builder = RegexRuleBuilder::new("test");
        assert_eq!(builder.priority, 1000);
    }

    #[test]
    fn test_regex_rule_command_pattern() {
        let rule = RegexRuleBuilder::new("git_push")
            .match_command_regex(r"git push$")
            .unwrap()
            .replace_simple("git push -u origin main")
            .unwrap();

        let cmd = Command {
            script: "git push".to_string(),
            output: "Everything up-to-date".to_string(),
            exit_code: 0,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
    }

    #[test]
    fn test_regex_rule_with_captures() {
        let rule = RegexRuleBuilder::new("git_branch")
            .match_command_regex(r"git push ([a-z]+)$")
            .unwrap()
            .replace_with(|_original, captures| {
                if let Some(branch) = captures.get(1) {
                    vec![format!("git push -u origin {}", branch.as_str())]
                } else {
                    vec![]
                }
            })
            .build()
            .unwrap();

        let cmd = Command {
            script: "git push main".to_string(),
            output: "".to_string(),
            exit_code: 0,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert_eq!(corrections[0], "git push -u origin main");
    }

    #[test]
    fn test_regex_rule_output_pattern() {
        let rule = RegexRuleBuilder::new("permission_denied")
            .match_output_regex(r"Permission denied")
            .unwrap()
            .replace_simple("sudo $0")
            .unwrap();

        let cmd = Command {
            script: "apt update".to_string(),
            output: "Permission denied".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_regex_rule_both_patterns() {
        let rule = RegexRuleBuilder::new("full_match")
            .match_command_regex(r"git")
            .unwrap()
            .match_output_regex(r"error")
            .unwrap()
            .replace_simple("echo 'git error'")
            .unwrap();

        let cmd_match = Command {
            script: "git status".to_string(),
            output: "error: not a repository".to_string(),
            exit_code: 1,
        };

        let cmd_no_match_output = Command {
            script: "git status".to_string(),
            output: "On branch main".to_string(),
            exit_code: 0,
        };

        assert!(rule.matches(&cmd_match));
        assert!(!rule.matches(&cmd_no_match_output));
    }

    #[test]
    fn test_regex_rule_missing_pattern_error() {
        let result = RegexRuleBuilder::new("invalid").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_rule_priority() {
        let rule = RegexRuleBuilder::new("test")
            .priority(500)
            .match_command_regex(r"test")
            .unwrap()
            .replace_simple("corrected")
            .unwrap();

        assert_eq!(rule.priority(), 500);
    }
}
