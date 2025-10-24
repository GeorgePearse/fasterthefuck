/// Permission-related command correction rules.
///
/// This module contains rules for common permission-related mistakes:
/// - Missing sudo for privileged operations
/// - File permissions (chmod)
/// - Directory permissions
/// - Ownership changes

use crate::{Rule, SimpleRuleBuilder};
#[cfg(test)]
use crate::Command;

/// Creates all permission-related rules.
pub fn permission_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // sudo_commands: Add sudo when permission denied
        create_sudo_permission_denied(),
        // sudo_apt: Add sudo for package managers
        create_sudo_apt(),
        // chmod_execute: Add execute permission
        create_chmod_execute(),
        // chmod_recursive: Add recursive flag for directories
        create_chmod_recursive(),
    ]
}

/// sudo_permission_denied: Add sudo when permission denied
fn create_sudo_permission_denied() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("sudo_permission_denied")
        .match_output("Permission denied")
        .priority(100)
        .replace("", "sudo ")
}

/// sudo_apt: Add sudo for apt/apt-get operations
fn create_sudo_apt() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("sudo_apt")
        .match_command("apt ")
        .match_output("E: Could not open lock file")
        .priority(200)
        .replace("apt ", "sudo apt ")
}

/// chmod_execute: Add execute permission when file is not executable
fn create_chmod_execute() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("chmod_execute")
        .match_output("Permission denied")
        .match_command(".")
        .priority(300)
        .replace(".", "chmod +x ")
}

/// chmod_recursive: Add recursive flag for directory chmod
fn create_chmod_recursive() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("chmod_recursive")
        .match_command("chmod ")
        .match_output("No such file or directory")
        .priority(400)
        .replace("chmod ", "chmod -R ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_permission_denied_rule() {
        let rule = create_sudo_permission_denied();
        assert_eq!(rule.name(), "sudo_permission_denied");

        let cmd = Command {
            script: "apt update".to_string(),
            output: "E: Could not open lock file /var/lib/apt/lists/lock - open (13: Permission denied)".to_string(),
            exit_code: 100,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_sudo_apt_rule() {
        let rule = create_sudo_apt();
        assert_eq!(rule.name(), "sudo_apt");

        let cmd = Command {
            script: "apt update".to_string(),
            output: "E: Could not open lock file /var/lib/apt/lists/lock".to_string(),
            exit_code: 100,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("sudo"));
    }

    #[test]
    fn test_chmod_execute_rule() {
        let rule = create_chmod_execute();
        assert_eq!(rule.name(), "chmod_execute");

        let cmd = Command {
            script: "./script.sh".to_string(),
            output: "bash: ./script.sh: Permission denied".to_string(),
            exit_code: 126,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_chmod_recursive_rule() {
        let rule = create_chmod_recursive();
        assert_eq!(rule.name(), "chmod_recursive");

        let cmd = Command {
            script: "chmod 755 /path/to/dir".to_string(),
            output: "chmod: cannot access '/path/to/dir': No such file or directory".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-R"));
    }

    #[test]
    fn test_permission_rules_exist() {
        let rules = permission_rules();
        assert_eq!(rules.len(), 4);
    }

    #[test]
    fn test_permission_rules_have_different_names() {
        let rules = permission_rules();
        let names: Vec<_> = rules.iter().map(|r| r.name()).collect();

        // Check that we have at least some unique names
        assert!(names.contains(&"sudo_permission_denied"));
        assert!(names.contains(&"sudo_apt"));
        assert!(names.contains(&"chmod_execute"));
        assert!(names.contains(&"chmod_recursive"));
    }
}
