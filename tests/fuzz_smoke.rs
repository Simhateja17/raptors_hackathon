//! INT-05 fuzz/smoke harness for Suri's assigned algorithm packets: Overlap,
//! Tanimoto, RatcliffObershelp, BWTRLENCD, MRA, Prefix, Postfix, Length,
//! Identity, and Matrix.
//!
//! The repository's existing fuzz driver (`fuzzing/textdistance_fuzzer.py`)
//! is atheris-based and requires a Linux/macOS libFuzzer toolchain that is
//! unavailable on Windows; `cargo-fuzz` has the same platform limitation.
//! This file is the Windows-compatible substitute: a deterministic,
//! seed-recorded PRNG feeds randomized and fixed edge-case inputs directly to
//! the Rust implementations (no Python/PyO3 boundary involved) and every call
//! is wrapped in `catch_unwind` so a panic in any one algorithm is recorded
//! instead of aborting the run.

use std::panic::{self, AssertUnwindSafe};

use textdistance_port::algorithms::compression::bwtrle_ncd::BWTRLENCD;
use textdistance_port::algorithms::phonetic::mra::MRA;
use textdistance_port::algorithms::sequence::ratcliff_obershelp::RatcliffObershelp;
use textdistance_port::algorithms::simple::identity::Identity;
use textdistance_port::algorithms::simple::length::Length;
use textdistance_port::algorithms::simple::matrix::Matrix;
use textdistance_port::algorithms::simple::postfix::Postfix;
use textdistance_port::algorithms::simple::prefix::Prefix;
use textdistance_port::algorithms::token::overlap::Overlap;
use textdistance_port::algorithms::token::tanimoto::Tanimoto;
use textdistance_port::{
    output_distance, output_similarity, prepare_sequences, Algorithm, Element, InputSequence,
    OutputAlgorithm, PreparedSequence, QValue,
};

/// Deterministic xorshift64* PRNG so a failure is reproducible from the fixed
/// seed recorded below. Adding the `rand` crate would require editing
/// `Cargo.toml`, which is outside this file's ownership.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn range(&mut self, lo: usize, hi: usize) -> usize {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() as usize) % (hi - lo)
    }

    fn chance(&mut self, one_in: u32) -> bool {
        one_in > 0 && self.next_u32() % one_in == 0
    }
}

const SEED: u64 = 0x5EED_F00D_1234_5678;
const ITERATIONS: usize = 2000;

fn random_char(rng: &mut Rng) -> char {
    match rng.range(0, 6) {
        0 => (rng.range(0x20, 0x7F) as u8) as char, // ASCII printable
        1 => char::from_u32(rng.range(0xA0, 0x2FF) as u32).unwrap_or('?'), // Latin extended
        2 => '\u{0301}',                            // combining acute accent
        3 => char::from_u32(rng.range(0x1F600, 0x1F64F) as u32).unwrap_or('!'), // emoji
        4 => ' ',
        _ => char::from_u32(rng.range(0x4E00, 0x9FFF) as u32).unwrap_or('#'), // CJK
    }
}

fn random_string(rng: &mut Rng) -> String {
    if rng.chance(10) {
        return String::new();
    }
    let len = if rng.chance(50) {
        rng.range(200, 600) // occasional stress-length input
    } else {
        rng.range(0, 24)
    };
    if rng.chance(8) {
        let c = random_char(rng);
        return std::iter::repeat(c).take(len.max(1)).collect();
    }
    (0..len).map(|_| random_char(rng)).collect()
}

