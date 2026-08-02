//! Global Needleman-Wunsch sequence alignment.

use std::collections::HashMap;

use crate::core::{maximum_length, Algorithm, Element, PreparedSequence, ScoreMode};

/// Element-level scoring seam for alignment algorithms.
pub trait ElementScorer {
    fn score(&self, left: &Element, right: &Element) -> f64;
}

/// Equality-based scoring, configurable for match and mismatch values.
#[derive(Clone, Copy, Debug)]
pub struct EqualityScorer {
    pub match_score: f64,
    pub mismatch_score: f64,
}

impl EqualityScorer {
    pub const fn new(match_score: f64, mismatch_score: f64) -> Self {
        Self {
            match_score,
            mismatch_score,
        }
    }
}

impl Default for EqualityScorer {
    fn default() -> Self {
        Self::new(1.0, 0.0)
    }
}

impl ElementScorer for EqualityScorer {
    fn score(&self, left: &Element, right: &Element) -> f64 {
        if left == right {
            self.match_score
        } else {
            self.mismatch_score
        }
    }
}

/// Matrix-backed element scoring with the same lookup rules as Python's
/// `Matrix`: exact pair, optional reversed pair, identity fallback, then the
/// mismatch fallback.
#[derive(Clone, Debug)]
pub struct MatrixScorer {
    scores: HashMap<(Element, Element), f64>,
    mismatch_score: f64,
    match_score: f64,
    symmetric: bool,
}

impl MatrixScorer {
    pub fn new<I>(scores: I, mismatch_score: f64, match_score: f64, symmetric: bool) -> Self
    where
        I: IntoIterator<Item = ((Element, Element), f64)>,
    {
        Self {
            scores: scores.into_iter().collect(),
            mismatch_score,
            match_score,
            symmetric,
        }
    }

    pub fn from_char_scores<I>(
        scores: I,
        mismatch_score: f64,
        match_score: f64,
        symmetric: bool,
    ) -> Self
    where
        I: IntoIterator<Item = ((char, char), f64)>,
    {
        Self::new(
            scores
                .into_iter()
                .map(|((left, right), score)| ((Element::Char(left), Element::Char(right)), score)),
            mismatch_score,
            match_score,
            symmetric,
        )
    }
}

impl ElementScorer for MatrixScorer {
    fn score(&self, left: &Element, right: &Element) -> f64 {
        if let Some(score) = self.scores.get(&(left.clone(), right.clone())) {
            return *score;
        }
        if self.symmetric {
            if let Some(score) = self.scores.get(&(right.clone(), left.clone())) {
                return *score;
            }
        }
        if left == right {
            self.match_score
        } else {
            self.mismatch_score
        }
    }
}

/// Global alignment with linear gap penalties.
#[derive(Clone, Debug)]
pub struct NeedlemanWunsch<S = EqualityScorer> {
    gap_cost: f64,
    scorer: S,
}

impl NeedlemanWunsch<EqualityScorer> {
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

impl Default for NeedlemanWunsch<EqualityScorer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> NeedlemanWunsch<S> {
    pub fn with_scorer(gap_cost: f64, scorer: S) -> Self {
        Self { gap_cost, scorer }
    }

    pub const fn gap_cost(&self) -> f64 {
        self.gap_cost
    }
}

impl<S: ElementScorer> Algorithm for NeedlemanWunsch<S> {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let (left, right) = match sequences {
            [left, right] => (left, right),
            [] | [_] => return 0.0,
            [left, right, ..] => (left, right),
        };

        let mut previous = vec![0.0; right.len() + 1];
        for (j, value) in previous.iter_mut().enumerate() {
            *value = -(j as f64) * self.gap_cost;
        }

        for (i, left_element) in left.iter().enumerate() {
            let mut current = vec![0.0; right.len() + 1];
            current[0] = -((i + 1) as f64) * self.gap_cost;

            for (j, right_element) in right.iter().enumerate() {
                let diagonal = previous[j] + self.scorer.score(left_element, right_element);
                let deletion = previous[j + 1] - self.gap_cost;
                let insertion = current[j] - self.gap_cost;
                current[j + 1] = diagonal.max(deletion).max(insertion);
            }

            previous = current;
        }

        previous[right.len()]
    }

    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        maximum_length(sequences) as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }

    fn distance(&self, sequences: &[PreparedSequence]) -> f64 {
        -self.raw_score(sequences)
    }

    fn similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        self.raw_score(sequences)
    }

    fn normalized_distance(&self, sequences: &[PreparedSequence]) -> f64 {
        let maximum = self.maximum(sequences);
        let minimum = -maximum * self.gap_cost;
        if maximum == 0.0 {
            0.0
        } else {
            (self.distance(sequences) - minimum) / (maximum - minimum)
        }
    }

    fn normalized_similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        let maximum = self.maximum(sequences);
        let minimum = -maximum * self.gap_cost;
        if maximum == 0.0 {
            1.0
        } else {
            (self.similarity(sequences) - minimum) / (maximum * 2.0)
        }
    }
}
