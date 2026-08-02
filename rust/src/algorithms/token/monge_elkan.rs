//! Monge-Elkan similarity.
//!
//! This is the Rust equivalent of
//! `textdistance.algorithms.token_based.MongeElkan`.  The configured edit
//! algorithm is held behind the shared similarity-comparator seam, while the
//! maximum callback is retained so the source algorithm's unusual maximum
//! calculation remains exact.

use std::sync::Arc;

use crate::algorithms::damerau_levenshtein::DamerauLevenshtein;
use crate::core::{
    prepare_sequences, Algorithm, AlgorithmError, Element, InputSequence, PreparedSequence, QValue,
    ScoreMode, SimilarityComparator,
};

type MaximumFn = dyn Fn(&[PreparedSequence]) -> f64;

/// Monge-Elkan configuration.
pub struct MongeElkan {
    pub qvalue: QValue,
    pub symmetric: bool,
    pub external: bool,
    comparator: Box<dyn SimilarityComparator>,
    maximum: Box<MaximumFn>,
}

impl Default for MongeElkan {
    fn default() -> Self {
        Self::new(DamerauLevenshtein::default(), false, QValue::Elements, true)
    }
}

impl MongeElkan {
    /// Construct Monge-Elkan with a built-in Rust algorithm that implements
    /// the shared `Algorithm` trait.
    pub fn new<C>(algorithm: C, symmetric: bool, qvalue: QValue, external: bool) -> Self
    where
        C: Algorithm + 'static,
    {
        let algorithm = Arc::new(algorithm);
        let comparator = Box::new(AlgorithmComparator {
            algorithm: algorithm.clone(),
        });
        let maximum = Box::new(move |sequences: &[PreparedSequence]| {
            maximum_for_algorithm(algorithm.as_ref(), sequences)
        });

        Self {
            qvalue,
            symmetric,
            external,
            comparator,
            maximum,
        }
    }

    /// Construct from Python-style options while selecting a named built-in
    /// Rust comparison algorithm.
    pub fn from_python<C>(
        algorithm: C,
        symmetric: bool,
        qval: Option<usize>,
        external: bool,
    ) -> Self
    where
        C: Algorithm + 'static,
    {
        Self::new(algorithm, symmetric, QValue::from_python(qval), external)
    }

    /// Run the Python-facing preparation path before calculating similarity.
    pub fn try_similarity_inputs(&self, inputs: &[InputSequence]) -> Result<f64, AlgorithmError> {
        let prepared = prepare_sequences(inputs, self.qvalue)?;
        Ok(Algorithm::similarity(self, &prepared))
    }

    fn calculate(&self, sequences: &[PreparedSequence]) -> f64 {
        let Some(first) = sequences.first() else {
            return 0.0;
        };

        if first.is_empty() {
            return 0.0;
        }

        let mut best_values = Vec::new();
        for first_element in first {
            let left = element_sequence(first_element);
            for sequence in &sequences[1..] {
                let mut best = f64::NEG_INFINITY;
                for candidate in sequence {
                    let right = element_sequence(candidate);
                    best = best.max(self.comparator.compare(&left, &right));
                }
                best_values.push(best);
            }
        }

        let sum: f64 = best_values.iter().sum();
        sum / first.len() as f64 / best_values.len() as f64
    }

    fn symmetric_score(&self, sequences: &[PreparedSequence]) -> f64 {
        let mut permutations = sequences.to_vec();
        let mut scores = Vec::new();
        collect_permutation_scores(self, &mut permutations, 0, &mut scores);
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

/// Return a fresh value corresponding to Python's `monge_elkan` singleton.
pub fn monge_elkan() -> MongeElkan {
    MongeElkan::default()
}

impl Algorithm for MongeElkan {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if let Some(answer) = <Self as Algorithm>::quick_answer(self, sequences) {
            return answer;
        }

        if self.symmetric {
            self.symmetric_score(sequences)
        } else {
            self.calculate(sequences)
        }
    }

    fn maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        (self.maximum)(sequences)
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}

/// Adapt a built-in `Algorithm` to the pairwise comparator used by
/// Monge-Elkan.  The algorithm is shared with the maximum callback so both
/// paths use the same configuration.
struct AlgorithmComparator<C> {
    algorithm: Arc<C>,
}

impl<C: Algorithm> SimilarityComparator for AlgorithmComparator<C> {
    fn compare(&self, left: &PreparedSequence, right: &PreparedSequence) -> f64 {
        let pair = [left.clone(), right.clone()];
        Algorithm::similarity(self.algorithm.as_ref(), &pair)
    }
}

/// Match Python's `self.algorithm.maximum(sequences)` call.  Python passes
/// the tuple of outer sequences as one argument, so its length is the number
/// of outer sequences.  The value of the placeholder elements is irrelevant
/// to the built-in maximum implementations; only their length is observed.
fn maximum_for_algorithm<C: Algorithm>(algorithm: &C, sequences: &[PreparedSequence]) -> f64 {
    let outer_bundle = vec![Element::Boolean(false); sequences.len()];
    let mut result = algorithm.maximum(&[outer_bundle]);

    for sequence in sequences {
        if sequence.is_empty() {
            continue;
        }
        let inner_sequences: Vec<PreparedSequence> =
            sequence.iter().map(element_sequence).collect();
        result = result.max(algorithm.maximum(&inner_sequences));
    }

    result
}

/// Convert one outer token into the sequence seen by the underlying Python
/// algorithm.  Text tokens become Unicode scalar values; q-gram tokens expose
/// their contained elements; scalar tokens remain one-element sequences.
fn element_sequence(element: &Element) -> PreparedSequence {
    match element {
        Element::Text(value) => value.chars().map(Element::Char).collect(),
        Element::Gram(elements) => elements.clone(),
        Element::Char(_) | Element::Byte(_) | Element::Integer(_) | Element::Boolean(_) => {
            vec![element.clone()]
        }
    }
}

fn collect_permutation_scores(
    algorithm: &MongeElkan,
    sequences: &mut [PreparedSequence],
    index: usize,
    scores: &mut Vec<f64>,
) {
    if index == sequences.len() {
        scores.push(algorithm.calculate(sequences));
        return;
    }

    for candidate_index in index..sequences.len() {
        sequences.swap(index, candidate_index);
        collect_permutation_scores(algorithm, sequences, index + 1, scores);
        sequences.swap(index, candidate_index);
    }
}
