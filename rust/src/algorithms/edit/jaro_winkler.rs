//! Jaro-Winkler similarity.
//!
//! Source: `textdistance/algorithms/edit_based.py::JaroWinkler`. Reuses the
//! shared matching/transposition core from `jaro`, then layers the prefix
//! boost and optional long-string adjustment on top. Full behavior card:
//! `docs/behavior-cards/manasa/jaro-winkler.md`.

use crate::algorithms::jaro::core_similarity;
use crate::core::{Algorithm, PreparedSequence, ScoreMode};

/// Jaro-Winkler similarity. See `docs/behavior-cards/manasa/jaro-winkler.md`.
pub struct JaroWinkler {
    /// Only meaningful when the core weight exceeds `0.7` and
    /// `min(len(s1), len(s2)) > 4` — see the long-string adjustment below.
    pub long_tolerance: bool,
    /// Weight applied per matching prefix character (up to 4) in the boost.
    pub prefix_weight: f64,
    /// Accepted for API-shape parity. The Rust core has no external-library
    /// path, so this is a no-op.
    pub external: bool,
}

impl Default for JaroWinkler {
    fn default() -> Self {
        Self {
            long_tolerance: false,
            prefix_weight: 0.1,
            external: true,
        }
    }
}

impl Algorithm for JaroWinkler {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        debug_assert_eq!(
            sequences.len(),
            2,
            "Jaro-Winkler compares exactly two sequences"
        );
        let s1 = &sequences[0];
        let s2 = &sequences[1];

        let core = match core_similarity(s1, s2) {
            Some(core) => core,
            None => return 0.0,
        };
        let mut weight = core.weight;

        // Boost only applies once the plain Jaro score is already fairly
        // similar — matches the source's `if weight <= 0.7: return weight`.
        if weight <= 0.7 {
            return weight;
        }

        let min_len = s1.len().min(s2.len());

        // Common prefix, capped at 4 characters.
        let prefix_cap = min_len.min(4);
        let mut prefix_len = 0usize;
        while prefix_len < prefix_cap && s1[prefix_len] == s2[prefix_len] {
            prefix_len += 1;
        }

        if prefix_len > 0 {
            weight += prefix_len as f64 * self.prefix_weight * (1.0 - weight);
        }

        if !self.long_tolerance || min_len <= 4 {
            return weight;
        }
        let common_chars = core.common_chars;
        if common_chars <= prefix_len + 1 || 2 * common_chars < min_len + prefix_len {
            return weight;
        }
        let s1_len = s1.len();
        let s2_len = s2.len();
        let tmp =
            (common_chars - prefix_len - 1) as f64 / (s1_len + s2_len - prefix_len * 2 + 2) as f64;
        weight += (1.0 - weight) * tmp;
        weight
    }

    // Source: `JaroWinkler.maximum` returns the constant `1`, not the
    // longest-sequence length the trait default would compute.
    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
