//! Length distance: the absolute difference between sequence lengths.

use crate::core::{Algorithm, PreparedSequence};

/// Length distance. Only sequence lengths matter; element content is never
/// compared.
#[derive(Clone, Copy, Debug, Default)]
pub struct Length;

impl Length {
    pub const fn new() -> Self {
        Self
    }
}

impl Algorithm for Length {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // The Python source computes `max(lengths) - min(lengths)` directly,
        // with no empty/identical shortcut. `max()`/`min()` raise ValueError
        // for zero sequences there; this port follows the rest of the crate's
        // convention (see `maximum_length`) of treating that vacuous case as
        // zero instead of panicking.
        let mut lengths = sequences.iter().map(Vec::len);
        let Some(first) = lengths.next() else {
            return 0.0;
        };
        let (min, max) = lengths.fold((first, first), |(min, max), length| {
            (min.min(length), max.max(length))
        });
        (max - min) as f64
    }
}
