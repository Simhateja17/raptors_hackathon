//! StrCmp95 similarity.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};

const SPECIAL_PAIRS: [(char, char); 36] = [
    ('A', 'E'),
    ('A', 'I'),
    ('A', 'O'),
    ('A', 'U'),
    ('B', 'V'),
    ('E', 'I'),
    ('E', 'O'),
    ('E', 'U'),
    ('I', 'O'),
    ('I', 'U'),
    ('O', 'U'),
    ('I', 'Y'),
    ('E', 'Y'),
    ('C', 'G'),
    ('E', 'F'),
    ('W', 'U'),
    ('W', 'V'),
    ('X', 'K'),
    ('S', 'Z'),
    ('X', 'S'),
    ('Q', 'C'),
    ('U', 'V'),
    ('M', 'N'),
    ('L', 'I'),
    ('Q', 'O'),
    ('P', 'R'),
    ('I', 'J'),
    ('2', 'Z'),
    ('5', 'S'),
    ('8', 'B'),
    ('1', 'I'),
    ('1', 'L'),
    ('0', 'O'),
    ('0', 'Q'),
    ('C', 'K'),
    ('G', 'J'),
];

/// The source's optional extra adjustment for long strings.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrCmp95 {
    long_strings: bool,
}

impl StrCmp95 {
    pub const fn new() -> Self {
        Self {
            long_strings: false,
        }
    }

    pub const fn with_long_strings(long_strings: bool) -> Self {
        Self { long_strings }
    }

    pub const fn long_strings(self) -> bool {
        self.long_strings
    }

    fn as_normalized_chars(sequence: &PreparedSequence) -> Vec<char> {
        let mut raw = String::new();
        for element in sequence {
            match element {
                Element::Char(value) => raw.push(*value),
                Element::Text(value) => raw.push_str(value),
                Element::Byte(value) => raw.push(*value as char),
                Element::Integer(value) => raw.push_str(&value.to_string()),
                Element::Boolean(value) => raw.push_str(if *value { "TRUE" } else { "FALSE" }),
                Element::Gram(_) => {}
            }
        }
        raw.trim().to_uppercase().chars().collect()
    }

    fn special_pair(left: char, right: char) -> bool {
        SPECIAL_PAIRS.iter().any(|&(first, second)| {
            (first == left && second == right) || (first == right && second == left)
        })
    }

    fn in_range(value: char) -> bool {
        value as u32 > 0 && (value as u32) < 91
    }

    fn pair_similarity(left: &[char], right: &[char], long_strings: bool) -> f64 {
        if left == right {
            return 1.0;
        }
        if left.is_empty() || right.is_empty() {
            return 0.0;
        }

        let left_len = left.len();
        let right_len = right.len();
        let min_length = left_len.min(right_len);
        let initial_range = left_len.max(right_len);
        let mut left_flags = vec![false; initial_range];
        let mut right_flags = vec![false; initial_range];
        let search_range = initial_range.saturating_div(2).saturating_sub(1);

        let mut common = 0usize;
        let right_last = right_len - 1;
        for (i, left_char) in left.iter().enumerate() {
            let low = i.saturating_sub(search_range);
            let high = (i + search_range).min(right_last);
            for j in low..=high {
                if !right_flags[j] && right[j] == *left_char {
                    right_flags[j] = true;
                    left_flags[i] = true;
                    common += 1;
                    break;
                }
            }
        }

        if common == 0 {
            return 0.0;
        }

        let mut next_right = 0usize;
        let mut transpositions = 0usize;
        for (i, left_char) in left.iter().enumerate() {
            if !left_flags[i] {
                continue;
            }
            let matched_right = (next_right..right_len)
                .find(|&j| right_flags[j])
                .expect("every matched left element has a matched right element");
            next_right = matched_right + 1;
            if *left_char != right[matched_right] {
                transpositions += 1;
            }
        }
        transpositions /= 2;

        let mut special_similarity = 0usize;
        if min_length > common {
            for (i, left_char) in left.iter().enumerate() {
                if left_flags[i] || !Self::in_range(*left_char) {
                    continue;
                }
                if let Some(j) = (0..right_len).find(|&j| {
                    !right_flags[j]
                        && Self::in_range(right[j])
                        && Self::special_pair(*left_char, right[j])
                }) {
                    special_similarity += 3;
                    right_flags[j] = true;
                }
            }
        }

        let common_with_special = common as f64 + special_similarity as f64 / 10.0;
        let mut weight = common_with_special / left_len as f64
            + common_with_special / right_len as f64
            + (common - transpositions) as f64 / common as f64;
        weight /= 3.0;

        if weight <= 0.7 {
            return weight;
        }

        let prefix_limit = min_length.min(4);
        let mut prefix = 0usize;
        for (left_char, right_char) in left.iter().zip(right.iter()).take(prefix_limit) {
            if *left_char != *right_char || left_char.is_numeric() {
                break;
            }
            prefix += 1;
        }
        if prefix != 0 {
            weight += prefix as f64 * 0.1 * (1.0 - weight);
        }

        if !long_strings
            || min_length <= 4
            || common <= prefix + 1
            || 2 * common < min_length + prefix
            || left[0].is_numeric()
        {
            return weight;
        }

        let remaining =
            (common - prefix - 1) as f64 / (left_len + right_len - prefix * 2 + 2) as f64;
        weight + (1.0 - weight) * remaining
    }
}

impl Algorithm for StrCmp95 {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let (left, right) = match sequences {
            [left, right] => (left, right),
            [] | [_] => return 1.0,
            [left, right, ..] => (left, right),
        };
        let left = Self::as_normalized_chars(left);
        let right = Self::as_normalized_chars(right);
        Self::pair_similarity(&left, &right, self.long_strings)
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
