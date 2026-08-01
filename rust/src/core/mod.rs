//! Shared data model and behavioral contract for every algorithm.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// A normalized input element supported by the initial adapter contract.
///
/// Python strings are represented as Unicode scalar values (`char`), not UTF-8
/// bytes.  The remaining variants cover the sequence forms exercised by the
/// original suite and keep the Rust core independent of Python objects.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Element {
    Char(char),
    Byte(u8),
    Integer(i64),
    Boolean(bool),
    Text(String),
    Gram(Vec<Element>),
}

/// A prepared sequence consumed by algorithms.
pub type Sequence = Vec<Element>;

/// Input forms accepted by the shared Rust contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputSequence {
    Text(String),
    Bytes(Vec<u8>),
    Integers(Vec<i64>),
    Booleans(Vec<bool>),
    Elements(Sequence),
}

/// A sequence after q-value/word preparation.
pub type PreparedSequence = Sequence;

/// Python-compatible q-value meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QValue {
    /// `qval=None` or `qval=0`: split text into words.
    Words,
    /// `qval=1`: compare individual sequence elements.
    Elements,
    /// `qval>1`: compare q-grams.
    NGrams(usize),
}

impl QValue {
    pub fn from_python(qval: Option<usize>) -> Self {
        match qval {
            None | Some(0) => Self::Words,
            Some(1) => Self::Elements,
            Some(value) => Self::NGrams(value),
        }
    }
}

/// Errors raised at the boundary before an algorithm runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputError {
    WordsRequireText,
    InvalidNGramSize,
}

impl Display for InputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WordsRequireText => formatter.write_str("qval=None/0 requires text input"),
            Self::InvalidNGramSize => formatter.write_str("q-gram size must be greater than one"),
        }
    }
}

impl Error for InputError {}

impl InputSequence {
    fn as_elements(&self) -> Sequence {
        match self {
            Self::Text(value) => value.chars().map(Element::Char).collect(),
            Self::Bytes(value) => value.iter().copied().map(Element::Byte).collect(),
            Self::Integers(value) => value.iter().copied().map(Element::Integer).collect(),
            Self::Booleans(value) => value.iter().copied().map(Element::Boolean).collect(),
            Self::Elements(value) => value.clone(),
        }
    }

    fn words(&self) -> Result<Sequence, InputError> {
        match self {
            Self::Text(value) => Ok(value
                .split_whitespace()
                .map(|word| Element::Text(word.to_owned()))
                .collect()),
            _ => Err(InputError::WordsRequireText),
        }
    }

    pub fn prepare(&self, qvalue: QValue) -> Result<PreparedSequence, InputError> {
        match qvalue {
            QValue::Words => self.words(),
            QValue::Elements => Ok(self.as_elements()),
            QValue::NGrams(size) => {
                if size <= 1 {
                    return Err(InputError::InvalidNGramSize);
                }
                let elements = self.as_elements();
                Ok(elements
                    .windows(size)
                    .map(|window| Element::Gram(window.to_vec()))
                    .collect())
            }
        }
    }
}

/// Prepare all input sequences using one q-value.
pub fn prepare_sequences(
    sequences: &[InputSequence],
    qvalue: QValue,
) -> Result<Vec<PreparedSequence>, InputError> {
    sequences
        .iter()
        .map(|sequence| sequence.prepare(qvalue))
        .collect()
}

/// Return the largest prepared-sequence length, or zero for no sequences.
pub fn maximum_length(sequences: &[PreparedSequence]) -> usize {
    sequences.iter().map(Vec::len).max().unwrap_or(0)
}

/// Return whether all sequences are equal.  Zero and one sequence are treated
/// as identical, matching the source library's quick-answer behavior.
pub fn all_identical(sequences: &[PreparedSequence]) -> bool {
    sequences
        .first()
        .map(|first| sequences.iter().all(|sequence| sequence == first))
        .unwrap_or(true)
}

pub fn normalize_distance(distance: f64, maximum: f64) -> f64 {
    if maximum == 0.0 {
        0.0
    } else {
        distance / maximum
    }
}

pub fn normalize_similarity(distance: f64, maximum: f64) -> f64 {
    1.0 - normalize_distance(distance, maximum)
}

/// Identifies whether an algorithm's raw score is a distance or similarity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScoreMode {
    Distance,
    Similarity,
}

/// Common behavior every algorithm implementation must expose.
pub trait Algorithm {
    /// Return the algorithm's direct/raw score.
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64;

    /// Most algorithms use the largest sequence length as their maximum.
    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        maximum_length(sequences) as f64
    }

    /// Whether `raw_score` is naturally a distance or similarity.
    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }

    fn call(&self, sequences: &[PreparedSequence]) -> f64 {
        self.raw_score(sequences)
    }

    fn distance(&self, sequences: &[PreparedSequence]) -> f64 {
        match self.score_mode() {
            ScoreMode::Distance => self.raw_score(sequences),
            ScoreMode::Similarity => self.maximum(sequences) - self.raw_score(sequences),
        }
    }

    fn similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        match self.score_mode() {
            ScoreMode::Distance => self.maximum(sequences) - self.raw_score(sequences),
            ScoreMode::Similarity => self.raw_score(sequences),
        }
    }

    fn normalized_distance(&self, sequences: &[PreparedSequence]) -> f64 {
        normalize_distance(self.distance(sequences), self.maximum(sequences))
    }

    fn normalized_similarity(&self, sequences: &[PreparedSequence]) -> f64 {
        normalize_similarity(self.distance(sequences), self.maximum(sequences))
    }

    /// Shared fast paths.  Algorithm-specific implementations may override
    /// this when their source behavior differs.
    fn quick_answer(&self, sequences: &[PreparedSequence]) -> Option<f64> {
        let maximum = self.maximum(sequences);
        if sequences.len() <= 1 || all_identical(sequences) {
            return Some(match self.score_mode() {
                ScoreMode::Distance => 0.0,
                ScoreMode::Similarity => maximum,
            });
        }
        if sequences.iter().any(Vec::is_empty) {
            return Some(match self.score_mode() {
                ScoreMode::Distance => maximum,
                ScoreMode::Similarity => 0.0,
            });
        }
        None
    }
}
