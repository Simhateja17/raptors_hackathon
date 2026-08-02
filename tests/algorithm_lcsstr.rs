use textdistance_port::algorithms::sequence::lcsstr::LCSStr;
use textdistance_port::{AlgorithmOutput, Element, InputSequence, OutputAlgorithm, QValue};

fn text(value: &str) -> InputSequence {
    InputSequence::Text(value.to_owned())
}

fn chars(value: &str) -> Vec<Element> {
    value.chars().map(Element::Char).collect()
}

fn output_chars(value: AlgorithmOutput) -> Vec<Element> {
    match value {
        AlgorithmOutput::Sequence(sequence) => sequence,
        AlgorithmOutput::Score(_) => panic!("LCSStr must return a sequence"),
    }
}

#[test]
fn lcsstr_matches_prefix_middle_suffix_and_no_match() {
    let algorithm = LCSStr::default();

    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("ab"), text("abcd")])
                .unwrap()
        ),
        chars("ab")
    );
    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("abcd"), text("bc")])
                .unwrap()
        ),
        chars("bc")
    );
    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("abcd"), text("cd")])
                .unwrap()
        ),
        chars("cd")
    );
    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("abcd"), text("ef")])
                .unwrap()
        ),
        chars("")
    );
}

#[test]
fn lcsstr_preserves_sequence_matcher_ties() {
    let algorithm = LCSStr::default();

    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("ababa"), text("babab")])
                .unwrap()
        ),
        chars("abab")
    );
    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("abc"), text("axc")])
                .unwrap()
        ),
        chars("a")
    );
}

#[test]
fn lcsstr_uses_custom_search_for_multiple_and_long_inputs() {
    let algorithm = LCSStr::default();

    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text("abc"), text("axc"), text("zabc")])
                .unwrap()
        ),
        chars("a")
    );

    let long = "MYTEST".repeat(100);
    assert_eq!(
        output_chars(
            algorithm
                .output_inputs(&[text(&long), text("TEST")])
                .unwrap()
        ),
        chars("TEST")
    );
}

#[test]
fn lcsstr_preserves_empty_and_single_input_call_order() {
    let algorithm = LCSStr::from_python(Some(2), false);

    assert_eq!(
        output_chars(algorithm.output_inputs(&[text("")]).unwrap()),
        chars("")
    );
    assert_eq!(
        output_chars(algorithm.output_inputs(&[text("abcd")]).unwrap()),
        chars("abcd")
    );
    assert_eq!(
        output_chars(algorithm.output_inputs(&[text(""), text("abcd")]).unwrap()),
        chars("")
    );
}

#[test]
fn lcsstr_prepares_words_and_ngrams_only_for_multi_input_calls() {
    let words = LCSStr::from_python(Some(0), false)
        .output_inputs(&[text("one two"), text("one three")])
        .unwrap();
    assert_eq!(
        words,
        AlgorithmOutput::Sequence(vec![Element::Text("one".to_owned())])
    );

    let grams = LCSStr::from_python(Some(2), false)
        .output_inputs(&[text("test"), text("text")])
        .unwrap();
    assert_eq!(
        grams,
        AlgorithmOutput::Sequence(vec![Element::Gram(vec![
            Element::Char('t'),
            Element::Char('e'),
        ])])
    );
}

#[test]
fn lcsstr_exposes_similarity_output_contract() {
    let algorithm = LCSStr::new(QValue::Elements, false);
    let prepared = vec![chars("abcd"), chars("bc")];
    let output = algorithm.output(&prepared).unwrap();

    assert_eq!(output, AlgorithmOutput::Sequence(chars("bc")));
    assert_eq!(output.scalar_value(), 2.0);
    assert_eq!(algorithm.output_maximum(&prepared), 4.0);
}
