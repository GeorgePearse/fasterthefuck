/// Macros for generating rule implementations with reduced boilerplate.

/// Simple rule macro for basic string matching and replacement.
///
/// # Example
/// ```ignore
/// simple_string_rule! {
///     name: "git_branch_delete",
///     match_cmd: "git branch -d",
///     match_out: "If you are sure you want to delete it",
///     replace: ("-d" => "-D")
/// }
/// ```
#[macro_export]
macro_rules! simple_string_rule {
    (
        name: $rule_name:ident,
        match_cmd: $match_cmd:expr,
        match_out: $match_out:expr,
        replace: ($old:expr => $new:expr)
    ) => {
        pub struct $rule_name;

        impl $crate::Rule for $rule_name {
            fn name(&self) -> &str {
                stringify!($rule_name)
            }

            fn matches(&self, command: &$crate::Command) -> bool {
                command.script.contains($match_cmd)
                    && command.output.contains($match_out)
            }

            fn get_new_commands(&self, command: &$crate::Command) -> Vec<String> {
                vec![command.script.replace($old, $new)]
            }

            fn priority(&self) -> i32 {
                100
            }
        }
    };
}

/// Rule macro for commands that should never match on empty output.
#[macro_export]
macro_rules! simple_string_rule_with_priority {
    (
        name: $rule_name:ident,
        match_cmd: $match_cmd:expr,
        match_out: $match_out:expr,
        replace: ($old:expr => $new:expr),
        priority: $priority:expr
    ) => {
        pub struct $rule_name;

        impl $crate::Rule for $rule_name {
            fn name(&self) -> &str {
                stringify!($rule_name)
            }

            fn matches(&self, command: &$crate::Command) -> bool {
                command.script.contains($match_cmd)
                    && command.output.contains($match_out)
            }

            fn get_new_commands(&self, command: &$crate::Command) -> Vec<String> {
                vec![command.script.replace($old, $new)]
            }

            fn priority(&self) -> i32 {
                $priority
            }
        }
    };
}

/// Macro for registering rules with the registry in one convenient call.
#[macro_export]
macro_rules! register_rules {
    ($registry:expr, $($rule_type:ty),*) => {
        $(
            $registry.add_rule(Box::new(<$rule_type>::default()));
        )*
    };
}

#[cfg(test)]
mod tests {
    use crate::{Command, Rule};

    #[test]
    fn test_simple_string_rule_macro() {
        simple_string_rule! {
            name: TestRule,
            match_cmd: "test",
            match_out: "error",
            replace: ("test" => "test2")
        }

        let rule = TestRule;
        let cmd = Command::new("test command", "error occurred", 1);
        assert!(rule.matches(&cmd));

        let corrections = rule.get_new_commands(&cmd);
        assert!(!corrections.is_empty());
        assert_eq!(corrections[0], "test2 command");
    }

    #[test]
    fn test_simple_string_rule_no_match() {
        simple_string_rule! {
            name: TestRule2,
            match_cmd: "test",
            match_out: "error",
            replace: ("test" => "test2")
        }

        let rule = TestRule2;
        let cmd = Command::new("other command", "no error", 0);
        assert!(!rule.matches(&cmd));
    }
}
