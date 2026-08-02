//! Longest common subsequence output.

use crate::core::{
    maximum_length, AlgorithmError, AlgorithmOutput, OutputAlgorithm, PreparedSequence, ScoreMode,
    Sequence,
};

/// Longest common subsequence. The output is the subsequence itself, not only
/// its length.
#[derive(Clone, Copy, Debug, Default)]
pub struct LCSSeq;

impl LCSSeq {
    pub const fn new() -> Self {
        Self
    }

    fn dynamic(left: &PreparedSequence, right: &PreparedSequence) -> Sequence {
        let mut lengths = vec![vec![0usize; right.len() + 1]; left.len() + 1];
        for (i, left_element) in left.iter().enumerate() {
            for (j, right_element) in right.iter().enumerate() {
                lengths[i + 1][j + 1] = if left_element == right_element {
                    lengths[i][j] + 1
                } else {
                    lengths[i + 1][j].max(lengths[i][j + 1])
                };
            }
        }

        let mut result = Vec::with_capacity(lengths[left.len()][right.len()]);
        let (mut i, mut j) = (left.len(), right.len());
        while i != 0 && j != 0 {
            // The Python implementation deliberately prefers moving up on a
            // tie, then moving left. Keep that choice for deterministic output.
            if lengths[i][j] == lengths[i - 1][j] {
                i -= 1;
            } else if lengths[i][j] == lengths[i][j - 1] {
                j -= 1;
            } else {
                result.push(left[i - 1].clone());
                i -= 1;
                j -= 1;
            }
        }
        result.reverse();
        result
    }

    fn recursive(sequences: &[PreparedSequence]) -> Sequence {
        if sequences.iter().any(Vec::is_empty) {
            return Vec::new();
        }

        let last = sequences[0].last().expect("non-empty sequence").clone();
        if sequences
            .iter()
            .all(|sequence| sequence.last() == Some(&last))
        {
            let trimmed: Vec<PreparedSequence> = sequences
                .iter()
                .map(|sequence| sequence[..sequence.len() - 1].to_vec())
                .collect();
            let mut result = Self::recursive(&trimmed);
            result.push(last);
            return result;
        }

        let mut best = Vec::new();
        for index in 0..sequences.len() {
            let mut trimmed = sequences.to_vec();
            trimmed[index].pop();
            let candidate = Self::recursive(&trimmed);
            // Python's max([candidate, best], key=len) selects candidate on a
            // tie, so later sequence branches win equal-length results.
            if candidate.len() >= best.len() {
                best = candidate;
            }
        }
        best
    }

    pub fn call(&self, sequences: &[PreparedSequence]) -> Sequence {
        match sequences {
            [] => Vec::new(),
            [sequence] => sequence.clone(),
            [left, right] => Self::dynamic(left, right),
            _ => Self::recursive(sequences),
        }
    }
}

impl OutputAlgorithm for LCSSeq {
    fn output(&self, sequences: &[PreparedSequence]) -> Result<AlgorithmOutput, AlgorithmError> {
        Ok(AlgorithmOutput::Sequence(self.call(sequences)))
    }

    fn output_maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        maximum_length(sequences) as f64
    }

    fn output_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
