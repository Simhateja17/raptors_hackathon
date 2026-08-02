use textdistance_port::algorithms::sequence::lcsseq::LCSSeq;
use textdistance_port::{
    output_distance, output_similarity, prepare_sequences, AlgorithmOutput, Element, InputSequence,
    OutputAlgorithm, PreparedSequence, QValue, ScoreMode,
};

fn text_pair(left: &str, right: &str, qvalue: QValue) -> Vec<PreparedSequence> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        qvalue,
    )
    .expect("text preparation should succeed")
}

fn chars(output: &AlgorithmOutput) -> String {
    output
        .sequence()
        .expect("LCS output should be a sequence")
        .iter()
        .map(|element| match element {
            Element::Char(value) => *value,
            other => panic!("expected a character output, got {other:?}"),
        })
        .collect()
}

#[test]
fn matches_original_two_sequence_fixtures() {
    let algorithm = LCSSeq::new();
    for (left, right, expected) in [
        ("ab", "cd", ""),
        ("abcd", "abcd", "abcd"),
        ("test", "text", "tet"),
        ("thisisatest", "testing123testing", "tsitest"),
        ("DIXON", "DICKSONX", "DION"),
        ("random exponential", "layer activation", "ratia"),
        (&"a".repeat(80), &"a".repeat(80), &"a".repeat(80)),
        (&"a".repeat(80), &"b".repeat(80), ""),
    ] {
        let output = algorithm
            .output(&text_pair(left, right, QValue::Elements))
            .unwrap();
        assert_eq!(chars(&output), expected, "{left:?}/{right:?}");
    }
}

#[test]
fn matches_original_multi_sequence_fixtures() {
    let algorithm = LCSSeq::new();
    let prepared = prepare_sequences(
        &[
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("b".to_owned()),
            InputSequence::Text("c".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(chars(&algorithm.output(&prepared).unwrap()), "");

    let prepared = prepare_sequences(
        &[
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("a".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(chars(&algorithm.output(&prepared).unwrap()), "a");

    let prepared = prepare_sequences(
        &[
            InputSequence::Text("test".to_owned()),
            InputSequence::Text("text".to_owned()),
            InputSequence::Text("tempest".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(chars(&algorithm.output(&prepared).unwrap()), "tet");
}

#[test]
fn preserves_sequence_output_empty_unicode_qgram_integer_and_conversion_behavior() {
    let algorithm = LCSSeq::new();
    let empty = algorithm.output(&[]).unwrap();
    assert_eq!(empty.sequence().unwrap(), &Vec::<Element>::new());

    let unicode = algorithm
        .output(&text_pair("café", "cafe", QValue::Elements))
        .unwrap();
    assert_eq!(chars(&unicode), "caf");

    let grams = algorithm
        .output(&text_pair("abcd", "abce", QValue::NGrams(2)))
        .unwrap();
    assert_eq!(grams.scalar_value(), 2.0);

    let integers = prepare_sequences(
        &[
            InputSequence::Integers(vec![1, 2, 3]),
            InputSequence::Integers(vec![1, 4, 3]),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(algorithm.output(&integers).unwrap().scalar_value(), 2.0);

    let prepared = text_pair("test", "text", QValue::Elements);
    let output = algorithm.output(&prepared).unwrap();
    assert_eq!(algorithm.output_maximum(&prepared), 4.0);
    assert_eq!(algorithm.output_mode(), ScoreMode::Similarity);
    assert_eq!(output_similarity(&output, ScoreMode::Similarity, 4.0), 3.0);
    assert_eq!(output_distance(&output, ScoreMode::Similarity, 4.0), 1.0);
}
