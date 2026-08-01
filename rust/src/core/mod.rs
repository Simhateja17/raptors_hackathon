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

/// Values returned by the public algorithm interface.
///
/// Numeric algorithms return `Score`; sequence-producing algorithms such as
/// LCSSeq and LCSStr return `Sequence` so the adapter can preserve the source
/// library's observable return value instead of reducing it to a length.
#[derive(Clone, Debug, PartialEq)]
pub enum AlgorithmOutput {
    Score(f64),
    Sequence(Sequence),
}

impl AlgorithmOutput {
    /// Convert an output to the scalar used by similarity/distance helpers.
    /// A returned sequence contributes its element count.
    pub fn scalar_value(&self) -> f64 {
        match self {
            Self::Score(value) => *value,
            Self::Sequence(sequence) => sequence.len() as f64,
        }
    }

    pub fn sequence(&self) -> Option<&Sequence> {
        match self {
            Self::Score(_) => None,
            Self::Sequence(sequence) => Some(sequence),
        }
    }
}

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

/// Errors that may be reported by an algorithm or the adapter seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlgorithmError {
    Input(InputError),
    InvalidConfiguration(String),
    InvalidInput(String),
    UnsupportedCustomComparator,
}

impl Display for AlgorithmError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => write!(formatter, "input error: {error}"),
            Self::InvalidConfiguration(message) => {
                write!(formatter, "invalid algorithm configuration: {message}")
            }
            Self::InvalidInput(message) => write!(formatter, "invalid algorithm input: {message}"),
            Self::UnsupportedCustomComparator => formatter
                .write_str("custom comparison functions are not supported by the Rust port"),
        }
    }
}

impl Error for AlgorithmError {}

impl From<InputError> for AlgorithmError {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

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

pub fn output_distance(output: &AlgorithmOutput, mode: ScoreMode, maximum: f64) -> f64 {
    match mode {
        ScoreMode::Distance => output.scalar_value(),
        ScoreMode::Similarity => maximum - output.scalar_value(),
    }
}

pub fn output_similarity(output: &AlgorithmOutput, mode: ScoreMode, maximum: f64) -> f64 {
    match mode {
        ScoreMode::Distance => maximum - output.scalar_value(),
        ScoreMode::Similarity => output.scalar_value(),
    }
}

/// A pairwise similarity seam for algorithms that delegate comparisons.
///
/// The adapter selects a built-in Rust implementation such as Jaro-Winkler
/// or Damerau-Levenshtein. Arbitrary source-language callbacks do not cross
/// this seam.
pub trait SimilarityComparator {
    fn compare(&self, left: &PreparedSequence, right: &PreparedSequence) -> f64;
}

impl<T: Algorithm> SimilarityComparator for T {
    fn compare(&self, left: &PreparedSequence, right: &PreparedSequence) -> f64 {
        let pair = [left.clone(), right.clone()];
        Algorithm::similarity(self, &pair)
    }
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

/// Output-capable interface used by numeric and sequence-producing modules.
/// Numeric `Algorithm` implementations receive this interface automatically;
/// LCSSeq/LCSStr implement it directly and return `AlgorithmOutput::Sequence`.
pub trait OutputAlgorithm {
    fn output(&self, sequences: &[PreparedSequence]) -> Result<AlgorithmOutput, AlgorithmError>;

    fn output_maximum(&self, sequences: &[PreparedSequence]) -> f64;

    fn output_mode(&self) -> ScoreMode;
}

impl<T: Algorithm> OutputAlgorithm for T {
    fn output(&self, sequences: &[PreparedSequence]) -> Result<AlgorithmOutput, AlgorithmError> {
        Ok(AlgorithmOutput::Score(Algorithm::call(self, sequences)))
    }

    fn output_maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        Algorithm::maximum(self, sequences)
    }

    fn output_mode(&self) -> ScoreMode {
        Algorithm::score_mode(self)
    }
}
