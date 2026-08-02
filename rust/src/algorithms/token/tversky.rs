//! Tversky similarity.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

type Counts = BTreeMap<Element, usize>;

/// Errors which the Python implementation raises for invalid Tversky options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TverskyError {
    MissingBiasedCoefficient,
    ZeroDenominator,
}

impl Display for TverskyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBiasedCoefficient => {
                formatter.write_str("biased Tversky requires two coefficients")
            }
            Self::ZeroDenominator => formatter.write_str("Tversky denominator is zero"),
        }
    }
}

impl Error for TverskyError {}

/// Tversky similarity configuration.
pub struct Tversky {
    pub qvalue: QValue,
    /// `None` means the Python default infinite stream of `1.0` values.
    pub ks: Option<Vec<f64>>,
    pub bias: Option<f64>,
    pub as_set: bool,
    pub external: bool,
}

impl Default for Tversky {
    fn default() -> Self {
        Self::new(QValue::Elements, None, None, false, true)
    }
}

impl Tversky {
    pub fn new(
        qvalue: QValue,
        ks: Option<Vec<f64>>,
        bias: Option<f64>,
        as_set: bool,
        external: bool,
    ) -> Self {
        Self {
            qvalue,
            ks: ks.filter(|values| !values.is_empty()),
            bias,
            as_set,
            external,
        }
    }

    pub fn from_python(
        qval: Option<usize>,
        ks: Option<Vec<f64>>,
        bias: Option<f64>,
        as_set: bool,
        external: bool,
    ) -> Self {
        Self::new(QValue::from_python(qval), ks, bias, as_set, external)
    }

    fn coefficients(&self, count: usize) -> Vec<f64> {
        match &self.ks {
            Some(values) => values.iter().copied().take(count).collect(),
            None => vec![1.0; count],
        }
    }

    /// Calculate the Python score while preserving its fallible option cases.
    pub fn try_similarity(&self, sequences: &[PreparedSequence]) -> Result<f64, TverskyError> {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return Ok(answer);
        }

        let counts: Vec<Counts> = sequences.iter().map(count).collect();
        let shared = counted(&intersection(&counts), self.as_set) as f64;
        let sizes: Vec<f64> = counts
            .iter()
            .map(|value| counted(value, self.as_set) as f64)
            .collect();
        let coefficients = self.coefficients(sequences.len());

        if sequences.len() == 2 {
            if let Some(bias) = self.bias {
                if coefficients.len() < 2 {
                    return Err(TverskyError::MissingBiasedCoefficient);
                }
                let alpha = coefficients[0];
                let beta = coefficients[1];
                let a = sizes[0].min(sizes[1]);
                let b = sizes[0].max(sizes[1]);
                let c = shared + bias;
                let result = alpha * beta * (a - b) + b * beta;
                let denominator = result + c;
                if denominator == 0.0 {
                    return Err(TverskyError::ZeroDenominator);
                }
                return Ok(c / denominator);
            }
        }

        let mut result = shared;
        for (coefficient, size) in coefficients.iter().zip(sizes.iter()) {
            result += coefficient * (size - shared);
        }
        if result == 0.0 {
            return Err(TverskyError::ZeroDenominator);
        }
        Ok(shared / result)
    }
}

pub fn tversky() -> Tversky {
    Tversky::default()
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

impl Algorithm for Tversky {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        self.try_similarity(sequences)
            .unwrap_or_else(|error| panic!("Tversky score failed: {error}"))
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}
