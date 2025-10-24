/// Rule system for command correction.
///
/// This module provides the infrastructure for defining and managing correction rules.

pub mod builders;
pub mod macros;

pub mod git;
pub mod permissions;
pub mod filesystem;
pub mod package_managers;

pub use builders::{SimpleRuleBuilder, RegexRuleBuilder, FuzzyRuleBuilder};

use crate::{Corrector, Rule};
#[cfg(test)]
use crate::Command;

/// Registry that manages all available rules.
pub struct RuleRegistry {
    pub rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Creates a new empty rule registry.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Adds a rule to the registry.
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Adds multiple rules to the registry.
    pub fn add_rules(&mut self, rules: Vec<Box<dyn Rule>>) {
        self.rules.extend(rules);
    }

    /// Gets the number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Gets a reference to all rules.
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Gets mutable access to rules (for disabling/enabling).
    pub fn rules_mut(&mut self) -> &mut [Box<dyn Rule>] {
        &mut self.rules
    }

    /// Gets all enabled rules.
    pub fn enabled_rules(&self) -> Vec<&Box<dyn Rule>> {
        self.rules
            .iter()
            .filter(|rule| rule.enabled_by_default())
            .collect()
    }

    /// Creates a corrector from this registry.
    pub fn into_corrector(self) -> Corrector {
        Corrector::new(self)
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl From<RuleRegistry> for Corrector {
    fn from(registry: RuleRegistry) -> Self {
        registry.into_corrector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule {
        name: String,
    }

    impl Rule for TestRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn matches(&self, _command: &Command) -> bool {
            false
        }

        fn get_new_commands(&self, _command: &Command) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn test_rule_registry_creation() {
        let registry = RuleRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_rule_registry_add() {
        let mut registry = RuleRegistry::new();
        registry.add_rule(Box::new(TestRule {
            name: "test".to_string(),
        }));

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_rule_registry_multiple() {
        let mut registry = RuleRegistry::new();
        registry.add_rules(vec![
            Box::new(TestRule {
                name: "test1".to_string(),
            }),
            Box::new(TestRule {
                name: "test2".to_string(),
            }),
        ]);

        assert_eq!(registry.len(), 2);
    }
}