fn random_words(rng: &mut Rng) -> String {
    let words = rng.range(0, 6);
    (0..words)
        .map(|_| {
            random_string(rng)
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn random_ints(rng: &mut Rng) -> Vec<i64> {
    let len = rng.range(0, 16);
    (0..len)
        .map(|_| match rng.range(0, 5) {
            0 => 0,
            1 => i64::MAX,
            2 => i64::MIN,
            3 => (rng.next_u64() as i64) % 1000,
            _ => -((rng.next_u32() as i64) % 1000),
        })
        .collect()
}

fn random_bytes(rng: &mut Rng) -> Vec<u8> {
    let len = rng.range(0, 32);
    (0..len).map(|_| rng.next_u32() as u8).collect()
}

fn random_bools(rng: &mut Rng) -> Vec<bool> {
    let len = rng.range(0, 12);
    (0..len).map(|_| rng.chance(2)).collect()
}

fn random_qvalue(rng: &mut Rng, text_only: bool) -> QValue {
    match rng.range(0, if text_only { 3 } else { 2 }) {
        0 => QValue::Elements,
        1 => QValue::NGrams(rng.range(1, 4)),
        _ => QValue::Words,
    }
}

/// Builds between 0 and 3 random input sequences, mixing text, integers,
/// bytes, and booleans -- the full `InputSequence` surface.
fn random_inputs(rng: &mut Rng) -> (Vec<InputSequence>, QValue) {
    let count = rng.range(0, 4);
    let mut text_only = true;
    let mut inputs = Vec::with_capacity(count);
    for _ in 0..count {
        let input = match rng.range(0, 4) {
            0 => InputSequence::Text(if rng.chance(3) {
                random_words(rng)
            } else {
                random_string(rng)
            }),
            1 => {
                text_only = false;
                InputSequence::Integers(random_ints(rng))
            }
            2 => {
                text_only = false;
                InputSequence::Bytes(random_bytes(rng))
            }
            _ => {
                text_only = false;
                InputSequence::Booleans(random_bools(rng))
            }
        };
        inputs.push(input);
    }
    let qvalue = random_qvalue(rng, text_only);
    (inputs, qvalue)
}

struct Failure {
    algorithm: &'static str,
    description: String,
    message: String,
}

fn record_panic<F: FnOnce()>(
    failures: &mut Vec<Failure>,
    algorithm: &'static str,
    description: &str,
    f: F,
) {
    if let Err(payload) = panic::catch_unwind(AssertUnwindSafe(f)) {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_owned());
        failures.push(Failure {
            algorithm,
            description: description.to_owned(),
            message,
        });
    }
}

fn exercise_algorithm_trait<A: Algorithm>(
    failures: &mut Vec<Failure>,
    name: &'static str,
    algorithm: &A,
    sequences: &[PreparedSequence],
    description: &str,
) {
    record_panic(failures, name, description, || {
        let _ = algorithm.call(sequences);
        let _ = algorithm.similarity(sequences);
        let _ = algorithm.distance(sequences);
        let _ = algorithm.maximum(sequences);
        let _ = algorithm.normalized_distance(sequences);
        let _ = algorithm.normalized_similarity(sequences);
    });
}

fn exercise_output_algorithm<A: OutputAlgorithm>(
    failures: &mut Vec<Failure>,
    name: &'static str,
    algorithm: &A,
    sequences: &[PreparedSequence],
    description: &str,
) {
    record_panic(failures, name, description, || {
        let maximum = algorithm.output_maximum(sequences);
        let mode = algorithm.output_mode();
        if let Ok(output) = algorithm.output(sequences) {
            let _ = output_distance(&output, mode, maximum);
            let _ = output_similarity(&output, mode, maximum);
        }
    });
}

fn exercise_all(failures: &mut Vec<Failure>, sequences: &[PreparedSequence], description: &str) {
    exercise_algorithm_trait(
        failures,
        "Overlap",
        &Overlap::default(),
        sequences,
        description,
    );
    exercise_algorithm_trait(
        failures,
        "Tanimoto",
        &Tanimoto::default(),
        sequences,
        description,
    );
    exercise_algorithm_trait(
        failures,
        "RatcliffObershelp",
        &RatcliffObershelp::new(),
        sequences,
        description,
    );
    exercise_algorithm_trait(
        failures,
        "BWTRLENCD",
        &BWTRLENCD::new(),
        sequences,
        description,
    );
    // MRA's public contract is character text only.  The PyO3 adapter checks
    // that contract before calling the core; do not turn an expected invalid
    // input rejection into a native panic finding in this core smoke test.
    if sequences.iter().all(|sequence| {
        sequence
            .iter()
            .all(|element| matches!(element, Element::Char(_)))
    }) {
        exercise_algorithm_trait(failures, "MRA", &MRA::new(), sequences, description);
    }
    exercise_algorithm_trait(failures, "Length", &Length::new(), sequences, description);
    exercise_algorithm_trait(
        failures,
        "Identity",
        &Identity::new(),
        sequences,
        description,
    );
    exercise_algorithm_trait(
        failures,
        "Matrix",
        &Matrix::default(),
        sequences,
        description,
    );
    exercise_output_algorithm(failures, "Prefix", &Prefix::new(), sequences, description);
    exercise_output_algorithm(failures, "Postfix", &Postfix::new(), sequences, description);
}

/// Fixed edge cases from the PRD's minimum proof corpus (Section 10), run in
/// addition to the randomized loop.
fn edge_cases() -> Vec<(Vec<InputSequence>, QValue, &'static str)> {
    vec![
        (vec![], QValue::Elements, "zero sequences"),
        (
            vec![InputSequence::Text(String::new())],
            QValue::Elements,
            "single empty sequence",
        ),
        (
            vec![
                InputSequence::Text(String::new()),
                InputSequence::Text(String::new()),
            ],
            QValue::Elements,
            "empty/empty",
        ),
        (
            vec![
                InputSequence::Text(String::new()),
                InputSequence::Text("abc".into()),
            ],
            QValue::Elements,
            "empty/non-empty",
        ),
        (
            vec![
                InputSequence::Text("abc".into()),
                InputSequence::Text("abc".into()),
            ],
            QValue::Elements,
            "equal",
        ),
        (
            vec![
                InputSequence::Text("abc".into()),
                InputSequence::Text("xyz".into()),
            ],
            QValue::Elements,
            "completely different",
        ),
        (
            vec![
                InputSequence::Text("cafe\u{0301}".into()),
                InputSequence::Text("café".into()),
            ],
            QValue::Elements,
            "combining vs precomposed unicode",
        ),
        (
            vec![
                InputSequence::Text("😀😀😀".into()),
                InputSequence::Text("😀".into()),
            ],
            QValue::Elements,
            "emoji",
        ),
        (
            vec![
                InputSequence::Text("aaaaaaaaaaaaaaaaaaaa".into()),
                InputSequence::Text("aa".into()),
            ],
            QValue::Elements,
            "repeated character",
        ),
        (
            vec![
                InputSequence::Text("test".into()),
                InputSequence::Text("text".into()),
            ],
            QValue::NGrams(1),
            "qval=1",
        ),
        (
            vec![
                InputSequence::Text("test".into()),
                InputSequence::Text("text".into()),
            ],
            QValue::NGrams(2),
            "qval=2",
        ),
        (
            vec![
                InputSequence::Text("test".into()),
                InputSequence::Text("text".into()),
            ],
            QValue::NGrams(3),
            "qval=3",
        ),
        (
            vec![
                InputSequence::Text("one two".into()),
                InputSequence::Text("one three".into()),
            ],
            QValue::Words,
            "qval=None word split",
        ),
        (
            vec![
                InputSequence::Text("abc".into()),
                InputSequence::Text("abd".into()),
                InputSequence::Text("abe".into()),
            ],
            QValue::Elements,
            "three sequences",
        ),
        (
            vec![
                InputSequence::Integers(vec![1, 2, 3]),
                InputSequence::Integers(vec![1, 2, 4]),
            ],
            QValue::Elements,
            "integer sequences",
        ),
        (
            vec![
                InputSequence::Bytes(vec![0, 1, 255]),
                InputSequence::Bytes(vec![0, 1, 254]),
            ],
            QValue::Elements,
            "byte sequences",
        ),
        (
            vec![
                InputSequence::Booleans(vec![true, false]),
                InputSequence::Booleans(vec![true, true]),
            ],
            QValue::Elements,
            "boolean sequences",
        ),
    ]
}

#[test]
fn suri_packets_survive_randomized_and_edge_case_inputs() {
    // `catch_unwind` is exercised thousands of times below; silence the
    // default panic hook so a genuine failure isn't buried in stderr noise.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let mut failures = Vec::new();
    let mut rng = Rng(SEED);
    let mut cases_run = 0usize;

    for _ in 0..ITERATIONS {
        let (inputs, qvalue) = random_inputs(&mut rng);
        let description = format!("{inputs:?} qval={qvalue:?}");
        if let Ok(sequences) = prepare_sequences(&inputs, qvalue) {
            exercise_all(&mut failures, &sequences, &description);
            cases_run += 1;
        }
        // An `Err` here is an expected, deterministic rejection (e.g.
        // `Words` requested on non-text input), not a crash.
    }

    let edges = edge_cases();
    for (inputs, qvalue, label) in &edges {
        if let Ok(sequences) = prepare_sequences(inputs, *qvalue) {
            exercise_all(&mut failures, &sequences, label);
            cases_run += 1;
        }
    }

    panic::set_hook(previous_hook);

    eprintln!(
        "fuzz_smoke: seed=0x{SEED:016x} random_iterations={ITERATIONS} edge_cases={} prepared_cases_exercised={cases_run}",
        edges.len()
    );

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!(
                "PANIC in {}: input={} message={}",
                failure.algorithm, failure.description, failure.message
            );
        }
        panic!("{} panic(s) found during fuzz smoke test", failures.len());
    }
}
