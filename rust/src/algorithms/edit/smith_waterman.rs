//! Local Smith-Waterman sequence alignment.

use super::needleman_wunsch::{ElementScorer, EqualityScorer};
use crate::core::{all_identical, Algorithm, PreparedSequence, ScoreMode};

/// Local alignment with a linear gap penalty.
#[derive(Clone, Debug)]
pub struct SmithWaterman<S = EqualityScorer> {
    gap_cost: f64,
    scorer: S,
}

impl SmithWaterman<EqualityScorer> {
    pub const fn new() -> Self {
        Self {
            gap_cost: 1.0,
            scorer: EqualityScorer::new(1.0, 0.0),
        }
    }

    pub const fn with_gap_cost(gap_cost: f64) -> Self {
        Self {
            gap_cost,
            scorer: EqualityScorer::new(1.0, 0.0),
        }
    }
}

impl Default for SmithWaterman<EqualityScorer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> SmithWaterman<S> {
    pub fn with_scorer(gap_cost: f64, scorer: S) -> Self {
        Self { gap_cost, scorer }
    }

    pub const fn gap_cost(&self) -> f64 {
        self.gap_cost
    }
}

impl<S: ElementScorer> Algorithm for SmithWaterman<S> {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // Smith-Waterman inherits BaseSimilarity's quick answers. In
        // particular, equal sequences return the minimum length even when a
        // custom scorer would assign a different diagonal value.
        if sequences.len() <= 1 {
            return sequences
                .first()
                .map_or(0.0, |sequence| sequence.len() as f64);
        }
        if all_identical(sequences) {
            return self.maximum(sequences);
        }
        if sequences.iter().any(Vec::is_empty) {
            return 0.0;
        }

        let (left, right) = (&sequences[0], &sequences[1]);
        let mut previous = vec![0.0; right.len() + 1];

        for left_element in left {
            let mut current = vec![0.0; right.len() + 1];
            for (j, right_element) in right.iter().enumerate() {
                let diagonal = previous[j] + self.scorer.score(left_element, right_element);
                let deletion = previous[j + 1] - self.gap_cost;
                let insertion = current[j] - self.gap_cost;
                current[j + 1] = 0.0_f64.max(diagonal).max(deletion).max(insertion);
            }
            previous = current;
        }

        previous[right.len()]
    }

    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        sequences.iter().map(Vec::len).min().unwrap_or(0) as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
