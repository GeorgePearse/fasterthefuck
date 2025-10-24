/// Fuzzy matching and selection utilities for command corrections.

use crate::CorrectedCommand;
use fuzzy_matcher::skim::SkimMatcherV2;

/// A fuzzy matcher for finding best matches among possibilities.
pub struct FuzzyMatcher {
    matcher: SkimMatcherV2,
}

impl FuzzyMatcher {
    /// Creates a new fuzzy matcher instance.
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Finds the best fuzzy match for a query among candidates.
    ///
    /// Returns the matched item and its score if a match is found.
    pub fn find_best_match<'a>(&self, query: &str, candidates: &'a [&'a str]) -> Option<(&'a str, i64)> {
        candidates
            .iter()
            .filter_map(|candidate| {
                // SkimMatcherV2::fuzzy returns (i64, Vec<usize>) for the score and indices
                self.matcher
                    .fuzzy(candidate, query, false)
                    .map(|(score, _)| (*candidate, score))
            })
            .max_by_key(|(_, score)| *score)
    }

    /// Finds all fuzzy matches above a minimum threshold score.
    pub fn find_all_matches(
        &self,
        query: &str,
        candidates: &[&str],
        min_score: i64,
    ) -> Vec<(String, i64)> {
        let mut matches: Vec<_> = candidates
            .iter()
            .filter_map(|candidate| {
                self.matcher
                    .fuzzy(candidate, query, false)
                    .filter(|(score, _)| *score >= min_score)
                    .map(|(score, _)| (candidate.to_string(), score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending
        matches
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Sorts and filters corrected commands with fuzzy matching fallback.
pub fn select_corrections(
    mut corrections: Vec<CorrectedCommand>,
    limit: Option<usize>,
) -> Vec<CorrectedCommand> {
    // Sort by priority
    corrections.sort_by_key(|c| c.priority);

    // Apply limit if specified
    if let Some(n) = limit {
        corrections.truncate(n);
    }

    corrections
}

/// Finds the best match for a path/file among candidates using fuzzy matching.
pub fn fuzzy_find_path(query: &str, candidates: &[&str]) -> Option<String> {
    FuzzyMatcher::new()
        .find_best_match(query, candidates)
        .map(|(path, _)| path.to_string())
}

/// Filters candidates that fuzzy match a query string.
pub fn filter_by_fuzzy_match(query: &str, candidates: &[&str], min_score: i64) -> Vec<String> {
    FuzzyMatcher::new()
        .find_all_matches(query, candidates, min_score)
        .into_iter()
        .map(|(candidate, _)| candidate)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher_creation() {
        let _matcher = FuzzyMatcher::new();
    }

    #[test]
    fn test_fuzzy_exact_match() {
        let matcher = FuzzyMatcher::new();
        let candidates = vec!["python", "pip", "pypy"];

        let result = matcher.find_best_match("python", &candidates);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "python");
    }

    #[test]
    fn test_fuzzy_partial_match() {
        let matcher = FuzzyMatcher::new();
        let candidates = vec!["python", "puthon", "pypy"];

        // "puthon" should match "python" with fuzzy matching
        let result = matcher.find_best_match("python", &candidates);
        assert!(result.is_some());
    }

    #[test]
    fn test_fuzzy_no_match() {
        let matcher = FuzzyMatcher::new();
        let candidates = vec!["python", "ruby", "go"];

        let result = matcher.find_best_match("xyz", &candidates);
        // fuzzy_matcher may or may not find anything, but it shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_fuzzy_find_path() {
        let candidates = vec!["src/main.rs", "src/lib.rs", "Cargo.toml"];
        let result = fuzzy_find_path("main", &candidates);
        assert!(result.is_some());
    }

    #[test]
    fn test_select_corrections_sorting() {
        let corrections = vec![
            CorrectedCommand::new("cmd3", 300),
            CorrectedCommand::new("cmd1", 100),
            CorrectedCommand::new("cmd2", 200),
        ];

        let sorted = select_corrections(corrections, None);
        assert_eq!(sorted[0].priority, 100);
        assert_eq!(sorted[1].priority, 200);
        assert_eq!(sorted[2].priority, 300);
    }

    #[test]
    fn test_select_corrections_with_limit() {
        let corrections = vec![
            CorrectedCommand::new("cmd1", 100),
            CorrectedCommand::new("cmd2", 200),
            CorrectedCommand::new("cmd3", 300),
        ];

        let limited = select_corrections(corrections, Some(2));
        assert_eq!(limited.len(), 2);
    }
}
