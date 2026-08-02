//! Editex phonetic edit distance.
//!
//! Source: `textdistance/algorithms/phonetic.py::Editex`. Full behavior
//! card: `docs/behavior-cards/manasa/editex.md`.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};

const GROUPS: [&[char]; 10] = [
    &['A', 'E', 'I', 'O', 'U', 'Y'],
    &['B', 'P'],
    &['C', 'K', 'Q'],
    &['D', 'T'],
    &['L', 'R'],
    &['M', 'N'],
    &['G', 'J'],
    &['F', 'P', 'V'],
    &['S', 'X', 'Z'],
    &['C', 'S', 'Z'],
];
const UNGROUPED: [char; 2] = ['H', 'W'];

fn in_grouped(c: char) -> bool {
    GROUPS.iter().any(|group| group.contains(&c))
}

fn share_a_group(a: char, b: char) -> bool {
    GROUPS
        .iter()
        .any(|group| group.contains(&a) && group.contains(&b))
}

/// Editex phonetic edit distance. See `docs/behavior-cards/manasa/editex.md`.
pub struct Editex {
    pub local: bool,
    match_cost: i64,
    group_cost: i64,
    mismatch_cost: i64,
    pub external: bool,
}

impl Editex {
    /// Mirrors the source constructor's clamping:
    /// `match_cost <= group_cost <= mismatch_cost` always holds, even if
    /// constructed with an inconsistent ordering.
    pub fn new(
        local: bool,
        match_cost: i64,
        group_cost: i64,
        mismatch_cost: i64,
        external: bool,
    ) -> Self {
        let group_cost = group_cost.max(match_cost);
        let mismatch_cost = mismatch_cost.max(group_cost);
        Self {
            local,
            match_cost,
            group_cost,
            mismatch_cost,
            external,
        }
    }

    fn r_cost(&self, a: char, b: char) -> i64 {
        if a == b {
            return self.match_cost;
        }
        if !in_grouped(a) || !in_grouped(b) {
            return self.mismatch_cost;
        }
        if share_a_group(a, b) {
            return self.group_cost;
        }
        self.mismatch_cost
    }

    fn d_cost(&self, a: char, b: char) -> i64 {
        if a != b && UNGROUPED.contains(&a) {
            return self.group_cost;
        }
        self.r_cost(a, b)
    }

    /// Uppercase-expand a prepared sequence into a space-prefixed `char`
    /// vector, matching `' ' + s.upper()` in the source. Some code points
    /// expand to more than one character when uppercased (e.g. German
    /// `ß` -> `SS`), so the padded length can exceed `elements.len()`.
    fn padded_upper(elements: &[Element]) -> Vec<char> {
        let mut padded = vec![' '];
        for element in elements {
            let ch = match element {
                Element::Char(c) => *c,
                other => panic!("Editex only supports character sequences, got {other:?}"),
            };
            padded.extend(ch.to_uppercase());
        }
        padded
    }
}

impl Default for Editex {
    fn default() -> Self {
        Self::new(false, 0, 1, 2, true)
    }
}

impl Algorithm for Editex {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        debug_assert_eq!(sequences.len(), 2, "Editex compares exactly two sequences");
        let s1 = &sequences[0];
        let s2 = &sequences[1];

        // Source: `Editex.__call__` calls `self.quick_answer(s1, s2)`
        // first (inherited from `Base`, not overridden). This is not
        // optional: without it, the DP below can return a *smaller* value
        // than the source for empty-input cases, because phonetically
        // related adjacent letters (e.g. 'E'/'I', both in the AEIOUY
        // group) are cheaper than plain mismatch cost — verified against
        // '' vs 'neilsen': natural DP gives 13, but the frozen expected
        // value (and this shortcut) gives 14.
        if s1 == s2 {
            return 0.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return self.maximum(sequences);
        }

        let padded_s1 = Self::padded_upper(s1);
        let padded_s2 = Self::padded_upper(s2);
        let len_s1 = padded_s1.len() - 1;
        let len_s2 = padded_s2.len() - 1;

        let mut d_mat = vec![vec![0i64; len_s2 + 1]; len_s1 + 1];

        if !self.local {
            for i in 1..=len_s1 {
                d_mat[i][0] = d_mat[i - 1][0] + self.d_cost(padded_s1[i - 1], padded_s1[i]);
            }
        }
        for j in 1..=len_s2 {
            d_mat[0][j] = d_mat[0][j - 1] + self.d_cost(padded_s2[j - 1], padded_s2[j]);
        }

        for i in 1..=len_s1 {
            for j in 1..=len_s2 {
                let delete = d_mat[i - 1][j] + self.d_cost(padded_s1[i - 1], padded_s1[i]);
                let insert = d_mat[i][j - 1] + self.d_cost(padded_s2[j - 1], padded_s2[j]);
                let substitute = d_mat[i - 1][j - 1] + self.r_cost(padded_s1[i], padded_s2[j]);
                d_mat[i][j] = delete.min(insert).min(substitute);
            }
        }

        (d_mat[len_s1][len_s2] as f64).min(self.maximum(sequences))
    }

    // Source: `Editex.maximum` = `max(len(s1), len(s2)) * mismatch_cost`,
    // computed from the *original* (pre-uppercase) sequence lengths.
    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        let longest = sequences.iter().map(Vec::len).max().unwrap_or(0);
        longest as f64 * self.mismatch_cost as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
