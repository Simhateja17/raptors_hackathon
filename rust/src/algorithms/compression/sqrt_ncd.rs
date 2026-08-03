//! Square-root NCD (Normalized Compression Distance).
//!
//! Source: `textdistance/algorithms/compression_based.py::SqrtNCD`, via
//! `_NCDBase`. Full behavior card: `docs/behavior-cards/manasa/sqrt-ncd.md`.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};
use std::collections::HashMap;

/// `SqrtNCD._get_size`: sum of `sqrt(count)` per distinct element. Not a
/// real compressor — a stand-in "compressed size" estimator with no
/// external dependency.
fn get_size(elements: &[Element]) -> f64 {
    let mut counts: HashMap<&Element, usize> = HashMap::new();
    for element in elements {
        *counts.entry(element).or_insert(0) += 1;
    }
    counts.values().map(|&count| (count as f64).sqrt()).sum()
}

/// Square-root NCD. See `docs/behavior-cards/manasa/sqrt-ncd.md`.
///
/// No constructor fields: q-value handling happens upstream during
/// sequence preparation, matching `_NCDBase.__init__`'s only role in the
/// source (storing `qval` for `_get_sequences`, which this port doesn't
/// need to repeat).
#[derive(Default)]
pub struct SqrtNcd;

impl Algorithm for SqrtNcd {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // Source: `_NCDBase.__call__` — `if not sequences: return 0`. No
        // `quick_answer` shortcut for identical/empty inputs anywhere in
        // the NCD family; the raw formula runs unconditionally otherwise.
        if sequences.is_empty() {
            return 0.0;
        }

        // Source takes `min` over all permutations' concatenated size, but
        // for a purely count-based estimator like this one, concatenation
        // order can never change the resulting element multiset, so every
        // permutat
        // ion yields an identical size — enumerating permutations
        // would be redundant work.
        let concatenated: Vec<&Element> = sequences.iter().flatten().collect();
        let mut counts: HashMap<&Element, usize> = HashMap::new();
        for element in concatenated {
            *counts.entry(element).or_insert(0) += 1;
        }
        let concat_len: f64 = counts.values().map(|&count| (count as f64).sqrt()).sum();

        let compressed_lens: Vec<f64> = sequences.iter().map(|s| get_size(s)).collect();
        let max_len = compressed_lens.iter().cloned().fold(f64::MIN, f64::max);
        if max_len == 0.0 {
            return 0.0;
        }
        let min_len = compressed_lens.iter().cloned().fold(f64::MAX, f64::min);
        let n = sequences.len() as f64;

        (concat_len - min_len * (n - 1.0)) / max_len
    }

    // Source: `_NCDBase.maximum` always returns `1`.
    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
