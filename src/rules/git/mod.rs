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

/// Creates all git push/pull operation rules.
pub fn git_push_pull_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // git_push_set_upstream: Set upstream on first push
        create_git_push_set_upstream(),
        // git_pull_rebase: Use rebase for pull
        create_git_pull_rebase(),
        // git_push_force: Handle force push scenarios
        create_git_push_force(),
    ]
}

/// Creates all git staging and commit rules.
pub fn git_staging_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // git_add_all: Add all files when trying to commit
        create_git_add_all(),
        // git_commit_amend: Amend last commit
        create_git_commit_amend(),
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

/// git_push_set_upstream: Set upstream on first push
fn create_git_push_set_upstream() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_push_set_upstream")
        .match_command("git push")
        .match_output("fatal: The current branch")
        .priority(600)
        .replace("git push", "git push -u origin")
}

/// git_pull_rebase: Use rebase when pulling
fn create_git_pull_rebase() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_pull_rebase")
        .match_command("git pull")
        .match_output("Please specify which branch you want to merge with")
        .priority(500)
        .replace("git pull", "git pull --rebase origin")
}

/// git_push_force: Handle rejected pushes with force flag
fn create_git_push_force() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_push_force")
        .match_command("git push")
        .match_output("rejected")
        .priority(700)
        .replace("git push", "git push --force-with-lease")
}

/// git_add_all: Add all files before commit
fn create_git_add_all() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_add_all")
        .match_command("git commit")
        .match_output("fatal: your current branch is behind")
        .priority(800)
        .replace("git commit", "git add -A && git commit")
}

/// git_commit_amend: Amend last commit
fn create_git_commit_amend() -> Box<dyn Rule> {
    SimpleRuleBuilder::new("git_commit_amend")
        .match_command("git commit")
        .match_output("nothing to commit")
        .priority(550)
        .replace("git commit", "git commit --amend --no-edit")
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

    #[test]
    fn test_git_push_set_upstream_rule() {
        let rule = create_git_push_set_upstream();
        assert_eq!(rule.name(), "git_push_set_upstream");

        let cmd = Command {
            script: "git push".to_string(),
            output: "fatal: The current branch main has no upstream branch.".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("-u origin"));
    }

    #[test]
    fn test_git_pull_rebase_rule() {
        let rule = create_git_pull_rebase();
        assert_eq!(rule.name(), "git_pull_rebase");

        let cmd = Command {
            script: "git pull".to_string(),
            output: "Please specify which branch you want to merge with".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("--rebase"));
    }

    #[test]
    fn test_git_push_force_rule() {
        let rule = create_git_push_force();
        assert_eq!(rule.name(), "git_push_force");

        let cmd = Command {
            script: "git push".to_string(),
            output: "error: failed to push some refs to origin\n[rejected]        main -> main (non-fast-forward)".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("--force-with-lease"));
    }

    #[test]
    fn test_git_add_all_rule() {
        let rule = create_git_add_all();
        assert_eq!(rule.name(), "git_add_all");

        let cmd = Command {
            script: "git commit -m 'test'".to_string(),
            output: "fatal: your current branch is behind 'origin/main' by 1 commit".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("git add -A"));
    }

    #[test]
    fn test_git_commit_amend_rule() {
        let rule = create_git_commit_amend();
        assert_eq!(rule.name(), "git_commit_amend");

        let cmd = Command {
            script: "git commit --allow-empty".to_string(),
            output: "On branch main\nnothing to commit, working tree clean".to_string(),
            exit_code: 1,
        };

        assert!(rule.matches(&cmd));
        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert!(corrections[0].contains("--amend"));
    }
}
