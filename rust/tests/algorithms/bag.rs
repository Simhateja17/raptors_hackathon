use textdistance_port::algorithms::token::bag::{Bag, BagError};
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

#[test]
fn bag_matches_source_examples() {
    let algorithm = Bag::default();
    for (left, right, expected) in [
        ("qwe", "qwe", 0.0),
        ("qwe", "erty", 3.0),
        ("qwe", "ewq", 0.0),
        ("qwe", "rtys", 4.0),
    ] {
        assert_eq!(
            algorithm.call(&prepared(left, right, QValue::Elements)),
            expected
        );
    }
}

#[test]
fn bag_preserves_multiset_set_and_qgram_behavior() {
    let multiset = Bag::default();
    let set = Bag {
        as_set: true,
        ..Bag::default()
    };
    let repeated = prepared("aaaa", "aa", QValue::Elements);
    assert_eq!(multiset.call(&repeated), 2.0);
    assert_eq!(set.call(&repeated), 1.0);
    assert_eq!(
        multiset.call(&prepared("test", "text", QValue::NGrams(2))),
        2.0
    );
    assert_eq!(
        multiset.call(&prepared("one two", "one three", QValue::Words)),
        1.0
    );
}

#[test]
fn bag_preserves_empty_and_equal_sequences_and_reports_no_arguments() {
    let algorithm = Bag::default();
    assert_eq!(algorithm.call(&prepared("", "", QValue::Elements)), 0.0);
    assert_eq!(algorithm.call(&prepared("same", "same", QValue::Elements)), 0.0);
    assert_eq!(algorithm.try_raw_score(&[]), Err(BagError::EmptyInputList));
}
