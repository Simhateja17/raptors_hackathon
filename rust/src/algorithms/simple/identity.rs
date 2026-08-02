//! Identity similarity: 1 if every sequence is exactly equal, else 0.

use crate::core::{Algorithm, PreparedSequence, ScoreMode};

/// Identity similarity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Identity;

impl Identity {
    pub const fn new() -> Self {
        Self
    }

    /// Mirrors `Base._ident`: zero sequences are *not* identical (unlike the
    /// shared `all_identical` helper, which treats zero/one sequences as
    /// identical), while a single sequence is always identical to itself.
    fn ident(sequences: &[PreparedSequence]) -> bool {
        match sequences.split_first() {
            None => false,
            Some((first, rest)) => rest.iter().all(|sequence| sequence == first),
        }
    }
}

impl Algorithm for Identity {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if Self::ident(sequences) {
            1.0
        } else {
            0.0
        }
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
