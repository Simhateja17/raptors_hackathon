//! Global Gotoh alignment with affine gap penalties.

use super::needleman_wunsch::{ElementScorer, EqualityScorer};
use crate::core::{Algorithm, PreparedSequence, ScoreMode};

/// Global alignment with separate gap-open and gap-extension costs.
#[derive(Clone, Debug)]
pub struct Gotoh<S = EqualityScorer> {
    gap_open: f64,
    gap_ext: f64,
    scorer: S,
}

impl Gotoh<EqualityScorer> {
    pub const fn new() -> Self {
        Self {
            gap_open: 1.0,
            gap_ext: 0.4,
            scorer: EqualityScorer::new(1.0, 0.0),
        }
    }

    pub const fn with_gap_costs(gap_open: f64, gap_ext: f64) -> Self {
        Self {
            gap_open,
            gap_ext,
            scorer: EqualityScorer::new(1.0, 0.0),
        }
    }
}

impl Default for Gotoh<EqualityScorer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Gotoh<S> {
    pub fn with_scorer(gap_open: f64, gap_ext: f64, scorer: S) -> Self {
        Self {
            gap_open,
            gap_ext,
            scorer,
        }
    }

    pub const fn gap_open(&self) -> f64 {
        self.gap_open
    }

    pub const fn gap_ext(&self) -> f64 {
        self.gap_ext
    }
}

impl<S: ElementScorer> Algorithm for Gotoh<S> {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let (left, right) = match sequences {
            [left, right] => (left, right),
            [] | [_] => return 0.0,
            [left, right, ..] => (left, right),
        };
        let left_len = left.len();
        let right_len = right.len();
        let negative_infinity = f64::NEG_INFINITY;

        let mut diagonal = vec![vec![0.0; right_len + 1]; left_len + 1];
        let mut gap_left = vec![vec![0.0; right_len + 1]; left_len + 1];
        let mut gap_right = vec![vec![0.0; right_len + 1]; left_len + 1];

        diagonal[0][0] = 0.0;
        gap_left[0][0] = negative_infinity;
        gap_right[0][0] = negative_infinity;

        for i in 1..=left_len {
            diagonal[i][0] = negative_infinity;
            gap_left[i][0] = -self.gap_open - self.gap_ext * (i as f64 - 1.0);
            gap_right[i][0] = negative_infinity;
            if right_len >= 1 {
                gap_right[i][1] = -self.gap_open;
            }
        }
        for j in 1..=right_len {
            diagonal[0][j] = negative_infinity;
            gap_left[0][j] = negative_infinity;
            gap_right[0][j] = -self.gap_open - self.gap_ext * (j as f64 - 1.0);
            if left_len >= 1 {
                gap_left[1][j] = -self.gap_open;
            }
        }

        for (i, left_element) in left.iter().enumerate() {
            let row = i + 1;
            for (j, right_element) in right.iter().enumerate() {
                let column = j + 1;
                let score = self.scorer.score(left_element, right_element);
                diagonal[row][column] = diagonal[row - 1][column - 1]
                    .max(gap_left[row - 1][column - 1])
                    .max(gap_right[row - 1][column - 1])
                    + score;
                gap_left[row][column] = (diagonal[row - 1][column] - self.gap_open)
                    .max(gap_left[row - 1][column] - self.gap_ext);
                gap_right[row][column] = (diagonal[row][column - 1] - self.gap_open)
                    .max(gap_right[row][column - 1] - self.gap_ext);
            }
        }

        diagonal[left_len][right_len]
            .max(gap_left[left_len][right_len])
            .max(gap_right[left_len][right_len])
    }

    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        sequences.iter().map(Vec::len).min().unwrap_or(0) as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }

    fn normalized_distance(&self, sequences: &[PreparedSequence]) -> f64 {
        let maximum = self.maximum(sequences);
        let minimum = -maximum;
        if maximum == 0.0 {
            0.0
        } else {
            (self.distance(sequences) - minimum) / (maximum - minimum)
        }
    }

    fn normalized_similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        let maximum = self.maximum(sequences);
        let minimum = -maximum;
        if maximum == 0.0 {
            1.0
        } else {
            (self.similarity(sequences) - minimum) / (maximum * 2.0)
        }
    }
}
