use textdistance_port::algorithms::token::cosine::Cosine;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn prepared(left: &str, right: &str, qvalue: QValue) -> Vec<Vec<textdistance_port::Element>> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        qvalue,
    )
    .unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

#[test]
fn cosine_matches_source_examples() {
    let algorithm = Cosine::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::Elements)),
        3.0 / 4.0,
    );
    close(
        algorithm.call(&prepared("nelson", "neilsen", QValue::Elements)),
        5.0 / (6.0_f64 * 7.0).sqrt(),
    );
}

#[test]
fn cosine_preserves_repeated_tokens_and_set_mode() {
    let multiset = Cosine::default();
    let set = Cosine::new(QValue::Elements, true, true);
    let sequences = prepared("aaaa", "aa", QValue::Elements);
    close(multiset.call(&sequences), 1.0 / 2.0_f64.sqrt());
    close(set.call(&sequences), 1.0);
}

#[test]
fn cosine_supports_qgrams_and_more_than_two_sequences() {
    let algorithm = Cosine::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        1.0 / 3.0,
    );
    close(
        algorithm.call(&prepared("one two", "one three", QValue::Words)),
        0.5,
    );
    let three = prepare_sequences(
        &[
            InputSequence::Text("abc".into()),
            InputSequence::Text("abd".into()),
            InputSequence::Text("abe".into()),
        ],
        QValue::Elements,
    )
    .unwrap();
    close(algorithm.call(&three), 2.0 / 3.0);
}

#[test]
fn cosine_preserves_empty_and_equal_quick_answers() {
    let algorithm = Cosine::default();
    close(algorithm.call(&prepared("", "", QValue::Elements)), 1.0);
    close(algorithm.call(&prepared("", "abc", QValue::Elements)), 0.0);
}
