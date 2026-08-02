//! Postfix similarity: the shared trailing run of elements across sequences.

use crate::core::{
    maximum_length, AlgorithmError, AlgorithmOutput, OutputAlgorithm, PreparedSequence, ScoreMode,
    Sequence,
};

/// Postfix similarity. The output is the common trailing elements themselves,
/// not only their count.
///
/// The Python source implements this as `Postfix(Prefix)`: it reverses every
/// sequence, delegates to `Prefix.__call__`, and reverses the result back.
/// The Rust port mirrors that "reverse, find common prefix, reverse back"
/// shape directly rather than allocating reversed copies through a shared
/// `Prefix` call, since `Prefix::call` already assumes forward iteration.
///
/// The Python source's default comparator is identity (`_ident`); custom
/// `sim_test` callbacks are not supported, matching this port's rule that
/// arbitrary source-language callbacks do not cross the Rust boundary.
#[derive(Clone, Copy, Debug, Default)]
pub struct Postfix;

impl Postfix {
    pub const fn new() -> Self {
        Self
    }

    pub fn call(&self, sequences: &[PreparedSequence]) -> Sequence {
        let Some(first) = sequences.first() else {
            return Vec::new();
        };
        let min_len = sequences.iter().map(Vec::len).min().unwrap_or(0);
        let mut result = Vec::with_capacity(min_len);
        for offset in 1..=min_len {
            let element = &first[first.len() - offset];
            let matches = sequences[1..]
                .iter()
                .all(|sequence| &sequence[sequence.len() - offset] == element);
            if !matches {
                break;
            }
            result.push(element.clone());
        }
        result.reverse();
        result
    }
}

impl OutputAlgorithm for Postfix {
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
