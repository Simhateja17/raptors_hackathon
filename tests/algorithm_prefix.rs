use textdistance_port::algorithms::simple::prefix::Prefix;
use textdistance_port::{
    output_distance, output_similarity, prepare_sequences, AlgorithmOutput, Element,
    InputSequence, OutputAlgorithm, PreparedSequence, QValue, ScoreMode,
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
        .expect("Prefix output should be a sequence")
        .iter()
        .map(|element| match element {
            Element::Char(value) => *value,
            other => panic!("expected a character output, got {other:?}"),
        })
        .collect()
}

// Prefix has no dedicated file under `tests/original`; reference values below
// were captured by running `textdistance.Prefix()` (and `Prefix(qval=None)`,
// `Prefix(qval=2)`) directly against `textdistance/algorithms/simple.py`.

#[test]
fn matches_source_prefix_examples() {
    let algorithm = Prefix::new();
    for (left, right, expected) in [
        ("test", "text", "te"),
        ("testme", "textthis", "te"),
        ("abc", "abc", "abc"),
        ("xyz", "abc", ""),
        ("ab", "abcdef", "ab"),
        ("café", "cafeteria", "caf"),
    ] {
        let output = algorithm
            .output(&text_pair(left, right, QValue::Elements))
            .unwrap();
        assert_eq!(chars(&output), expected, "{left:?}/{right:?}");
    }
}

#[test]
fn handles_empty_single_and_multi_sequence_cases() {
    let algorithm = Prefix::new();

    assert_eq!(
        chars(&algorithm.output(&text_pair("", "abc", QValue::Elements)).unwrap()),
        ""
    );
    assert_eq!(
        chars(&algorithm.output(&text_pair("abc", "", QValue::Elements)).unwrap()),
        ""
    );

    // Python: `Prefix()()` with zero sequences returns `''`.
    let empty = algorithm.output(&[]).unwrap();
    assert_eq!(empty.sequence().unwrap(), &Vec::<Element>::new());

    // Python: a single sequence is trivially its own prefix.
    let single = text_pair("abc", "abc", QValue::Elements);
    let single = &single[..1];
    assert_eq!(chars(&algorithm.output(single).unwrap()), "abc");

    let three = prepare_sequences(
        &[
            InputSequence::Text("abcd".to_owned()),
            InputSequence::Text("abce".to_owned()),
            InputSequence::Text("abcf".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(chars(&algorithm.output(&three).unwrap()), "abc");
}

#[test]
fn similarity_distance_and_maximum_match_source_for_default_qval() {
    let algorithm = Prefix::new();

    let prepared = text_pair("test", "text", QValue::Elements);
    let output = algorithm.output(&prepared).unwrap();
    let maximum = algorithm.output_maximum(&prepared);
    assert_eq!(algorithm.output_mode(), ScoreMode::Similarity);
    assert_eq!(maximum, 4.0);
    assert_eq!(output_similarity(&output, ScoreMode::Similarity, maximum), 2.0);
    assert_eq!(output_distance(&output, ScoreMode::Similarity, maximum), 2.0);
    // normalized_distance / normalized_similarity, per `Base` in base.py.
    let normalized_distance =
        output_distance(&output, ScoreMode::Similarity, maximum) / maximum;
    assert_eq!(normalized_distance, 0.5);
    assert_eq!(1.0 - normalized_distance, 0.5);

    let prepared = text_pair("", "", QValue::Elements);
    let output = algorithm.output(&prepared).unwrap();
    let maximum = algorithm.output_maximum(&prepared);
    assert_eq!(maximum, 0.0);
    assert_eq!(output_distance(&output, ScoreMode::Similarity, maximum), 0.0);
}

#[test]
fn supports_qgrams_and_words() {
    let algorithm = Prefix::new();

    // qval=2: "testing"/"tester" share the leading bigrams te, es, st.
    let grams = algorithm
        .output(&text_pair("testing", "tester", QValue::NGrams(2)))
        .unwrap();
    assert_eq!(grams.scalar_value(), 3.0);

    // qval=None: word-splitting. `call` and `similarity` (element count)
    // match the Python source exactly.
    let words = text_pair("one two three", "one two four", QValue::Words);
    let output = algorithm.output(&words).unwrap();
    let result = output.sequence().unwrap();
    assert_eq!(
        result,
        &vec![
            Element::Text("one".to_owned()),
            Element::Text("two".to_owned()),
        ]
    );
    assert_eq!(output.scalar_value(), 2.0);

    // Known, contract-level divergence: Python's `Base.maximum` uses the
    // *raw* (pre-qval) sequence length (13, from "one two three"), but this
    // port's `Algorithm`/`OutputAlgorithm` contract only ever hands
    // algorithms prepared sequences (see docs/API_CONTRACT.md: "All
    // algorithm implementations receive prepared sequences. They must not
    // reimplement q-value preparation."). `output_maximum` therefore uses
    // the prepared (word-count) length, matching the established `LCSSeq`
    // precedent. This only affects `distance`/`maximum`/`normalized_*` under
    // non-default `qval`; `call`/`similarity` match exactly for every qval.
    assert_eq!(algorithm.output_maximum(&words), 3.0);
}
