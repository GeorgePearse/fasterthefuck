/// Rule evaluation and command correction engine with parallel processing.

use crate::{Command, CorrectedCommand, Rule};
use rayon::prelude::*;

/// Registry that holds all available rules.
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Creates a new empty rule registry.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a rule to the registry.
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Gets the number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Gets all enabled rules.
    pub fn enabled_rules(&self) -> Vec<&Box<dyn Rule>> {
        self.rules
            .iter()
            .filter(|rule| rule.enabled_by_default())
            .collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The command correction engine.
pub struct Corrector {
    registry: RuleRegistry,
}

impl Corrector {
    /// Creates a new corrector with an empty rule registry.
    pub fn new(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    /// Gets all enabled rules.
    pub fn rules(&self) -> Vec<&Box<dyn Rule>> {
        self.registry.enabled_rules()
    }

    /// Finds and returns all corrections for a command, sorted by priority.
    ///
    /// This uses parallel evaluation via Rayon for performance.
    pub fn get_corrections(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Parallel rule matching and correction
        let mut corrections: Vec<CorrectedCommand> = self
            .registry
            .rules
            .par_iter()
            .filter(|rule| {
                // Skip rules that require output but command has no output
                if rule.requires_output() && command.output.is_empty() {
                    return false;
                }
                rule.matches(command)
            })
            .flat_map(|rule| rule.get_corrected_commands(command))
            .collect();

        // Sort by priority and remove duplicates
        corrections.sort();
        corrections.dedup();

        corrections
    }

    /// Gets the best (highest priority) correction for a command.
    pub fn get_best_correction(&self, command: &Command) -> Option<CorrectedCommand> {
        self.get_corrections(command).into_iter().next()
    }
}

impl Default for Corrector {
    fn default() -> Self {
        Self::new(RuleRegistry::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test rule for testing purposes.
    struct TestRule {
        name: String,
        matches_result: bool,
        suggestions: Vec<String>,
    }

    impl TestRule {
        fn new(name: &str, matches_result: bool, suggestions: Vec<String>) -> Self {
            Self {
                name: name.to_string(),
                matches_result,
                suggestions,
            }
        }
    }

    impl Rule for TestRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn matches(&self, _command: &Command) -> bool {
            self.matches_result
        }

        fn get_new_commands(&self, _command: &Command) -> Vec<String> {
            self.suggestions.clone()
        }
    }

    #[test]
    fn test_rule_registry() {
        let mut registry = RuleRegistry::new();
        assert_eq!(registry.len(), 0);

        registry.add_rule(Box::new(TestRule::new("test1", true, vec![])));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_corrector_basic() {
        let mut registry = RuleRegistry::new();
        registry.add_rule(Box::new(TestRule::new(
            "test",
            true,
            vec!["corrected".to_string()],
        )));

        let corrector = Corrector::new(registry);
        let cmd = Command::new("test", "error", 1);
        let corrections = corrector.get_corrections(&cmd);

        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0].script, "corrected");
    }

    #[test]
    fn test_corrector_no_match() {
        let registry = RuleRegistry::new();
        let corrector = Corrector::new(registry);
        let cmd = Command::new("test", "error", 1);
        let corrections = corrector.get_corrections(&cmd);

        assert!(corrections.is_empty());
    }

    #[test]
    fn test_corrector_multiple_rules() {
        let mut registry = RuleRegistry::new();
        registry.add_rule(Box::new(TestRule::new(
            "rule1",
            true,
            vec!["correction1".to_string()],
        )));
        registry.add_rule(Box::new(TestRule::new(
            "rule2",
            true,
            vec!["correction2".to_string()],
        )));

        let corrector = Corrector::new(registry);
        let cmd = Command::new("test", "error", 1);
        let corrections = corrector.get_corrections(&cmd);

        assert_eq!(corrections.len(), 2);
    }

    #[test]
    fn test_rule_requires_output() {
        let mut registry = RuleRegistry::new();
        registry.add_rule(Box::new(TestRule::new(
            "test",
            true,
            vec!["corrected".to_string()],
        )));

        let corrector = Corrector::new(registry);
        let cmd = Command::new("test", "", 1); // No output
        let corrections = corrector.get_corrections(&cmd);

        // Should NOT match because rule requires output but command has none
        // (Our TestRule defaults to requires_output() = true)
        assert_eq!(corrections.len(), 0);
    }
}
