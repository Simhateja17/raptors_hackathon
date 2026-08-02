use textdistance_port::algorithms::compression::bwtrle_ncd::BWTRLENCD;
use textdistance_port::{prepare_sequences, Algorithm, Element, InputSequence, PreparedSequence, QValue};

fn text_sequence(value: &str) -> PreparedSequence {
    prepare_sequences(&[InputSequence::Text(value.to_owned())], QValue::Elements)
        .expect("text preparation should succeed")
        .remove(0)
}

fn text_pair(left: &str, right: &str) -> Vec<PreparedSequence> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "actual={actual}, expected={expected}"
    );
}

// Reference values below were captured by running the original Python
// `textdistance.bwtrle_ncd` / `textdistance.algorithms.compression_based.BWTRLENCD`
// directly. `matches_original_test_bwtrle_ncd_fixtures` reproduces
// `tests/original/test_compression/test_bwtrle_ncd.py::test_similarity`.

#[test]
fn matches_original_test_bwtrle_ncd_fixtures() {
    let algorithm = BWTRLENCD::new();
    assert_close(algorithm.call(&text_pair("test", "test")), 0.6);
    assert_close(algorithm.call(&text_pair("test", "nani")), 0.8);
}

#[test]
fn matches_common_monotonicity_and_normalization_fixtures() {
    // tests/original/test_compression/test_common.py::test_monotonicity
    let algorithm = BWTRLENCD::new();
    let same = algorithm.call(&text_pair("test", "test"));
    let similar = algorithm.call(&text_pair("test", "text"));
    let different = algorithm.call(&text_pair("test", "nani"));
    assert!(same <= similar);
    assert!(similar <= different);

    // call/distance/normalized_distance agree because maximum() is always 1.
    let sequences = text_pair("test", "nani");
    assert_close(algorithm.call(&sequences), algorithm.distance(&sequences));
    assert_close(
        algorithm.call(&sequences),
        algorithm.normalized_distance(&sequences),
    );
    assert_close(
        algorithm.normalized_similarity(&sequences) + algorithm.normalized_distance(&sequences),
        1.0,
    );
}

#[test]
fn matches_empty_and_single_character_fixtures() {
    let algorithm = BWTRLENCD::new();
    assert_close(algorithm.call(&text_pair("", "")), 0.0);
    assert_close(algorithm.call(&text_pair("", "abc")), 0.75);
    assert_close(algorithm.call(&text_pair("a", "a")), 0.5);
    assert_close(algorithm.call(&[]), 0.0);
}

#[test]
fn matches_repeated_character_run_length_fixtures() {
    // Exercises the RLE run-length>2 branch ("aaaa" -> "4a\0") and the
    // terminator-already-present short-circuit ("\0abc" stays untransformed).
    let algorithm = BWTRLENCD::new();
    assert_close(algorithm.call(&text_pair("aaaa", "aa")), 0.0);
    assert_close(algorithm.call(&text_pair("aaaaa", "aaaaa")), 1.0 / 3.0);
    assert_close(algorithm.call(&text_pair("banana", "bandana")), 0.5);
}

#[test]
fn matches_unicode_and_three_sequence_fixtures() {
    let algorithm = BWTRLENCD::new();
    assert_close(
        algorithm.call(&text_pair("cafe\u{0301}", "cafe")),
        0.8333333333333334,
    );

    let three = prepare_sequences(
        &[
            InputSequence::Text("abc".into()),
            InputSequence::Text("abd".into()),
            InputSequence::Text("abe".into()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    assert_close(algorithm.call(&three), 0.0);
}

#[test]
fn supports_custom_terminator_matching_python_constructor_argument() {
    let algorithm = BWTRLENCD::with_terminator(Element::Char('$'));
    assert_close(algorithm.call(&text_pair("test", "nani")), 0.8);
    assert_eq!(algorithm.terminator(), &Element::Char('$'));
}

#[test]
fn direct_compressed_size_matches_python_compress_output_lengths() {
    // Captured via `BWTRLENCD()._compress(s)` in the original Python package:
    // len('') -> 1 ('\x00'), len('aaaa') -> 3 ('4a\x00'),
    // len('aaaaaaaaaaa') -> 4 ('11a\x00'), len('\x00abc') -> 4 (untransformed).
    let algorithm = BWTRLENCD::new();
    assert_eq!(algorithm.size(&text_sequence("")), 1);
    assert_eq!(algorithm.size(&text_sequence("aaaa")), 3);
    assert_eq!(algorithm.size(&text_sequence("aaaaaaaaaaa")), 4);
    assert_eq!(algorithm.size(&text_sequence("\0abc")), 4);
    assert_eq!(algorithm.size(&text_sequence("banana")), 7);
}
