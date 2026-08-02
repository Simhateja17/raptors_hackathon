//! Unit-cost Levenshtein distance.
//!
//! This packet operates on prepared Rust sequences. Input preparation, q-value
//! handling, and the Python boundary are intentionally kept outside the
//! algorithm implementation.

use crate::core::{Algorithm, PreparedSequence};

/// Unit-cost insertion, deletion, and substitution distance.
#[derive(Clone, Copy, Debug, Default)]
pub struct Levenshtein;

impl Levenshtein {
    pub const fn new() -> Self {
        Self
    }

    /// Compute the distance for one pair of prepared sequences.
    fn pair_distance(left: &PreparedSequence, right: &PreparedSequence) -> usize {
        // Keep only the previous and current rows of the dynamic-programming
        // matrix. The sequence elements are compared directly, so Unicode
        // strings are compared by Rust `char` values rather than UTF-8 bytes.
        let mut previous: Vec<usize> = (0..=right.len()).collect();

        for (left_index, left_element) in left.iter().enumerate() {
            let mut current = vec![0; right.len() + 1];
            current[0] = left_index + 1;

            for (right_index, right_element) in right.iter().enumerate() {
                let substitution =
                    previous[right_index] + usize::from(left_element != right_element);
                let insertion = current[right_index] + 1;
                let deletion = previous[right_index + 1] + 1;
                current[right_index + 1] = substitution.min(insertion).min(deletion);
            }

            previous = current;
        }

        previous[right.len()]
    }
}

impl Algorithm for Levenshtein {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // The source API is pairwise. The adapter validates arity; keeping a
        // total implementation here makes the shared native trait safe for
        // its zero/one-sequence contract as well.
        match sequences {
            [left, right] => Self::pair_distance(left, right) as f64,
            [] | [_] => 0.0,
            [left, right, ..] => Self::pair_distance(left, right) as f64,
        }
    }
}
