use textdistance_port::algorithms::simple::postfix::Postfix;
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
        .expect("Postfix output should be a sequence")
        .iter()
        .map(|element| match element {
            Element::Char(value) => *value,
            other => panic!("expected a character output, got {other:?}"),
        })
        .collect()
}

// Postfix has no dedicated file under `tests/original`; reference values
// below were captured by running `textdistance.Postfix()` directly against
// `textdistance/algorithms/simple.py`.

#[test]
fn matches_source_postfix_examples() {
    let algorithm = Postfix::new();
    for (left, right, expected) in [
        ("test", "text", "t"),
        ("testme", "textthis", ""),
        ("abc", "abc", "abc"),
        ("xyz", "abc", ""),
        ("cdab", "efab", "ab"),
        ("teria cafe", "big cafe", " cafe"),
    ] {
        let output = algorithm
            .output(&text_pair(left, right, QValue::Elements))
            .unwrap();
        assert_eq!(chars(&output), expected, "{left:?}/{right:?}");
    }
}

#[test]
fn handles_empty_single_and_multi_sequence_cases() {
    let algorithm = Postfix::new();

    assert_eq!(
        chars(&algorithm.output(&text_pair("", "abc", QValue::Elements)).unwrap()),
        ""
    );
    assert_eq!(
        chars(&algorithm.output(&text_pair("abc", "", QValue::Elements)).unwrap()),
        ""
    );

    // Python: zero-argument `Postfix()()` actually raises `IndexError`
    // (`Postfix.__call__` reads `sequences[0]` before checking for an empty
    // tuple, unlike `Prefix.__call__`). The Rust port does not replicate
    // that crash - `&[]` is a legitimate slice - and returns an empty
    // sequence instead, matching the `Prefix` port's precedent of not
    // reproducing source-language dynamic-typing artifacts.
    let empty = algorithm.output(&[]).unwrap();
    assert_eq!(empty.sequence().unwrap(), &Vec::<Element>::new());

    // Python: a single sequence is trivially its own postfix.
    let single = text_pair("abc", "abc", QValue::Elements);
    let single = &single[..1];
    assert_eq!(chars(&algorithm.output(single).unwrap()), "abc");

    let three = prepare_sequences(
        &[
            InputSequence::Text("wxyz".to_owned()),
            InputSequence::Text("axyz".to_owned()),
            InputSequence::Text("bxyz".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();
    assert_eq!(chars(&algorithm.output(&three).unwrap()), "xyz");
}

#[test]
fn similarity_distance_and_maximum_match_source_for_default_qval() {
    let algorithm = Postfix::new();

    let prepared = text_pair("test", "text", QValue::Elements);
    let output = algorithm.output(&prepared).unwrap();
    let maximum = algorithm.output_maximum(&prepared);
    assert_eq!(algorithm.output_mode(), ScoreMode::Similarity);
    assert_eq!(maximum, 4.0);
    assert_eq!(output_similarity(&output, ScoreMode::Similarity, maximum), 1.0);
    assert_eq!(output_distance(&output, ScoreMode::Similarity, maximum), 3.0);
    // normalized_distance / normalized_similarity, per `Base` in base.py.
    let normalized_distance =
        output_distance(&output, ScoreMode::Similarity, maximum) / maximum;
    assert_eq!(normalized_distance, 0.75);
    assert_eq!(1.0 - normalized_distance, 0.25);

    let prepared = text_pair("", "", QValue::Elements);
    let output = algorithm.output(&prepared).unwrap();
    let maximum = algorithm.output_maximum(&prepared);
    assert_eq!(maximum, 0.0);
    assert_eq!(output_distance(&output, ScoreMode::Similarity, maximum), 0.0);
}

#[test]
fn supports_qgrams_and_words() {
    let algorithm = Postfix::new();

    // qval=2: "testing" -> te,es,st,ti,in,ng; "resting" -> re,es,st,ti,in,ng.
    // Common trailing bigrams: es,st,ti,in,ng (5), diverging only at the
    // leading te/re pair.
    let grams = algorithm
        .output(&text_pair("testing", "resting", QValue::NGrams(2)))
        .unwrap();
    assert_eq!(grams.scalar_value(), 5.0);

    // qval=None: word-splitting. `call`/`similarity` (element count) match
    // the trailing-common-words semantics that `Postfix` is documented to
    // provide.
    let words = text_pair("one two three", "zero two three", QValue::Words);
    let output = algorithm.output(&words).unwrap();
    let result = output.sequence().unwrap();
    assert_eq!(
        result,
        &vec![
            Element::Text("two".to_owned()),
            Element::Text("three".to_owned()),
        ]
    );
    assert_eq!(output.scalar_value(), 2.0);

    // Known, source-level bug *not* replicated: in the original Python
    // package, `Postfix(qval=2)` and `Postfix(qval=None)` both raise
    // exceptions (`TypeError` / `AttributeError`) because `Postfix.__call__`
    // reverses raw sequences into plain `list`s *before* delegating to
    // `Prefix.__call__`, which then either tries to `.split()` a list
    // (qval=None) or `str.join` a list of q-gram tuples (qval>1). Neither
    // configuration is exercised by `tests/original/test_common.py` (which
    // only uses the default `qval=1` singleton `textdistance.postfix`), and
    // the Rust port already treats q-value preparation as happening before
    // algorithms ever run (see docs/API_CONTRACT.md), so this divergence is
    // deliberate and matches the `Prefix` port's precedent.
    assert_eq!(algorithm.output_maximum(&words), 3.0);
}
