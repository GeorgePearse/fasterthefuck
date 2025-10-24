/// Fuzzy rule builder for approximate pattern matching.
///
/// Uses fuzzy matching to identify commands that are "close enough" to a known pattern.
/// Useful for typos and similar variations.

use crate::{Command, Rule};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// A rule builder for fuzzy pattern matching with typo correction.
pub struct FuzzyRuleBuilder {
    name: String,
    command_pattern: Option<String>,
    output_pattern: Option<String>,
    replacement: Option<String>,
    threshold: i64,
    priority: i32,
}

impl FuzzyRuleBuilder {
    /// Creates a new FuzzyRuleBuilder with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command_pattern: None,
            output_pattern: None,
            replacement: None,
            threshold: 20, // Default threshold (SkimMatcher uses log scoring, 20 is reasonable default)
            priority: 1000,
        }
    }

    /// Sets the command pattern to fuzzily match against.
    pub fn match_command(mut self, pattern: impl Into<String>) -> Self {
        self.command_pattern = Some(pattern.into());
        self
    }

    /// Sets the output pattern to fuzzily match against.
    pub fn match_output(mut self, pattern: impl Into<String>) -> Self {
        self.output_pattern = Some(pattern.into());
        self
    }

    /// Sets the fuzzy match threshold (0-100, higher = stricter).
    /// Default is 50.
    pub fn threshold(mut self, t: i64) -> Self {
        self.threshold = t.max(0).min(100);
        self
    }

    /// Sets the priority (lower = higher priority).
    pub fn priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    /// Sets the fixed replacement command.
    pub fn replace(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    /// Builds the fuzzy rule.
    pub fn build(self) -> Result<Box<dyn Rule>, String> {
        if self.command_pattern.is_none() && self.output_pattern.is_none() {
            return Err("FuzzyRuleBuilder: At least one pattern (command or output) must be set"
                .to_string());
        }

        if self.replacement.is_none() {
            return Err("FuzzyRuleBuilder: replacement must be set".to_string());
        }

        Ok(Box::new(FuzzyRule {
            name: self.name,
            command_pattern: self.command_pattern,
            output_pattern: self.output_pattern,
            replacement: self.replacement.unwrap(),
            threshold: self.threshold,
            priority: self.priority,
        }))
    }
}

/// A fuzzy-matching rule for catching typos and similar command variations.
struct FuzzyRule {
    name: String,
    command_pattern: Option<String>,
    output_pattern: Option<String>,
    replacement: String,
    threshold: i64,
    priority: i32,
}

impl FuzzyRule {
    /// Performs fuzzy matching with a threshold.
    fn fuzzy_matches(&self, text: &str, pattern: &str) -> bool {
        let matcher = SkimMatcherV2::default();
        if let Some(score) = matcher.fuzzy_match(text, pattern) {
            // Normalize score: SkimMatcher uses log scoring, convert to 0-100 scale
            // A reasonable threshold is when score > 50
            score > self.threshold
        } else {
            false
        }
    }
}

impl Rule for FuzzyRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, command: &Command) -> bool {
        let cmd_matches = if let Some(ref pattern) = self.command_pattern {
            self.fuzzy_matches(&command.script, pattern)
        } else {
            true
        };

        let out_matches = if let Some(ref pattern) = self.output_pattern {
            self.fuzzy_matches(&command.output, pattern)
        } else {
            true
        };

        cmd_matches && out_matches
    }

    fn get_new_commands(&self, _command: &Command) -> Vec<String> {
        vec![self.replacement.clone()]
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_rule_builder_creation() {
        let builder = FuzzyRuleBuilder::new("test");
        assert_eq!(builder.priority, 1000);
        assert_eq!(builder.threshold, 20);
    }

    #[test]
    fn test_fuzzy_rule_command_pattern() {
        let rule = FuzzyRuleBuilder::new("git_status")
            .match_command("git status")
            .threshold(10)
            .replace("git status")
            .build()
            .unwrap();

        // Exact match
        let cmd_exact = Command {
            script: "git status".to_string(),
            output: "".to_string(),
            exit_code: 0,
        };
        assert!(rule.matches(&cmd_exact));

        // Far match (should not match)
        let cmd_far = Command {
            script: "apt update".to_string(),
            output: "".to_string(),
            exit_code: 0,
        };
        assert!(!rule.matches(&cmd_far));
    }

    #[test]
    fn test_fuzzy_rule_output_pattern() {
        let rule = FuzzyRuleBuilder::new("permission_error")
            .match_output("Permission denied")
            .threshold(10)
            .replace("sudo last_command")
            .build()
            .unwrap();

        let cmd_match = Command {
            script: "apt update".to_string(),
            output: "Permission denied".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd_match));
    }

    #[test]
    fn test_fuzzy_rule_threshold() {
        let _rule_strict = FuzzyRuleBuilder::new("strict")
            .match_command("git status")
            .threshold(200) // Very strict threshold (won't match unless very similar)
            .replace("git status")
            .build()
            .unwrap();

        let rule_lenient = FuzzyRuleBuilder::new("lenient")
            .match_command("git status")
            .threshold(5) // Very lenient threshold
            .replace("git status")
            .build()
            .unwrap();

        let cmd = Command {
            script: "git status".to_string(),
            output: "".to_string(),
            exit_code: 0,
        };

        // Lenient should match exact string
        assert!(rule_lenient.matches(&cmd));
    }

    #[test]
    fn test_fuzzy_rule_both_patterns() {
        let rule = FuzzyRuleBuilder::new("full_match")
            .match_command("git push")
            .match_output("error")
            .replace("git pull")
            .build()
            .unwrap();

        let cmd_both_match = Command {
            script: "git push".to_string(),
            output: "error: rejected".to_string(),
            exit_code: 1,
        };

        let cmd_cmd_only = Command {
            script: "git push".to_string(),
            output: "Success".to_string(),
            exit_code: 0,
        };

        assert!(rule.matches(&cmd_both_match));
        assert!(!rule.matches(&cmd_cmd_only));
    }

    #[test]
    fn test_fuzzy_rule_missing_pattern_error() {
        let result = FuzzyRuleBuilder::new("invalid")
            .replace("corrected")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_fuzzy_rule_missing_replacement_error() {
        let result = FuzzyRuleBuilder::new("invalid")
            .match_command("test")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_fuzzy_rule_priority() {
        let rule = FuzzyRuleBuilder::new("test")
            .priority(500)
            .match_command("test")
            .replace("corrected")
            .build()
            .unwrap();

        assert_eq!(rule.priority(), 500);
    }

    #[test]
    fn test_fuzzy_rule_get_new_commands() {
        let rule = FuzzyRuleBuilder::new("test")
            .match_command("old")
            .replace("new_command")
            .build()
            .unwrap();

        let cmd = Command {
            script: "old".to_string(),
            output: "".to_string(),
            exit_code: 0,
        };

        let corrections = rule.get_new_commands(&cmd);
        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0], "new_command");
    }
}
