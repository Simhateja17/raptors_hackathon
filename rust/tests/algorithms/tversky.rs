use textdistance_port::algorithms::token::tversky::{Tversky, TverskyError};
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn prepared(left: &str, right: &str) -> Vec<Vec<textdistance_port::Element>> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

#[test]
fn tversky_matches_jaccard_and_sorensen_parameters() {
    let sequences = prepared("test", "text");
    close(Tversky::default().call(&sequences), 0.6);

    let jaccard = Tversky::new(QValue::Elements, Some(vec![1.0, 1.0]), None, false, true);
    close(jaccard.call(&sequences), 0.6);

    let sorensen = Tversky::new(QValue::Elements, Some(vec![0.5, 0.5]), None, false, true);
    close(sorensen.call(&sequences), 0.75);
}

#[test]
fn tversky_preserves_bias_set_mode_and_empty_answers() {
    let biased = Tversky::new(
        QValue::Elements,
        Some(vec![2.0, 1.0]),
        Some(0.5),
        false,
        true,
    );
    close(biased.call(&prepared("ab", "ac")), 3.0 / 7.0);

    let set = Tversky::new(QValue::Elements, None, None, true, true);
    close(set.call(&prepared("aaaa", "aa")), 1.0);
    close(Tversky::default().call(&prepared("", "")), 1.0);
    close(Tversky::default().call(&prepared("", "abc")), 0.0);
}

#[test]
fn tversky_supports_three_sequences_and_reports_invalid_options() {
    let three = prepare_sequences(
        &[
            InputSequence::Text("abc".into()),
            InputSequence::Text("abd".into()),
            InputSequence::Text("abe".into()),
        ],
        QValue::Elements,
    )
    .unwrap();
    close(Tversky::default().call(&three), 2.0 / 5.0);
    let words = prepare_sequences(
        &[
            InputSequence::Text("one two".into()),
            InputSequence::Text("one three".into()),
        ],
        QValue::Words,
    )
    .unwrap();
    close(Tversky::default().call(&words), 1.0 / 3.0);

    let short = Tversky::new(
        QValue::Elements,
        Some(vec![1.0]),
        Some(0.5),
        false,
        true,
    );
    assert_eq!(
        short.try_similarity(&prepared("a", "b")),
        Err(TverskyError::MissingBiasedCoefficient)
    );

    let zero = Tversky::new(
        QValue::Elements,
        Some(vec![0.0, 0.0]),
        None,
        false,
        true,
    );
    assert_eq!(
        zero.try_similarity(&prepared("a", "b")),
        Err(TverskyError::ZeroDenominator)
    );
}
