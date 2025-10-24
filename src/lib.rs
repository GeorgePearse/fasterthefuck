/// Fasterthefuck: A blazingly fast command correction engine written in Rust.

pub mod error;
pub mod types;
pub mod corrector;
pub mod fuzzy;

pub use error::{Error, Result};
pub use types::{Command, CorrectedCommand, Rule};
pub use corrector::Corrector;
pub use fuzzy::FuzzyMatcher;
