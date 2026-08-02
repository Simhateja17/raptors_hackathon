//! Run-length encoded normalized compression distance.

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::{Algorithm, Element, PreparedSequence, QValue, ScoreMode};

/// Errors which Python raises when RLE is given values that cannot be joined
/// into its string output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RleError {
    UnsupportedElement(&'static str),
}

impl Display for RleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedElement(kind) => {
                write!(formatter, "RLE cannot encode {kind} elements as strings")
            }
        }
    }
}

impl Error for RleError {}

/// RLE NCD configuration.
pub struct RleNcd {
    pub qvalue: QValue,
}

impl Default for RleNcd {
    fn default() -> Self {
        Self::new(QValue::Elements)
    }
}

impl RleNcd {
    pub fn new(qvalue: QValue) -> Self {
        Self { qvalue }
    }

    pub fn from_python(qval: Option<usize>) -> Self {
        Self::new(QValue::from_python(qval))
    }

    /// Return the exact encoded representation for one prepared sequence.
    pub fn compress(&self, sequence: &PreparedSequence) -> Result<String, RleError> {
        let mut output = String::new();
        let mut start = 0;

        while start < sequence.len() {
            let element = &sequence[start];
            let mut end = start + 1;
            while end < sequence.len() && sequence[end] == *element {
                end += 1;
            }

            let token = render(element)?;
            let run_length = end - start;
            match run_length {
                1 => output.push_str(&token),
                2 => {
                    output.push_str(&token);
                    output.push_str(&token);
                }
                count => {
                    output.push_str(&count.to_string());
                    output.push_str(&token);
                }
            }
            start = end;
        }

        Ok(output)
    }

    pub fn compressed_size(&self, sequence: &PreparedSequence) -> Result<usize, RleError> {
        Ok(self.compress(sequence)?.chars().count())
    }

    /// Calculate NCD and preserve unsupported-input failures as a `Result`.
    pub fn try_raw_score(&self, sequences: &[PreparedSequence]) -> Result<f64, RleError> {
        if sequences.is_empty() {
            return Ok(0.0);
        }

        let compressed_sizes: Vec<usize> = sequences
            .iter()
            .map(|sequence| self.compressed_size(sequence))
            .collect::<Result<_, _>>()?;
        let Some(&maximum) = compressed_sizes.iter().max() else {
            return Ok(0.0);
        };
        if maximum == 0 {
            return Ok(0.0);
        }
        let minimum = compressed_sizes.iter().copied().min().unwrap_or(0);

        let mut order: Vec<usize> = (0..sequences.len()).collect();
        let mut smallest_concat = usize::MAX;
        for_each_permutation(&mut order, 0, &mut |permutation| {
            let mut concatenated = PreparedSequence::new();
            for &index in permutation {
                concatenated.extend(sequences[index].iter().cloned());
            }
            // Every individual sequence was validated above, so a concatenated
            // permutation contains only renderable elements as well.
            let size = self
                .compressed_size(&concatenated)
                .expect("validated RLE elements must remain valid after concatenation");
            smallest_concat = smallest_concat.min(size);
        });

        let numerator = smallest_concat as f64 - minimum as f64 * (sequences.len() - 1) as f64;
        Ok(numerator / maximum as f64)
    }
}

pub fn rle_ncd() -> RleNcd {
    RleNcd::default()
}

fn render(element: &Element) -> Result<String, RleError> {
    match element {
        Element::Char(value) => Ok(value.to_string()),
        Element::Text(value) => Ok(value.clone()),
        Element::Byte(_) => Err(RleError::UnsupportedElement("byte")),
        Element::Integer(_) => Err(RleError::UnsupportedElement("integer")),
        Element::Boolean(_) => Err(RleError::UnsupportedElement("boolean")),
        Element::Gram(_) => Err(RleError::UnsupportedElement("q-gram")),
    }
}

fn for_each_permutation<F>(values: &mut [usize], start: usize, callback: &mut F)
where
    F: FnMut(&[usize]),
{
    if start == values.len() {
        callback(values);
        return;
    }

    for index in start..values.len() {
        values.swap(start, index);
        for_each_permutation(values, start + 1, callback);
        values.swap(start, index);
    }
}

impl Algorithm for RleNcd {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        self.try_raw_score(sequences)
            .unwrap_or_else(|error| panic!("RLE NCD failed: {error}"))
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
