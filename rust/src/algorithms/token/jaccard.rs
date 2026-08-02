//! Jaccard similarity.

use std::collections::BTreeMap;

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

type Counts = BTreeMap<Element, usize>;

/// Jaccard similarity configuration.
pub struct Jaccard {
    pub qvalue: QValue,
    pub as_set: bool,
    pub external: bool,
}

impl Default for Jaccard {
    fn default() -> Self {
        Self::new(QValue::Elements, false, true)
    }
}

impl Jaccard {
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
}

/// Return a fresh value corresponding to Python's `jaccard` singleton.
pub fn jaccard() -> Jaccard {
    Jaccard::default()
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
    let mut result = Counts::new();
    for current in counts {
        for (key, value) in current {
            let entry = result.entry(key.clone()).or_insert(0);
            *entry = (*entry).max(*value);
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

impl Algorithm for Jaccard {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return answer;
        }

        let counts: Vec<Counts> = sequences.iter().map(count).collect();
        let intersection = counted(&intersection(&counts), self.as_set) as f64;
        let union = counted(&union(&counts), self.as_set) as f64;
        intersection / union
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
