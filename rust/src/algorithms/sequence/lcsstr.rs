//! Longest common substring.
//!
//! This is the Rust equivalent of `textdistance.algorithms.sequence_based.LCSStr`.
//! Unlike numeric algorithms, the Python call returns the matching substring
//! itself.  The shared output contract therefore carries the result as a
//! `Sequence`; the adapter can reconstruct the source-language value from it.

use crate::core::{
    maximum_length, prepare_sequences, AlgorithmError, AlgorithmOutput, Element, InputSequence,
    OutputAlgorithm, PreparedSequence, QValue, ScoreMode,
};

/// Longest-common-substring configuration.
pub struct LCSStr {
    pub qvalue: QValue,
    pub external: bool,
}

impl Default for LCSStr {
    fn default() -> Self {
        Self::new(QValue::Elements, true)
    }
}

impl LCSStr {
    pub fn new(qvalue: QValue, external: bool) -> Self {
        Self { qvalue, external }
    }

    pub fn from_python(qval: Option<usize>, external: bool) -> Self {
        Self::new(QValue::from_python(qval), external)
    }

    /// Apply the Python call-order rules to unprepared input.
    ///
    /// In particular, the source returns an empty value before q-value
    /// preparation and returns a single non-empty input unchanged, even when
    /// `qval` requests words or n-grams.
    pub fn output_inputs(
        &self,
        inputs: &[InputSequence],
    ) -> Result<AlgorithmOutput, AlgorithmError> {
        let raw = prepare_sequences(inputs, QValue::Elements)?;

        if inputs.is_empty() || raw.iter().any(Vec::is_empty) {
            return Ok(AlgorithmOutput::Sequence(Vec::new()));
        }

        if inputs.len() == 1 {
            return Ok(AlgorithmOutput::Sequence(raw[0].clone()));
        }

        let prepared = prepare_sequences(inputs, self.qvalue)?;
        self.output(&prepared)
    }

    /// Return the matching contiguous sequence from already-prepared inputs.
    pub fn substring(
        &self,
        sequences: &[PreparedSequence],
    ) -> Result<PreparedSequence, AlgorithmError> {
        if sequences.is_empty() || sequences.iter().any(Vec::is_empty) {
            return Ok(Vec::new());
        }

        if sequences.len() == 1 {
            return Ok(sequences[0].clone());
        }

        if sequences.len() == 2 && maximum_length(sequences) < 200 {
            Ok(standard_substring(&sequences[0], &sequences[1]))
        } else {
            custom_substring(sequences)
        }
    }

    pub fn similarity(&self, sequences: &[PreparedSequence]) -> Result<f64, AlgorithmError> {
        Ok(self.substring(sequences)?.len() as f64)
    }
}

/// Return a fresh value corresponding to Python's `lcsstr` singleton.
pub fn lcsstr() -> LCSStr {
    LCSStr::default()
}

impl OutputAlgorithm for LCSStr {
    fn output(&self, sequences: &[PreparedSequence]) -> Result<AlgorithmOutput, AlgorithmError> {
        Ok(AlgorithmOutput::Sequence(self.substring(sequences)?))
    }

    fn output_maximum(&self, sequences: &[PreparedSequence]) -> f64 {
        maximum_length(sequences) as f64
    }

    fn output_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}

/// Dynamic-programming implementation of `SequenceMatcher.find_longest_match`.
///
/// For equal-length matches, the strict comparison preserves the first match
/// encountered while scanning the first sequence and then the second sequence,
/// matching `SequenceMatcher`'s deterministic tie-breaking.
fn standard_substring(left: &PreparedSequence, right: &PreparedSequence) -> PreparedSequence {
    let mut previous = vec![0usize; right.len() + 1];
    let mut best_length = 0usize;
    let mut best_end = 0usize;

    for (left_index, left_element) in left.iter().enumerate() {
        let mut current = vec![0usize; right.len() + 1];
        for (right_index, right_element) in right.iter().enumerate() {
            if left_element == right_element {
                let length = previous[right_index] + 1;
                current[right_index + 1] = length;
                if length > best_length {
                    best_length = length;
                    best_end = left_index + 1;
                }
            }
        }
        previous = current;
    }

    left[best_end - best_length..best_end].to_vec()
}

/// The source's fallback searches windows in the first shortest sequence from
/// longest to shortest, then returns the first candidate present in every
/// sequence.  It joins string-like elements before membership checks, just as
/// Python's `''.join(subseq)` does.  Non-string-like elements therefore report
/// a clear boundary error instead of being silently stringified.
fn custom_substring(sequences: &[PreparedSequence]) -> Result<PreparedSequence, AlgorithmError> {
    let shortest_index = sequences
        .iter()
        .enumerate()
        .min_by_key(|(_, sequence)| sequence.len())
        .map(|(index, _)| index)
        .expect("custom_substring requires at least one sequence");
    let shortest = &sequences[shortest_index];

    for length in (1..=shortest.len()).rev() {
        for start in 0..=shortest.len() - length {
            let window = &shortest[start..start + length];
            let joined = join_string_like(window)?;

            if sequences
                .iter()
                .all(|sequence| contains_custom(sequence, &joined))
            {
                return Ok(joined.chars().map(Element::Char).collect());
            }
        }
    }

    Ok(Vec::new())
}

fn join_string_like(sequence: &[Element]) -> Result<String, AlgorithmError> {
    let mut joined = String::new();
    for element in sequence {
        match element {
            Element::Char(value) => joined.push(*value),
            Element::Text(value) => joined.push_str(value),
            Element::Byte(_) | Element::Integer(_) | Element::Boolean(_) | Element::Gram(_) => {
                return Err(AlgorithmError::InvalidInput(
                    "LCSStr fallback requires string-like elements".to_owned(),
                ));
            }
        }
    }
    Ok(joined)
}

fn contains_custom(sequence: &PreparedSequence, joined: &str) -> bool {
    match sequence.first() {
        Some(Element::Char(_)) => {
            let candidate: PreparedSequence = joined.chars().map(Element::Char).collect();
            sequence
                .windows(candidate.len())
                .any(|window| window == candidate.as_slice())
        }
        Some(Element::Text(_)) => sequence
            .iter()
            .any(|element| matches!(element, Element::Text(value) if value == joined)),
        Some(Element::Byte(_))
        | Some(Element::Integer(_))
        | Some(Element::Boolean(_))
        | Some(Element::Gram(_))
        | None => false,
    }
}
