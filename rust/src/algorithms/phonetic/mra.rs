//! Western Airlines Surname Match Rating Algorithm (MRA) similarity.
//! https://en.wikipedia.org/wiki/Match_rating_approach
//! https://github.com/Yomguithereal/talisman/blob/master/src/metrics/mra.js

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};

/// Match Rating Approach comparison.
#[derive(Clone, Copy, Debug, Default)]
pub struct MRA;

impl MRA {
    pub const fn new() -> Self {
        Self
    }

    /// Render a prepared character sequence back into text. MRA (like the
    /// source implementation) is defined only over strings.
    fn to_text(sequence: &PreparedSequence) -> String {
        sequence
            .iter()
            .map(|element| match element {
                Element::Char(c) => *c,
                _ => panic!("MRA requires character sequences"),
            })
            .collect()
    }

    /// Reduce a word to its match-rating codex: uppercase, always keep the
    /// first character, drop interior vowels, collapse consecutive
    /// duplicates, and once longer than six characters keep only the first
    /// three and last three characters.
    ///
    /// Uppercasing is applied to the whole string (not char-by-char) so that
    /// Unicode expansions such as `'ß'.to_uppercase() == "SS"` line up with
    /// Python's `str.upper()`, which the original algorithm relies on.
    fn codex(word: &str) -> Vec<char> {
        if word.is_empty() {
            return Vec::new();
        }
        let upper: Vec<char> = word.to_uppercase().chars().collect();
        let mut kept = vec![upper[0]];
        kept.extend(upper[1..].iter().copied().filter(|c| !"AEIOU".contains(*c)));

        let mut deduped: Vec<char> = Vec::with_capacity(kept.len());
        for c in kept {
            if deduped.last() != Some(&c) {
                deduped.push(c);
            }
        }

        if deduped.len() > 6 {
            let mut result = deduped[..3].to_vec();
            result.extend_from_slice(&deduped[deduped.len() - 3..]);
            result
        } else {
            deduped
        }
    }
}

impl Algorithm for MRA {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // Mirrors `if not all(sequences): return 0` in the source: a word's
        // codex is empty iff the original word was empty, so checking the
        // raw inputs here is equivalent to checking the transformed codes.
        if sequences.iter().any(Vec::is_empty) {
            return 0.0;
        }

        let mut codes: Vec<Vec<char>> = sequences
            .iter()
            .map(|sequence| Self::codex(&Self::to_text(sequence)))
            .collect();

        let mut lengths: Vec<usize> = codes.iter().map(Vec::len).collect();
        let count = lengths.len();
        let max_length = lengths.iter().copied().max().unwrap_or(0);
        let min_length = lengths.iter().copied().min().unwrap_or(0);
        if max_length.abs_diff(min_length) > count {
            return 0.0;
        }

        for _ in 0..count {
            let minlen = lengths.iter().copied().min().unwrap_or(0);

            // Positions up to the shortest current code where the sequences
            // disagree - i.e. characters that remain unmatched this round.
            let mut mismatched: Vec<Vec<char>> = vec![Vec::new(); count];
            for position in 0..minlen {
                let first = codes[0][position];
                let all_match = codes.iter().all(|code| code[position] == first);
                if !all_match {
                    for (sequence_index, code) in codes.iter().enumerate() {
                        mismatched[sequence_index].push(code[position]);
                    }
                }
            }

            codes = mismatched
                .into_iter()
                .zip(codes.iter())
                .map(|(mut head, tail)| {
                    head.extend_from_slice(&tail[minlen..]);
                    head
                })
                .collect();
            lengths = codes.iter().map(Vec::len).collect();
        }

        (max_length as isize - lengths.iter().copied().max().unwrap_or(0) as isize) as f64
    }

    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        sequences
            .iter()
            .map(|sequence| Self::codex(&Self::to_text(sequence)).len())
            .max()
            .unwrap_or(0) as f64
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
