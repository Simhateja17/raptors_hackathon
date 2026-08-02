//! Restricted and unrestricted Damerau-Levenshtein distance.

use std::collections::HashMap;

use crate::core::{Algorithm, Element, PreparedSequence};

/// Unit-cost edit distance with adjacent transpositions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamerauLevenshtein {
    /// `true` selects the optimal-string-alignment/restricted algorithm.
    /// `false` selects the unrestricted last-seen-position algorithm.
    restricted: bool,
}

impl Default for DamerauLevenshtein {
    fn default() -> Self {
        Self::new()
    }
}

impl DamerauLevenshtein {
    pub const fn new() -> Self {
        Self { restricted: true }
    }

    pub const fn with_restricted(restricted: bool) -> Self {
        Self { restricted }
    }

    pub const fn is_restricted(self) -> bool {
        self.restricted
    }

    fn restricted_distance(left: &PreparedSequence, right: &PreparedSequence) -> usize {
        let left_len = left.len();
        let right_len = right.len();
        let mut distances = vec![vec![0; right_len + 1]; left_len + 1];

        for (i, row) in distances.iter_mut().enumerate() {
            row[0] = i;
        }
        for (j, cell) in distances[0].iter_mut().enumerate() {
            *cell = j;
        }

        for i in 1..=left_len {
            for j in 1..=right_len {
                let substitution =
                    distances[i - 1][j - 1] + usize::from(left[i - 1] != right[j - 1]);
                let insertion = distances[i][j - 1] + 1;
                let deletion = distances[i - 1][j] + 1;
                distances[i][j] = substitution.min(insertion).min(deletion);

                if i > 1 && j > 1 && left[i - 1] == right[j - 2] && left[i - 2] == right[j - 1] {
                    distances[i][j] = distances[i][j].min(distances[i - 2][j - 2] + 1);
                }
            }
        }

        distances[left_len][right_len]
    }

    fn unrestricted_distance(left: &PreparedSequence, right: &PreparedSequence) -> usize {
        let left_len = left.len();
        let right_len = right.len();
        let maximum_distance = left_len + right_len;
        let mut distances = vec![vec![0; right_len + 2]; left_len + 2];

        distances[0][0] = maximum_distance;
        for i in 0..=left_len {
            distances[i + 1][0] = maximum_distance;
            distances[i + 1][1] = i;
        }
        for j in 0..=right_len {
            distances[0][j + 1] = maximum_distance;
            distances[1][j + 1] = j;
        }

        let mut last_seen: HashMap<Element, usize> = HashMap::new();
        for i in 1..=left_len {
            let mut last_match_in_right = 0;
            for j in 1..=right_len {
                let last_match_in_left = last_seen.get(&right[j - 1]).copied().unwrap_or(0);
                let previous_match_in_right = last_match_in_right;
                let substitution_cost = if left[i - 1] == right[j - 1] {
                    last_match_in_right = j;
                    0
                } else {
                    1
                };

                let substitution = distances[i][j] + substitution_cost;
                let insertion = distances[i + 1][j] + 1;
                let deletion = distances[i][j + 1] + 1;
                let transposition = distances[last_match_in_left][previous_match_in_right]
                    + (i - last_match_in_left - 1)
                    + 1
                    + (j - previous_match_in_right - 1);
                distances[i + 1][j + 1] =
                    substitution.min(insertion).min(deletion).min(transposition);
            }
            last_seen.insert(left[i - 1].clone(), i);
        }

        distances[left_len + 1][right_len + 1]
    }
}

impl Algorithm for DamerauLevenshtein {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let distance = match sequences {
            [left, right] => {
                if self.restricted {
                    Self::restricted_distance(left, right)
                } else {
                    Self::unrestricted_distance(left, right)
                }
            }
            [] | [_] => 0,
            [left, right, ..] => {
                if self.restricted {
                    Self::restricted_distance(left, right)
                } else {
                    Self::unrestricted_distance(left, right)
                }
            }
        };
        distance as f64
    }
}
