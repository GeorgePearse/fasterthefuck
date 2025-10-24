/// Error types for fasterthefuck.

use std::io;
use thiserror::Error;

/// Result type alias using our custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type for fasterthefuck.
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rule error: {0}")]
    Rule(String),

    #[error("Shell error: {0}")]
    Shell(String),

    #[error("Fuzzy matching error: {0}")]
    FuzzyMatch(String),

    #[error("Command parsing error: {0}")]
    CommandParse(String),

    #[error("No corrections found")]
    NoCorrections,

    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Creates a new configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Creates a new rule error.
    pub fn rule(msg: impl Into<String>) -> Self {
        Error::Rule(msg.into())
    }

    /// Creates a new shell error.
    pub fn shell(msg: impl Into<String>) -> Self {
        Error::Shell(msg.into())
    }

    /// Creates a new fuzzy matching error.
    pub fn fuzzy_match(msg: impl Into<String>) -> Self {
        Error::FuzzyMatch(msg.into())
    }

    /// Creates a new command parsing error.
    pub fn command_parse(msg: impl Into<String>) -> Self {
        Error::CommandParse(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test config error");
        assert_eq!(err.to_string(), "Configuration error: test config error");
    }

    #[test]
    fn test_result_type() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
    }
}
