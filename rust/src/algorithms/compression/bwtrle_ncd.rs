//! Burrows-Wheeler transform + run-length-encoding normalized compression distance.

use crate::core::{Algorithm, Element, PreparedSequence};

fn permutation_indices(length: usize) -> Vec<Vec<usize>> {
    fn visit(
        length: usize,
        used: &mut [bool],
        current: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if current.len() == length {
            result.push(current.clone());
            return;
        }
        for index in 0..length {
            if used[index] {
                continue;
            }
            used[index] = true;
            current.push(index);
            visit(length, used, current, result);
            current.pop();
            used[index] = false;
        }
    }

    let mut result = Vec::new();
    visit(
        length,
        &mut vec![false; length],
        &mut Vec::new(),
        &mut result,
    );
    result
}

/// Burrows-Wheeler transform + run-length-encoding normalized compression distance.
#[derive(Clone, Debug)]
pub struct BWTRLENCD {
    terminator: Element,
}

impl BWTRLENCD {
    pub fn new() -> Self {
        Self {
            terminator: Element::Char('\0'),
        }
    }

    pub fn with_terminator(terminator: Element) -> Self {
        Self { terminator }
    }

    pub fn terminator(&self) -> &Element {
        &self.terminator
    }

    /// Burrows-Wheeler transform of `data`, or `data` unchanged when the
    /// terminator is already present. Mirrors `BWTRLENCD._compress`'s guard.
    fn transform(&self, data: &PreparedSequence) -> PreparedSequence {
        if data.is_empty() {
            return vec![self.terminator.clone()];
        }
        if data.contains(&self.terminator) {
            return data.clone();
        }

        let mut extended = data.clone();
        extended.push(self.terminator.clone());

        let mut rotations: Vec<PreparedSequence> = (0..extended.len())
            .map(|start| {
                let mut rotation = extended[start..].to_vec();
                rotation.extend_from_slice(&extended[..start]);
                rotation
            })
            .collect();
        rotations.sort();

        rotations
            .into_iter()
            .map(|rotation| rotation.last().expect("rotation is never empty").clone())
            .collect()
    }

    /// Length of the run-length encoding of `data`, matching
    /// `RLENCD._compress`: a run of one element keeps its element, a run of
    /// two stays unchanged, and a longer run becomes `"{n}"` plus the
    /// element.
    fn compressed_len(data: &PreparedSequence) -> usize {
        let mut total = 0usize;
        let mut index = 0;
        while index < data.len() {
            let mut end = index + 1;
            while end < data.len() && data[end] == data[index] {
                end += 1;
            }
            let run_length = end - index;
            total += match run_length {
                1 => 1,
                2 => 2,
                _ => run_length.to_string().len() + 1,
            };
            index = end;
        }
        total
    }

    /// Length of the compressed representation of `data`, i.e. Python's
    /// `len(self._compress(data))`.
    pub fn size(&self, data: &PreparedSequence) -> usize {
        Self::compressed_len(&self.transform(data))
    }

    fn concatenated(sequences: &[PreparedSequence], order: &[usize]) -> PreparedSequence {
        order
            .iter()
            .flat_map(|&index| sequences[index].iter().cloned())
            .collect()
    }
}

impl Default for BWTRLENCD {
    fn default() -> Self {
        Self::new()
    }
}

impl Algorithm for BWTRLENCD {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }

        let mut concatenated_size = usize::MAX;
        for order in permutation_indices(sequences.len()) {
            let data = Self::concatenated(sequences, &order);
            concatenated_size = concatenated_size.min(self.size(&data));
        }

        let compressed_sizes: Vec<usize> = sequences.iter().map(|data| self.size(data)).collect();
        let maximum = compressed_sizes.iter().copied().max().unwrap_or(0);
        if maximum == 0 {
            return 0.0;
        }
        let minimum = compressed_sizes.iter().copied().min().unwrap_or(0);
        (concatenated_size as f64 - minimum as f64 * (sequences.len() - 1) as f64) / maximum as f64
    }

    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }
}
