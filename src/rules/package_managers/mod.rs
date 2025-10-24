/// Package manager rules for apt, brew, npm, pip, etc.
///
/// This module contains rules for common package manager mistakes:
/// - Missing flags (update, upgrade, search)
/// - Wrong command order
/// - Permission issues
/// - Configuration problems

use crate::{Rule, SimpleRuleBuilder};
#[cfg(test)]
use crate::Command;

/// Creates all package manager rules.
pub fn package_manager_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // apt_autoremove: Clean up unused packages
        create_apt_autoremove(),
        // apt_get_search: Search for packages
        create_apt_get_search(),
        // apt_install_builddeps: Install build dependencies
        create_apt_install_builddeps(),
    ]
}

/// apt_autoremove: Suggest autoremove to clean up unused packages
fn create_apt_autoremove() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("apt_autoremove")
        .match_command("apt remove")
        .match_output("WARNING: The following")
        .priority(600)
        .replace("apt remove", "apt autoremove")
}

/// apt_get_search: Use apt-cache search instead of apt search if not available
fn create_apt_get_search() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("apt_get_search")
        .match_command("apt search")
        .match_output("E: Invalid operation")
        .priority(500)
        .replace("apt search", "apt-cache search")
}

/// apt_install_builddeps: Install build dependencies for compilation
fn create_apt_install_builddeps() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("apt_install_builddeps")
        .match_command("apt install")
        .match_output("error: you need to be root")
        .priority(700)
        .replace("apt install", "apt install build-essential")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_autoremove_rule() {
        let rule = create_apt_autoremove();
        assert_eq!(rule.name(), "apt_autoremove");

        let cmd = Command {
            script: "apt remove package_name".to_string(),
            output: "WARNING: The following packages were automatically installed".to_string(),
            exit_code: 0,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_apt_get_search_rule() {
        let rule = create_apt_get_search();
        assert_eq!(rule.name(), "apt_get_search");

        let cmd = Command {
            script: "apt search keyword".to_string(),
            output: "E: Invalid operation search".to_string(),
            exit_code: 100,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("apt-cache"));
    }

    #[test]
    fn test_apt_install_builddeps_rule() {
        let rule = create_apt_install_builddeps();
        assert_eq!(rule.name(), "apt_install_builddeps");

        let cmd = Command {
            script: "apt install some-package".to_string(),
            output: "error: you need to be root".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_package_manager_rules_exist() {
        let rules = package_manager_rules();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_package_manager_rules_have_priorities() {
        let rules = package_manager_rules();
        let priorities: Vec<_> = rules.iter().map(|r| r.priority()).collect();

        assert!(priorities.contains(&600));
        assert!(priorities.contains(&500));
        assert!(priorities.contains(&700));
    }
}
