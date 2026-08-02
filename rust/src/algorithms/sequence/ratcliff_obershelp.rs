//! Ratcliff-Obershelp similarity (Gestalt Pattern Matching).
//!
//! <https://en.wikipedia.org/wiki/Gestalt_Pattern_Matching>

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};

/// Ratcliff-Obershelp similarity.
///
/// Finds the longest common substring shared by all sequences, then
/// recurses on the parts to its left and right, summing the matched
/// lengths. The result is `sequence_count * matched / element_count`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RatcliffObershelp;

impl RatcliffObershelp {
    pub const fn new() -> Self {
        Self
    }

    /// Longest common substring across all sequences.
    ///
    /// The source library dispatches on the same condition: two sequences
    /// shorter than 200 elements use `difflib.SequenceMatcher` (`standard_match`
    /// below); everything else falls back to an n-gram scan seeded by the
    /// shortest sequence (`ngram_match`). The two strategies can pick a
    /// different substring among equal-length ties, so the branch matters for
    /// exact parity, not just for performance.
    fn longest_common_substring(sequences: &[PreparedSequence]) -> PreparedSequence {
        if sequences.iter().any(Vec::is_empty) {
            return Vec::new();
        }
        if let [left, right] = sequences {
            if left.len().max(right.len()) < 200 {
                return Self::standard_match(left, right);
            }
        }
        Self::ngram_match(sequences)
    }

    /// Longest common substring of two sequences, replicating
    /// `difflib.SequenceMatcher.find_longest_match`'s tie-break: earliest
    /// start in `left`, then earliest start in `right`. With no junk
    /// elements in play (guaranteed by the caller's under-200 guard), a
    /// classic DP with a strict `>` update produces the identical match.
    fn standard_match(left: &PreparedSequence, right: &PreparedSequence) -> PreparedSequence {
        let mut previous_row = vec![0usize; right.len() + 1];
        let (mut best_start, mut best_len) = (0usize, 0usize);
        for (i, left_element) in left.iter().enumerate() {
            let mut current_row = vec![0usize; right.len() + 1];
            for (j, right_element) in right.iter().enumerate() {
                if left_element == right_element {
                    let length = previous_row[j] + 1;
                    current_row[j + 1] = length;
                    if length > best_len {
                        best_len = length;
                        best_start = i + 1 - length;
                    }
                }
            }
            previous_row = current_row;
        }
        left[best_start..best_start + best_len].to_vec()
    }

    /// Longest common substring across two or more sequences, mirroring the
    /// source library's n-gram fallback: scan window sizes from the shortest
    /// sequence's length down to one, and within a size, take the first
    /// (leftmost) window of the shortest sequence that occurs in every
    /// sequence.
    fn ngram_match(sequences: &[PreparedSequence]) -> PreparedSequence {
        let shortest = sequences
            .iter()
            .min_by_key(|sequence| sequence.len())
            .expect("at least one sequence");

        for window_len in (1..=shortest.len()).rev() {
            for start in 0..=(shortest.len() - window_len) {
                let window = &shortest[start..start + window_len];
                if sequences
                    .iter()
                    .all(|sequence| contains_window(sequence, window))
                {
                    return window.to_vec();
                }
            }
        }
        Vec::new()
    }

    /// Mirrors `RatcliffObershelp._find`: the longest common substring's
    /// length plus the matched lengths recursed on the parts before and
    /// after it in every sequence.
    fn matched_length(sequences: &[PreparedSequence]) -> usize {
        let subseq = Self::longest_common_substring(sequences);
        let length = subseq.len();
        if length == 0 {
            return 0;
        }

        let mut before = Vec::with_capacity(sequences.len());
        let mut after = Vec::with_capacity(sequences.len());
        for sequence in sequences {
            let position = first_window_position(sequence, &subseq)
                .expect("the longest common substring must occur in every sequence");
            before.push(sequence[..position].to_vec());
            after.push(sequence[position + length..].to_vec());
        }

        Self::matched_length(&before) + length + Self::matched_length(&after)
    }
}

fn contains_window(sequence: &PreparedSequence, window: &[Element]) -> bool {
    sequence
        .windows(window.len())
        .any(|candidate| candidate == window)
}

fn first_window_position(sequence: &PreparedSequence, window: &[Element]) -> Option<usize> {
    sequence
        .windows(window.len())
        .position(|candidate| candidate == window)
}

impl Algorithm for RatcliffObershelp {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return answer;
        }

        let sequence_count = sequences.len();
        let element_count: usize = sequences.iter().map(Vec::len).sum();
        let matched = Self::matched_length(sequences);
        (sequence_count * matched) as f64 / element_count as f64
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
