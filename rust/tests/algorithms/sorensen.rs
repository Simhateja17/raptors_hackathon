use textdistance_port::algorithms::token::sorensen::{dice, Sorensen};
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
fn sorensen_matches_source_formula() {
    let algorithm = Sorensen::default();
    let sequences = prepared("test", "text", QValue::Elements);
    close(algorithm.call(&sequences), 6.0 / 8.0);
    close(algorithm.distance(&sequences), 2.0 / 8.0);
    close(algorithm.normalized_similarity(&sequences), 6.0 / 8.0);
}

#[test]
fn sorensen_preserves_repeated_tokens_and_set_mode() {
    let multiset = Sorensen::default();
    let set = Sorensen::new(QValue::Elements, true, true);
    let sequences = prepared("aaaa", "aa", QValue::Elements);
    close(multiset.call(&sequences), 4.0 / 6.0);
    close(set.call(&sequences), 1.0);
}

#[test]
fn sorensen_supports_qgrams_and_alias_constructors() {
    let algorithm = Sorensen::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        1.0 / 3.0,
    );
    close(
        algorithm.call(&prepared("one two", "one three", QValue::Words)),
        0.5,
    );
    close(
        dice().call(&prepared("test", "text", QValue::Elements)),
        0.75,
    );
}

#[test]
fn sorensen_preserves_empty_and_equal_quick_answers() {
    let algorithm = Sorensen::default();
    close(algorithm.call(&prepared("", "", QValue::Elements)), 1.0);
    close(algorithm.call(&prepared("", "abc", QValue::Elements)), 0.0);
}
