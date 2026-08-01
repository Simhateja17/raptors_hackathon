//! Bag distance.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

type Counts = BTreeMap<Element, usize>;

/// Error raised by the Python implementation when Bag receives no sequences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BagError {
    EmptyInputList,
}

impl Display for BagError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInputList => formatter.write_str("Bag requires at least one sequence"),
        }
    }
}

impl Error for BagError {}

/// Bag distance configuration.
pub struct Bag {
    pub qvalue: QValue,
    pub as_set: bool,
    pub external: bool,
}

impl Default for Bag {
    fn default() -> Self {
        Self::new(QValue::Elements, false, true)
    }
}

impl Bag {
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

    /// Calculate Bag while preserving the no-sequence error as a `Result`.
    pub fn try_raw_score(&self, sequences: &[PreparedSequence]) -> Result<f64, BagError> {
        if sequences.is_empty() {
            return Err(BagError::EmptyInputList);
        }

        let counts: Vec<Counts> = sequences.iter().map(count).collect();
        let shared = intersection(&counts);
        let largest_remainder = counts
            .iter()
            .map(|sequence| counted(&subtract(sequence, &shared), self.as_set))
            .max()
            .unwrap_or(0);
        Ok(largest_remainder as f64)
    }
}

pub fn bag() -> Bag {
    Bag::default()
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

fn subtract(left: &Counts, right: &Counts) -> Counts {
    let mut result = Counts::new();
    for (key, value) in left {
        let remainder = value.saturating_sub(right.get(key).copied().unwrap_or(0));
        if remainder > 0 {
            result.insert(key.clone(), remainder);
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

impl Algorithm for Bag {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        self.try_raw_score(sequences)
            .unwrap_or_else(|error| panic!("Bag score failed: {error}"))
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
