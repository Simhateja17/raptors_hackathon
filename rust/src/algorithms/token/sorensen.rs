//! Sørensen-Dice similarity.

use std::collections::BTreeMap;

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

type Counts = BTreeMap<Element, usize>;

/// Sørensen-Dice similarity configuration.
pub struct Sorensen {
    pub qvalue: QValue,
    pub as_set: bool,
    pub external: bool,
}

impl Default for Sorensen {
    fn default() -> Self {
        Self::new(QValue::Elements, false, true)
    }
}

impl Sorensen {
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

/// Return a fresh value corresponding to Python's `sorensen` singleton.
pub fn sorensen() -> Sorensen {
    Sorensen::default()
}

/// Python's `dice` and `sorensen_dice` aliases have the same behavior.
pub fn dice() -> Sorensen {
    Sorensen::default()
}

/// Python's `sorensen_dice` alias has the same behavior.
pub fn sorensen_dice() -> Sorensen {
    Sorensen::default()
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

fn counted(counts: &Counts, as_set: bool) -> usize {
    if as_set {
        counts.len()
    } else {
        counts.values().sum()
    }
}

impl Algorithm for Sorensen {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return answer;
        }

        let counts: Vec<Counts> = sequences.iter().map(count).collect();
        let total: usize = counts.iter().map(|value| counted(value, self.as_set)).sum();
        let shared = counted(&intersection(&counts), self.as_set);
        2.0 * shared as f64 / total as f64
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
