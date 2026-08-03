//! Arithmetic-coding normalized compression distance.

use std::cmp::Ordering;

use crate::core::{Algorithm, Element, PreparedSequence};

/// A positive exact rational used by the arithmetic-coding interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rational {
    pub numerator: u128,
    pub denominator: u128,
}

impl Rational {
    pub const fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    pub const fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    pub fn new(numerator: u128, denominator: u128) -> Self {
        assert!(denominator != 0, "rational denominator must not be zero");
        if numerator == 0 {
            return Self::zero();
        }
        let divisor = gcd(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }

    fn try_add(self, other: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .checked_mul(other.denominator)?
            .checked_add(other.numerator.checked_mul(self.denominator)?)?;
        let denominator = self.denominator.checked_mul(other.denominator)?;
        Some(Self::new(numerator, denominator))
    }

    fn try_multiply(self, other: Self) -> Option<Self> {
        Some(Self::new(
            self.numerator.checked_mul(other.numerator)?,
            self.denominator.checked_mul(other.denominator)?,
        ))
    }

    fn try_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.numerator
                .checked_mul(other.denominator)?
                .cmp(&other.numerator.checked_mul(self.denominator)?),
        )
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        self.try_cmp(other).unwrap_or_else(|| {
            (self.numerator as f64 / self.denominator as f64)
                .partial_cmp(&(other.numerator as f64 / other.denominator as f64))
                .unwrap_or(Ordering::Equal)
        })
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

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

/// Arithmetic-coding normalized compression distance.
#[derive(Clone, Copy, Debug)]
pub struct ArithNCD {
    base: u32,
    terminator: Option<char>,
}

impl ArithNCD {
    pub const fn new() -> Self {
        Self {
            base: 2,
            terminator: None,
        }
    }

    pub const fn with_config(base: u32, terminator: Option<char>) -> Self {
        Self { base, terminator }
    }

    pub const fn base(self) -> u32 {
        self.base
    }

    pub const fn terminator(self) -> Option<char> {
        self.terminator
    }

    fn counts(&self, sequences: &[PreparedSequence]) -> Vec<(Element, u128)> {
        let mut counts: Vec<(Element, u128)> = Vec::new();
        for sequence in sequences {
            for element in sequence {
                if let Some((_, count)) = counts.iter_mut().find(|(key, _)| key == element) {
                    *count += 1;
                } else {
                    counts.push((element.clone(), 1));
                }
            }
        }

        if let Some(terminator) = self.terminator {
            let element = Element::Char(terminator);
            if let Some((_, count)) = counts.iter_mut().find(|(key, _)| *key == element) {
                *count = 1;
            } else {
                counts.push((element, 1));
            }
        }

        // Python Counter.most_common() sorts by descending count while
        // retaining first-seen order for ties. Vec::sort_by is stable.
        counts.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        counts
    }

    /// Return `(symbol, cumulative_probability, symbol_probability)` entries.
    pub fn make_probs(&self, sequences: &[PreparedSequence]) -> Vec<(Element, Rational, Rational)> {
        let counts = self.counts(sequences);
        let total: u128 = counts.iter().map(|(_, count)| *count).sum();
        if total == 0 {
            return Vec::new();
        }

        let mut cumulative = 0u128;
        counts
            .into_iter()
            .map(|(element, count)| {
                let start = Rational::new(cumulative, total);
                let width = Rational::new(count, total);
                cumulative += count;
                (element, start, width)
            })
            .collect()
    }

    fn probability_for(
        probabilities: &[(Element, Rational, Rational)],
        element: &Element,
    ) -> (Rational, Rational) {
        probabilities
            .iter()
            .find(|(key, _, _)| key == element)
            .map(|(_, start, width)| (*start, *width))
            .expect("arithmetic-coding input symbol missing from probability table")
    }

    fn try_interval(&self, data: &PreparedSequence) -> Option<(Rational, Rational)> {
        let probabilities = self.make_probs(std::slice::from_ref(data));
        let mut symbols = data.clone();
        if let Some(terminator) = self.terminator {
            symbols.retain(|element| *element != Element::Char(terminator));
            symbols.push(Element::Char(terminator));
        }

        let mut start = Rational::zero();
        let mut width = Rational::one();
        for element in symbols {
            let (probability_start, probability_width) =
                Self::probability_for(&probabilities, &element);
            start = start.try_add(probability_start.try_multiply(width)?)?;
            width = width.try_multiply(probability_width)?;
        }
        Some((start, start.try_add(width)?))
    }

    fn try_compress(&self, data: &PreparedSequence) -> Option<Rational> {
        let (start, end) = self.try_interval(data)?;
        let mut output = Rational::zero();
        let mut output_denominator = 1u128;

        loop {
            let in_range = start.try_cmp(&output)? != Ordering::Greater
                && output.try_cmp(&end)? == Ordering::Less;
            if in_range {
                return Some(output);
            }

            let numerator =
                start.numerator.checked_mul(output_denominator)? / start.denominator + 1;
            output = Rational::new(numerator, output_denominator);
            output_denominator = output_denominator.checked_mul(2)?;
        }
    }

    fn approximate_size(&self, data: &PreparedSequence) -> u128 {
        let counts = self.counts(std::slice::from_ref(data));
        let total: f64 = counts.iter().map(|(_, count)| *count as f64).sum();
        if total == 0.0 {
            return 0;
        }
        let base = self.base.max(2) as f64;
        let code_length: f64 = counts
            .iter()
            .map(|(_, count)| {
                let probability = *count as f64 / total;
                *count as f64 * -probability.log(base)
            })
            .sum();
        code_length.ceil().max(1.0) as u128
    }

    /// Compress one prepared sequence into the exact representative fraction
    /// when it fits in `u128`, with a finite approximation for long inputs.
    pub fn compress(&self, data: &PreparedSequence) -> Rational {
        if let Some(output) = self.try_compress(data) {
            return output;
        }

        let size = self.approximate_size(data);
        let base = self.base.max(2) as u128;
        let numerator = (0..size)
            .try_fold(1u128, |value, _| value.checked_mul(base))
            .unwrap_or(u128::MAX);
        Rational::new(numerator, 1)
    }

    fn size(&self, data: &PreparedSequence) -> u128 {
        let Some(compressed) = self.try_compress(data) else {
            return self.approximate_size(data);
        };
        let numerator = compressed.numerator;
        if numerator == 0 {
            return 0;
        }
        let base = self.base.max(2) as u128;
        let mut power = 1u128;
        let mut exponent = 0u128;
        while power < numerator {
            power = power
                .checked_mul(base)
                .expect("arithmetic-coding size overflow");
            exponent += 1;
        }
        exponent
    }

    fn concatenated(sequences: &[PreparedSequence], order: &[usize]) -> PreparedSequence {
        order
            .iter()
            .flat_map(|&index| sequences[index].iter().cloned())
            .collect()
    }
}

impl Default for ArithNCD {
    fn default() -> Self {
        Self::new()
    }
}

impl Algorithm for ArithNCD {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }

        let mut concatenated_size = u128::MAX;
        for order in permutation_indices(sequences.len()) {
            let data = Self::concatenated(sequences, &order);
            concatenated_size = concatenated_size.min(self.size(&data));
        }

        let compressed_sizes: Vec<u128> = sequences.iter().map(|data| self.size(data)).collect();
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
