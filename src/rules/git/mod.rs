/// Git-related command correction rules.
///
/// This module contains rules for common git mistakes including:
/// - Branch operations (delete, rename, create)
/// - Push/pull operations
/// - Staging and committing
/// - Rebasing and merging
/// - Typos and similar errors

use crate::{Command, Rule, SimpleRuleBuilder, RegexRuleBuilder};

/// Creates all git branch operation rules.
/// These are simple git branch-related corrections.
pub fn git_branch_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // git_branch_delete: Suggest force delete when normal delete fails
        create_git_branch_delete(),
        // git_branch_exists: Create branch if it doesn't exist
        create_git_branch_exists(),
        // git_branch_0flag: Add missing flag
        create_git_branch_0flag(),
    ]
}

/// git_branch_delete: Try force delete when branch has unmerged commits
fn create_git_branch_delete() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_branch_delete")
        .match_command("git branch -d")
        .match_output("error: The branch")
        .priority(500)
        .replace("git branch -d", "git branch -D")
}

/// git_branch_exists: Create branch if it doesn't exist
fn create_git_branch_exists() -> Box<dyn Rule> {
    RegexRuleBuilder::new("git_branch_exists")
        .match_command_regex(r"git checkout ([a-z_-]+)")
        .unwrap()
        .match_output_regex(r"error: pathspec '([a-z_-]+)' did not match")
        .unwrap()
        .replace_with(|_original, captures| {
            if let Some(branch) = captures.get(1) {
                vec![format!("git checkout -b {}", branch.as_str())]
            } else {
                vec![]
            }
        })
        .build()
        .unwrap()
}

/// git_branch_0flag: Fix missing argument flag
fn create_git_branch_0flag() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_branch_0flag")
        .match_command("git branch")
        .match_output("fatal: bad revision")
        .priority(400)
        .replace("git branch", "git branch -a")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_branch_delete_rule() {
        let rule = create_git_branch_delete();
        assert_eq!(rule.name(), "git_branch_delete");

        let cmd = Command {
            script: "git branch -d feature".to_string(),
            output: "error: The branch 'feature' is not fully merged.".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-D"));
    }

    #[test]
    fn test_git_branch_delete_no_match() {
        let rule = create_git_branch_delete();

        let cmd = Command {
            script: "git branch -d main".to_string(),
            output: "Deleted branch main".to_string(),
            exit_code: 0,
        };

        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_branch_exists_rule() {
        let rule = create_git_branch_exists();
        assert_eq!(rule.name(), "git_branch_exists");

        let cmd = Command {
            script: "git checkout feature".to_string(),
            output: "error: pathspec 'feature' did not match any file(s) known to git".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("checkout -b"));
    }

    #[test]
    fn test_git_branch_0flag_rule() {
        let rule = create_git_branch_0flag();
        assert_eq!(rule.name(), "git_branch_0flag");

        let cmd = Command {
            script: "git branch".to_string(),
            output: "fatal: bad revision ''".to_string(),
            exit_code: 128,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-a"));
    }
}
