/// Rule builders for creating rules with fluent API.

pub mod simple;
pub mod regex;
pub mod fuzzy;

pub use simple::SimpleRuleBuilder;
pub use regex::RegexRuleBuilder;
pub use fuzzy::FuzzyRuleBuilder;
