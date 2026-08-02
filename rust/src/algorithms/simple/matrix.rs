//! Matrix similarity: a lookup-table (substitution matrix) comparator.
//!
//! Unlike every other algorithm in this crate, the Python source does not
//! prepare its inputs with `qval` at all: it compares the raw sequences
//! given to it as opaque, whole units. In this port each `PreparedSequence`
//! stands in for one such whole, untouched input (callers should prepare
//! with `QValue::Elements` to mirror the source's lack of splitting).

use std::collections::BTreeMap;

use crate::core::{Algorithm, PreparedSequence, ScoreMode};

/// Matrix similarity configuration.
pub struct Matrix {
    /// Lookup table keyed by the exact sequence-of-sequences to compare.
    /// `None` (or an empty map, matching Python's `not self.mat` check)
    /// falls back to a plain identity comparison.
    pub mat: Option<BTreeMap<Vec<PreparedSequence>, f64>>,
    pub mismatch_cost: f64,
    pub match_cost: f64,
    pub symmetric: bool,
    pub external: bool,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new(None, 0.0, 1.0, true, true)
    }
}

impl Matrix {
    pub fn new(
        mat: Option<BTreeMap<Vec<PreparedSequence>, f64>>,
        mismatch_cost: f64,
        match_cost: f64,
        symmetric: bool,
        external: bool,
    ) -> Self {
        Self {
            mat,
            mismatch_cost,
            match_cost,
            symmetric,
            external,
        }
    }

    /// Mirrors `Base._ident` as used directly by `Matrix.__call__` (i.e.
    /// bypassing `quick_answer`'s empty-sequence shortcut): zero sequences
    /// are *not* identical, one sequence trivially is, and two or more are
    /// identical only if every one of them is equal.
    fn is_identical(sequences: &[PreparedSequence]) -> bool {
        match sequences.split_first() {
            None => false,
            Some((first, rest)) => rest.iter().all(|sequence| sequence == first),
        }
    }
}

impl Algorithm for Matrix {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let identity_fallback = || {
            if Self::is_identical(sequences) {
                self.match_cost
            } else {
                self.mismatch_cost
            }
        };

        let Some(mat) = self.mat.as_ref().filter(|mat| !mat.is_empty()) else {
            return identity_fallback();
        };

        if let Some(&cost) = mat.get(sequences) {
            return cost;
        }
        if self.symmetric {
            let reversed: Vec<PreparedSequence> = sequences.iter().rev().cloned().collect();
            if let Some(&cost) = mat.get(&reversed) {
                return cost;
            }
        }
        identity_fallback()
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        self.match_cost
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
