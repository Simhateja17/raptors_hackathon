//! MLIPNS binary similarity.

use crate::core::{all_identical, Algorithm, PreparedSequence, ScoreMode};

/// Mismatch-limited pattern-neighborhood similarity.
#[derive(Clone, Copy, Debug)]
pub struct MLIPNS {
    threshold: f64,
    maxmismatches: usize,
}

impl MLIPNS {
    pub const fn new() -> Self {
        Self {
            threshold: 0.25,
            maxmismatches: 2,
        }
    }

    pub const fn with_params(threshold: f64, maxmismatches: usize) -> Self {
        Self {
            threshold,
            maxmismatches,
        }
    }

    pub const fn threshold(self) -> f64 {
        self.threshold
    }

    pub const fn maxmismatches(self) -> usize {
        self.maxmismatches
    }

    fn hamming_distance(sequences: &[PreparedSequence]) -> i64 {
        let maximum = sequences.iter().map(Vec::len).max().unwrap_or(0);
        (0..maximum)
            .filter(|&index| {
                let first = sequences.first().and_then(|sequence| sequence.get(index));
                sequences
                    .iter()
                    .skip(1)
                    .any(|sequence| sequence.get(index) != first)
            })
            .count() as i64
    }
}

impl Default for MLIPNS {
    fn default() -> Self {
        Self::new()
    }
}

impl Algorithm for MLIPNS {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // BaseSimilarity's quick-answer order is significant: equal empty
        // inputs are similar, while only one empty side is dissimilar.
        if sequences.len() <= 1 || all_identical(sequences) {
            return 1.0;
        }
        if sequences.iter().any(Vec::is_empty) {
            return 0.0;
        }

        let mut mismatches = 0usize;
        let mut hamming = Self::hamming_distance(sequences);
        let mut maximum_length = sequences.iter().map(Vec::len).max().unwrap_or(0) as i64;

        while mismatches <= self.maxmismatches {
            if maximum_length == 0 {
                return 1.0;
            }
            let mismatch_ratio = 1.0 - (maximum_length - hamming) as f64 / maximum_length as f64;
            if mismatch_ratio <= self.threshold {
                return 1.0;
            }
            mismatches += 1;
            hamming -= 1;
            maximum_length -= 1;
        }

        if maximum_length == 0 {
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
