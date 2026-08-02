use textdistance_port::algorithms::compression::arith_ncd::ArithNCD;
use textdistance_port::{
    prepare_sequences, Algorithm, Element, InputSequence, PreparedSequence, QValue,
};

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

#[test]
fn matches_original_arithmetic_ncd_fixtures() {
    let algorithm = ArithNCD::new();
    assert_close(algorithm.distance(&text_pair("test", "test")), 1.0);
    assert_close(
        algorithm.distance(&text_pair("test", "nani")),
        2.1666666666666665,
    );
}

#[test]
fn preserves_probability_order_and_exact_compressed_fraction() {
    let algorithm = ArithNCD::with_config(2, Some('\0'));
    let probabilities = algorithm.make_probs(&text_pair("lol", "lal"));

    let l_width = probabilities
        .iter()
        .find(|(element, _, _)| *element == Element::Char('l'))
        .map(|(_, _, width)| *width)
        .expect("l probability should exist");
    assert_eq!(
        l_width,
        textdistance_port::algorithms::compression::arith_ncd::Rational::new(4, 7)
    );

    let o_width = probabilities
        .iter()
        .find(|(element, _, _)| *element == Element::Char('o'))
        .map(|(_, _, width)| *width)
        .expect("o probability should exist");
    assert_eq!(
        o_width,
        textdistance_port::algorithms::compression::arith_ncd::Rational::new(1, 7)
    );

    let compressed = algorithm.compress(&text_sequence("BANANA"));
    assert_eq!(compressed.numerator, 1525);
}

#[test]
fn handles_empty_and_qgram_inputs_with_documented_configuration() {
    let algorithm = ArithNCD::new();
    assert_eq!(algorithm.distance(&[]), 0.0);

    let grams = prepare_sequences(
        &[
            InputSequence::Text("abcd".to_owned()),
            InputSequence::Text("abce".to_owned()),
        ],
        QValue::NGrams(2),
    )
    .expect("q-gram preparation should succeed");
    assert!(algorithm.distance(&grams).is_finite());
    assert_eq!(algorithm.base(), 2);
    assert_eq!(algorithm.terminator(), None);
}
