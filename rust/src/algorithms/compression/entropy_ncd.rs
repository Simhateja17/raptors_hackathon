//! Entropy NCD (Normalized Compression Distance).
//!
//! Source: `textdistance/algorithms/compression_based.py::EntropyNCD`, via
//! `_NCDBase`. Full behavior card:
//! `docs/behavior-cards/manasa/entropy-ncd.md`.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};
use std::collections::HashMap;

/// Entropy NCD. See `docs/behavior-cards/manasa/entropy-ncd.md`.
pub struct EntropyNcd {
    /// Constant added to raw entropy before treating it as a "size".
    pub coef: f64,
    /// Logarithm base for the entropy calculation (bits, by default).
    pub base: f64,
}

impl Default for EntropyNcd {
    fn default() -> Self {
        Self {
            coef: 1.0,
            base: 2.0,
        }
    }
}

impl EntropyNcd {
    /// `EntropyNCD._compress`: Shannon entropy of the element distribution.
    /// Empty input has an empty `Counter`, so the loop never runs and this
    /// returns `0.0` without any division-by-zero risk.
    fn entropy(&self, elements: &[Element]) -> f64 {
        let total = elements.len();
        if total == 0 {
            return 0.0;
        }
        let mut counts: HashMap<&Element, usize> = HashMap::new();
        for element in elements {
            *counts.entry(element).or_insert(0) += 1;
        }
        let total = total as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total;
            entropy -= p * p.log(self.base);
        }
        debug_assert!(entropy >= 0.0, "entropy must be non-negative");
        entropy
    }

    /// `EntropyNCD._get_size`: `coef + entropy`.
    fn get_size(&self, elements: &[Element]) -> f64 {
        self.coef + self.entropy(elements)
    }
}

impl Algorithm for EntropyNcd {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // Source: `_NCDBase.__call__` — `if not sequences: return 0`. No
        // `quick_answer` shortcut anywhere in the NCD family; identical
        // inputs naturally score `0` here because entropy depends only on
        // element *proportions*, which concatenating a sequence with
        // itself does not change — not because of any special-casing.
        if sequences.is_empty() {
            return 0.0;
        }

        // Entropy depends only on the element multiset's proportions, so
        // (like Sqrt NCD) concatenation order can never change the result;
        // permutation enumeration from the source would be redundant here.
        let concatenated: Vec<Element> = sequences.iter().flatten().cloned().collect();
        let concat_len = self.get_size(&concatenated);

        let compressed_lens: Vec<f64> = sequences.iter().map(|s| self.get_size(s)).collect();
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
