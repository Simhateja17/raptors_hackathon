//! Prefix similarity: the shared leading run of elements across sequences.

use crate::core::{
    maximum_length, AlgorithmError, AlgorithmOutput, OutputAlgorithm, PreparedSequence, ScoreMode,
    Sequence,
};

/// Prefix similarity. The output is the common leading elements themselves,
/// not only their count.
///
/// The Python source's default comparator is identity (`_ident`); custom
/// `sim_test` callbacks are not supported, matching this port's rule that
/// arbitrary source-language callbacks do not cross the Rust boundary.
#[derive(Clone, Copy, Debug, Default)]
pub struct Prefix;

impl Prefix {
    pub const fn new() -> Self {
        Self
    }

    pub fn call(&self, sequences: &[PreparedSequence]) -> Sequence {
        let Some(first) = sequences.first() else {
            return Vec::new();
        };
        let min_len = sequences.iter().map(Vec::len).min().unwrap_or(0);
        let mut result = Vec::with_capacity(min_len);
        for index in 0..min_len {
            let element = &first[index];
            let matches = sequences[1..]
                .iter()
                .all(|sequence| &sequence[index] == element);
            if !matches {
                break;
            }
            result.push(element.clone());
        }
        result
    }
}

impl OutputAlgorithm for Prefix {
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
