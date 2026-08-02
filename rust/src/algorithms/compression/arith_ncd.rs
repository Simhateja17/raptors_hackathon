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

    fn add(self, other: Self) -> Self {
        Self::new(
            self.numerator
                .checked_mul(other.denominator)
                .and_then(|value| {
                    other
                        .numerator
                        .checked_mul(self.denominator)
                        .and_then(|other_value| value.checked_add(other_value))
                })
                .expect("arithmetic-coding rational numerator overflow"),
            self.denominator
                .checked_mul(other.denominator)
                .expect("arithmetic-coding rational denominator overflow"),
        )
    }

    fn multiply(self, other: Self) -> Self {
        Self::new(
            self.numerator
                .checked_mul(other.numerator)
                .expect("arithmetic-coding rational numerator overflow"),
            self.denominator
                .checked_mul(other.denominator)
                .expect("arithmetic-coding rational denominator overflow"),
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
        let left = self
            .numerator
            .checked_mul(other.denominator)
            .expect("arithmetic-coding comparison overflow");
        let right = other
            .numerator
            .checked_mul(self.denominator)
            .expect("arithmetic-coding comparison overflow");
        left.cmp(&right)
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
        counts.sort_by(|left, right| right.1.cmp(&left.1));
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

    fn interval(&self, data: &PreparedSequence) -> (Rational, Rational) {
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
            start = start.add(probability_start.multiply(width));
            width = width.multiply(probability_width);
        }
        (start, start.add(width))
    }

    /// Compress one prepared sequence into the exact representative fraction.
    pub fn compress(&self, data: &PreparedSequence) -> Rational {
        let (start, end) = self.interval(data);
        let mut output = Rational::zero();
        let mut output_denominator = 1u128;

        while !(start <= output && output < end) {
            let numerator = start
                .numerator
                .checked_mul(output_denominator)
                .expect("arithmetic-coding output overflow")
                / start.denominator
                + 1;
            output = Rational::new(numerator, output_denominator);
            output_denominator = output_denominator
                .checked_mul(2)
                .expect("arithmetic-coding output denominator overflow");
        }
        output
    }

    fn size(&self, data: &PreparedSequence) -> u128 {
        let numerator = self.compress(data).numerator;
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
