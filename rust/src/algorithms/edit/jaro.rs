//! Jaro similarity.
//!
//! Source: `textdistance/algorithms/edit_based.py::Jaro`, implemented there
//! as a `JaroWinkler` subclass with `winklerize` forced to `false`. Full
//! behavior card: `docs/behavior-cards/manasa/jaro.md`.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};

/// Result of the shared Jaro matching core, exposing the pieces
/// `jaro_winkler`'s long-tolerance adjustment needs beyond the plain weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct JaroCore {
    /// Jaro similarity in `[0, 1]`, before any Winkler prefix boost.
    pub weight: f64,
    /// Count of matched (not necessarily in-order) characters.
    pub common_chars: usize,
}

/// Shared Jaro matching/transposition core, reused by `jaro_winkler`.
///
/// Mirrors `JaroWinkler.__call__` in the source through the point where
/// `winklerize` starts to matter. Returns `None` when either sequence is
/// empty or no characters matched (both score `0.0` in the source);
/// `Some(JaroCore { .. })` otherwise.
pub(crate) fn core_similarity(s1: &[Element], s2: &[Element]) -> Option<JaroCore> {
    let s1_len = s1.len();
    let s2_len = s2.len();
    if s1_len == 0 || s2_len == 0 {
        return None;
    }

    // `(max_len // 2) - 1`, clamped to zero — matches the source's
    // `if search_range < 0: search_range = 0`.
    let search_range = (s1_len.max(s2_len) / 2).saturating_sub(1);

    let mut s1_flags = vec![false; s1_len];
    let mut s2_flags = vec![false; s2_len];
    let mut common_chars: usize = 0;

    for (i, s1_ch) in s1.iter().enumerate() {
        let low = i.saturating_sub(search_range);
        let high = (i + search_range).min(s2_len.saturating_sub(1));
        for j in low..=high {
            if !s2_flags[j] && s2[j] == *s1_ch {
                s1_flags[i] = true;
                s2_flags[j] = true;
                common_chars += 1;
                break;
            }
        }
    }

    if common_chars == 0 {
        return None;
    }

    // Walk the matched positions on both sides in order to count
    // transpositions. `j` is intentionally read after the inner loop, same
    // as the source's Python loop-variable-leak — every matched `i` is
    // guaranteed to find a corresponding flagged `j` before running off
    // the end, since both sides have exactly `common_chars` flags set.
    let mut k: usize = 0;
    let mut trans_count: usize = 0;
    for (i, &matched) in s1_flags.iter().enumerate() {
        if !matched {
            continue;
        }
        let mut j = k;
        while j < s2_len {
            if s2_flags[j] {
                k = j + 1;
                break;
            }
            j += 1;
        }
        if s1[i] != s2[j] {
            trans_count += 1;
        }
    }
    let trans_count = trans_count / 2;

    let common = common_chars as f64;
    let mut weight = common / s1_len as f64 + common / s2_len as f64;
    weight += (common - trans_count as f64) / common;
    weight /= 3.0;

    Some(JaroCore {
        weight,
        common_chars,
    })
}

/// Jaro similarity. See `docs/behavior-cards/manasa/jaro.md`.
pub struct Jaro {
    /// Accepted for API-shape parity with the Python constructor. Has no
    /// effect here: the long-tolerance adjustment only runs on the Winkler
    /// branch (see `jaro_winkler`), which `Jaro` never takes.
    pub long_tolerance: bool,
    /// Accepted for API-shape parity. The Rust core has no external-library
    /// path, so this is a no-op.
    pub external: bool,
}

impl Default for Jaro {
    fn default() -> Self {
        Self {
            long_tolerance: false,
            external: true,
        }
    }
}

impl Algorithm for Jaro {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        debug_assert_eq!(sequences.len(), 2, "Jaro compares exactly two sequences");
        core_similarity(&sequences[0], &sequences[1])
            .map(|core| core.weight)
            .unwrap_or(0.0)
    }

    // Source: `JaroWinkler.maximum` returns the constant `1`, not the
    // longest-sequence length the trait default would compute.
    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
