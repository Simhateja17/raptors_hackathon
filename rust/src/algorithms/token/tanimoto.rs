//! Tanimoto distance.

use std::collections::BTreeMap;

use crate::core::{all_identical, Algorithm, Element, PreparedSequence, QValue, ScoreMode};

type Counts = BTreeMap<Element, usize>;

/// Tanimoto distance configuration.
///
/// Tanimoto is the log2 of the Jaccard similarity between two sequences: it
/// shares Jaccard's `qval`/`as_set`/`external` configuration and multiset
/// counting rules, but reports a value between `-inf` (totally different)
/// and `0` (equal) instead of Jaccard's `0..1` similarity.
pub struct Tanimoto {
    pub qvalue: QValue,
    pub as_set: bool,
    pub external: bool,
}

impl Default for Tanimoto {
    fn default() -> Self {
        Self::new(QValue::Elements, false, true)
    }
}

impl Tanimoto {
    pub fn new(qvalue: QValue, as_set: bool, external: bool) -> Self {
        Self {
            qvalue,
            as_set,
            external,
        }
    }

    pub fn from_python(qval: Option<usize>, as_set: bool, external: bool) -> Self {
        Self::new(QValue::from_python(qval), as_set, external)
    }

    /// The underlying Jaccard similarity, including Jaccard's own quick
    /// answers (equal/short-circuit sequences score `1`, any empty sequence
    /// scores `0`) before the Tanimoto log2 transform is applied.
    fn jaccard_similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        if all_identical(sequences) {
            return 1.0;
        }
        if sequences.iter().any(Vec::is_empty) {
            return 0.0;
        }

        let counts: Vec<Counts> = sequences.iter().map(count).collect();
        let intersection = counted(&intersection(&counts), self.as_set) as f64;
        let union = counted(&union(&counts), self.as_set) as f64;
        intersection / union
    }
}

/// Return a fresh value corresponding to Python's `tanimoto` singleton.
pub fn tanimoto() -> Tanimoto {
    Tanimoto::default()
}

fn count(sequence: &PreparedSequence) -> Counts {
    let mut counts = Counts::new();
    for element in sequence {
        *counts.entry(element.clone()).or_insert(0) += 1;
    }
    counts
}

fn intersection(counts: &[Counts]) -> Counts {
    let Some(first) = counts.first() else {
        return Counts::new();
    };
    let mut result = first.clone();
    for other in &counts[1..] {
        for key in result.keys().cloned().collect::<Vec<_>>() {
            let current = result[&key];
            match other.get(&key).copied() {
                Some(value) if value < current => {
                    result.insert(key, value);
                }
                None => {
                    result.remove(&key);
                }
                _ => {}
            }
        }
    }
    result
}

fn union(counts: &[Counts]) -> Counts {
    let Some(first) = counts.first() else {
        return Counts::new();
    };
    let mut result = first.clone();
    for other in &counts[1..] {
        for (key, value) in other {
            let entry = result.entry(key.clone()).or_insert(0);
            if *value > *entry {
                *entry = *value;
            }
        }
    }
    result
}

fn counted(counts: &Counts, as_set: bool) -> usize {
    if as_set {
        counts.len()
    } else {
        counts.values().sum()
    }
}

impl Algorithm for Tanimoto {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let similarity = self.jaccard_similarity(sequences);
        if similarity == 0.0 {
            f64::NEG_INFINITY
        } else {
            similarity.log2()
        }
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
