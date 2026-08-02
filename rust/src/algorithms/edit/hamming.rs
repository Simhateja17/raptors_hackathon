//! Hamming distance.
//!
//! This is the Rust equivalent of `textdistance.algorithms.edit_based.Hamming`.
//! The caller prepares q-values through the shared core contract; this module
//! only performs the aligned comparison.

use std::sync::Arc;

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

/// A Rust representation of Python Hamming's variable-arity `test_func`.
/// `None` represents the fill value used by Python's `zip_longest`.
pub type TestFunc = Arc<dyn Fn(&[Option<&Element>]) -> bool + Send + Sync>;

/// Hamming distance configuration.
pub struct Hamming {
    pub qvalue: QValue,
    pub truncate: bool,
    pub external: bool,
    test_func: TestFunc,
}

impl Default for Hamming {
    fn default() -> Self {
        Self::new(QValue::Elements, false, true)
    }
}

impl Hamming {
    /// Construct the default identity-based Hamming comparator.
    pub fn new(qvalue: QValue, truncate: bool, external: bool) -> Self {
        Self {
            qvalue,
            truncate,
            external,
            test_func: Arc::new(default_test),
        }
    }

    /// Construct from the Python-facing q-value convention.
    pub fn from_python(qval: Option<usize>, truncate: bool, external: bool) -> Self {
        Self::new(QValue::from_python(qval), truncate, external)
    }

    /// Install a Rust comparator corresponding to Python's `test_func`.
    pub fn with_test_func<F>(qvalue: QValue, truncate: bool, external: bool, test_func: F) -> Self
    where
        F: Fn(&[Option<&Element>]) -> bool + Send + Sync + 'static,
    {
        Self {
            qvalue,
            truncate,
            external,
            test_func: Arc::new(test_func),
        }
    }

    fn aligned_difference_count(&self, sequences: &[PreparedSequence]) -> usize {
        let limit = if self.truncate {
            sequences.iter().map(Vec::len).min().unwrap_or(0)
        } else {
            sequences.iter().map(Vec::len).max().unwrap_or(0)
        };

        (0..limit)
            .filter(|&index| {
                let values: Vec<Option<&Element>> = sequences
                    .iter()
                    .map(|sequence| sequence.get(index))
                    .collect();
                !(self.test_func)(&values)
            })
            .count()
    }
}

/// Return a fresh value corresponding to Python's `hamming` singleton.
pub fn hamming() -> Hamming {
    Hamming::default()
}

fn default_test(values: &[Option<&Element>]) -> bool {
    let Some(first) = values.first() else {
        return true;
    };
    values.iter().all(|value| value == first)
}

impl Algorithm for Hamming {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return answer;
        }
        self.aligned_difference_count(sequences) as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
