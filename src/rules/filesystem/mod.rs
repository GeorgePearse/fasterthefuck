/// File system operation rules.
///
/// This module contains rules for common filesystem mistakes:
/// - File/directory not found
/// - Path issues
/// - Recursive operations
/// - Directory creation

use crate::{Command, Rule, SimpleRuleBuilder};

/// Creates all filesystem operation rules.
pub fn filesystem_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // mkdir_p: Create parent directories
        create_mkdir_p(),
        // rm_recursive: Add recursive flag for directories
        create_rm_recursive(),
        // cp_recursive: Add recursive flag for directories
        create_cp_recursive(),
        // mv_to_directory: Create target directory if it doesn't exist
        create_mv_to_directory(),
    ]
}

/// mkdir_p: Create parent directories with -p flag
fn create_mkdir_p() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("mkdir_p")
        .match_command("mkdir ")
        .match_output("No such file or directory")
        .priority(100)
        .replace("mkdir ", "mkdir -p ")
}

/// rm_recursive: Add recursive flag when trying to remove directory
fn create_rm_recursive() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("rm_recursive")
        .match_command("rm ")
        .match_output("Is a directory")
        .priority(200)
        .replace("rm ", "rm -r ")
}

/// cp_recursive: Add recursive flag when trying to copy directory
fn create_cp_recursive() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("cp_recursive")
        .match_command("cp ")
        .match_output("Is a directory")
        .priority(300)
        .replace("cp ", "cp -r ")
}

/// mv_to_directory: Suggest moving into directory when target doesn't exist
fn create_mv_to_directory() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("mv_to_directory")
        .match_command("mv ")
        .match_output("No such file or directory")
        .priority(400)
        .replace("mv ", "mkdir -p /path && mv ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkdir_p_rule() {
        let rule = create_mkdir_p();
        assert_eq!(rule.name(), "mkdir_p");

        let cmd = Command {
            script: "mkdir a/b/c".to_string(),
            output: "mkdir: cannot create directory 'a/b/c': No such file or directory".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-p"));
    }

    #[test]
    fn test_rm_recursive_rule() {
        let rule = create_rm_recursive();
        assert_eq!(rule.name(), "rm_recursive");

        let cmd = Command {
            script: "rm my_dir".to_string(),
            output: "rm: cannot remove 'my_dir': Is a directory".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-r"));
    }

    #[test]
    fn test_cp_recursive_rule() {
        let rule = create_cp_recursive();
        assert_eq!(rule.name(), "cp_recursive");

        let cmd = Command {
            script: "cp my_dir /backup/".to_string(),
            output: "cp: my_dir is a directory (not copied).Is a directory".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-r"));
    }

    #[test]
    fn test_mv_to_directory_rule() {
        let rule = create_mv_to_directory();
        assert_eq!(rule.name(), "mv_to_directory");

        let cmd = Command {
            script: "mv file.txt backup/file.txt".to_string(),
            output: "mv: cannot move 'file.txt' to 'backup/file.txt': No such file or directory".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_filesystem_rules_exist() {
        let rules = filesystem_rules();
        assert_eq!(rules.len(), 4);
    }

    #[test]
    fn test_filesystem_rules_priorities() {
        let rules = filesystem_rules();
        let priorities: Vec<_> = rules.iter().map(|r| r.priority()).collect();

        // Check that priorities are different
        assert!(priorities.contains(&100));
        assert!(priorities.contains(&200));
        assert!(priorities.contains(&300));
        assert!(priorities.contains(&400));
    }
}
