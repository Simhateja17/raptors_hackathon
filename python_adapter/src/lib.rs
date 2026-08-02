//! PyO3 compatibility boundary for the Rust TextDistance implementation.
//!
//! The adapter is intentionally thin: it validates the supported Python input
//! domain, prepares inputs through the shared Rust core, dispatches to the
//! existing algorithm packets, and exposes the common source-level methods.
//! It never imports or calls the original Python implementation.

use std::collections::BTreeMap;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};

use crate::algorithms::{
    arith_ncd::ArithNCD, bwtrle_ncd::BWTRLENCD, bz2_ncd::Bz2Ncd, cosine::Cosine,
    damerau_levenshtein::DamerauLevenshtein, editex::Editex, entropy_ncd::EntropyNcd, gotoh::Gotoh,
    hamming::Hamming, identity::Identity, jaccard::Jaccard, jaro::Jaro, jaro_winkler::JaroWinkler,
    lcsseq::LCSSeq, lcsstr::LCSStr, length::Length, levenshtein::Levenshtein, lzma_ncd::LzmaNcd,
    matrix::Matrix, mlipns::MLIPNS, monge_elkan::MongeElkan, mra::MRA,
    needleman_wunsch::NeedlemanWunsch, overlap::Overlap, postfix::Postfix, prefix::Prefix,
    ratcliff_obershelp::RatcliffObershelp, rle_ncd::RleNcd, smith_waterman::SmithWaterman,
    sorensen::Sorensen, sqrt_ncd::SqrtNcd, strcmp95::StrCmp95, tanimoto::Tanimoto,
    tversky::Tversky, zlib_ncd::ZlibNcd,
};
use crate::core::{
    output_distance, output_similarity, prepare_sequences, Algorithm, AlgorithmError,
    AlgorithmOutput, Element, InputSequence, OutputAlgorithm, PreparedSequence, QValue, ScoreMode,
    Sequence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AlgorithmKind {
    Levenshtein,
    DamerauLevenshtein,
    NeedlemanWunsch,
    SmithWaterman,
    Gotoh,
    StrCmp95,
    Mlipns,
    Jaro,
    JaroWinkler,
    Hamming,
    Jaccard,
    Sorensen,
    Tversky,
    Cosine,
    MongeElkan,
    Bag,
    Overlap,
    Tanimoto,
    LCSSeq,
    LCSStr,
    RatcliffObershelp,
    ArithNcd,
    RleNcd,
    BwtrleNcd,
    SqrtNcd,
    EntropyNcd,
    Bz2Ncd,
    LzmaNcd,
    ZlibNcd,
    Editex,
    Mra,
    Prefix,
    Postfix,
    Length,
    Identity,
    Matrix,
}

impl AlgorithmKind {
    fn parse(name: &str) -> PyResult<Self> {
        let normalized = name.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        let kind = match normalized.as_str() {
            "levenshtein" => Self::Levenshtein,
            "damerau" | "damerau_levenshtein" => Self::DamerauLevenshtein,
            "needleman_wunsch" => Self::NeedlemanWunsch,
            "smith_waterman" => Self::SmithWaterman,
            "gotoh" => Self::Gotoh,
            "strcmp95" | "str_cmp95" => Self::StrCmp95,
            "mlipns" => Self::Mlipns,
            "jaro" => Self::Jaro,
            "jaro_winkler" => Self::JaroWinkler,
            "hamming" => Self::Hamming,
            "jaccard" => Self::Jaccard,
            "sorensen" | "sorensen_dice" | "dice" => Self::Sorensen,
            "tversky" => Self::Tversky,
            "cosine" => Self::Cosine,
            "monge_elkan" => Self::MongeElkan,
            "bag" => Self::Bag,
            "overlap" => Self::Overlap,
            "tanimoto" => Self::Tanimoto,
            "lcsseq" | "lcs_seq" => Self::LCSSeq,
            "lcsstr" | "lcs_str" => Self::LCSStr,
            "ratcliff_obershelp" | "ratcliff" => Self::RatcliffObershelp,
            "arith_ncd" | "arithncd" => Self::ArithNcd,
            "rle_ncd" | "rlen_cd" | "rlencd" => Self::RleNcd,
            "bwtrle_ncd" | "bwtrlen_cd" | "bwtrlen c d" => Self::BwtrleNcd,
            "sqrt_ncd" | "sqrtncd" => Self::SqrtNcd,
            "entropy_ncd" | "entropyncd" => Self::EntropyNcd,
            "bz2_ncd" | "bz2ncd" => Self::Bz2Ncd,
            "lzma_ncd" | "lzman_cd" | "lzma ncd" => Self::LzmaNcd,
            "zlib_ncd" | "zlibncd" => Self::ZlibNcd,
            "editex" => Self::Editex,
            "mra" => Self::Mra,
            "prefix" => Self::Prefix,
            "postfix" => Self::Postfix,
            "length" => Self::Length,
            "identity" => Self::Identity,
            "matrix" => Self::Matrix,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "unknown Rust TextDistance algorithm: {name}"
                )))
            }
        };
        Ok(kind)
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Levenshtein => "levenshtein",
            Self::DamerauLevenshtein => "damerau_levenshtein",
            Self::NeedlemanWunsch => "needleman_wunsch",
            Self::SmithWaterman => "smith_waterman",
            Self::Gotoh => "gotoh",
            Self::StrCmp95 => "strcmp95",
            Self::Mlipns => "mlipns",
            Self::Jaro => "jaro",
            Self::JaroWinkler => "jaro_winkler",
            Self::Hamming => "hamming",
            Self::Jaccard => "jaccard",
            Self::Sorensen => "sorensen",
            Self::Tversky => "tversky",
            Self::Cosine => "cosine",
            Self::MongeElkan => "monge_elkan",
            Self::Bag => "bag",
            Self::Overlap => "overlap",
            Self::Tanimoto => "tanimoto",
            Self::LCSSeq => "lcsseq",
            Self::LCSStr => "lcsstr",
            Self::RatcliffObershelp => "ratcliff_obershelp",
            Self::ArithNcd => "arith_ncd",
            Self::RleNcd => "rle_ncd",
            Self::BwtrleNcd => "bwtrle_ncd",
            Self::SqrtNcd => "sqrt_ncd",
            Self::EntropyNcd => "entropy_ncd",
            Self::Bz2Ncd => "bz2_ncd",
            Self::LzmaNcd => "lzma_ncd",
            Self::ZlibNcd => "zlib_ncd",
            Self::Editex => "editex",
            Self::Mra => "mra",
            Self::Prefix => "prefix",
            Self::Postfix => "postfix",
            Self::Length => "length",
            Self::Identity => "identity",
            Self::Matrix => "matrix",
        }
    }
}

#[derive(Clone, Debug)]
struct Evaluated {
    output: AlgorithmOutput,
    maximum: f64,
    mode: ScoreMode,
}

/// One Python-facing handle for a named Rust algorithm.
#[pyclass(module = "textdistance_port")]
pub struct RustAlgorithm {
    kind: AlgorithmKind,
    qval: Option<usize>,
    external: bool,
    as_set: bool,
    truncate: bool,
    symmetric: bool,
    restricted: bool,
    local: bool,
    base: f64,
    coef: f64,
    bias: Option<f64>,
    ks: Option<Vec<f64>>,
    terminator: Option<char>,
    gap_cost: f64,
    gap_open: f64,
    gap_ext: f64,
    long_tolerance: bool,
    prefix_weight: f64,
    long_strings: bool,
    threshold: f64,
    maxmismatches: usize,
    match_cost: i64,
    group_cost: i64,
    mismatch_cost: i64,
    matrix: Option<BTreeMap<Vec<PreparedSequence>, f64>>,
    monge_comparator: String,
}

impl RustAlgorithm {
    fn from_options(
        name: &str,
        qval: Option<usize>,
        external: bool,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        reject_python_callbacks(options)?;

        Ok(Self {
            kind: AlgorithmKind::parse(name)?,
            qval,
            external,
            as_set: option_bool(options, "as_set", false)?,
            truncate: option_bool(options, "truncate", false)?,
            symmetric: option_bool(options, "symmetric", false)?,
            restricted: option_bool(options, "restricted", true)?,
            local: option_bool(options, "local", false)?,
            base: option_f64(options, "base", 2.0)?,
            coef: option_f64(options, "coef", 1.0)?,
            bias: option_optional_f64(options, "bias")?,
            ks: option_vec_f64(options, "ks")?,
            terminator: option_char(options, "terminator")?,
            gap_cost: option_f64(options, "gap_cost", 1.0)?,
            gap_open: option_f64(options, "gap_open", 1.0)?,
            gap_ext: option_f64(options, "gap_ext", 0.4)?,
            long_tolerance: option_bool(options, "long_tolerance", false)?,
            prefix_weight: option_f64(options, "prefix_weight", 0.1)?,
            long_strings: option_bool(options, "long_strings", false)?,
            threshold: option_f64(options, "threshold", 0.25)?,
            maxmismatches: option_usize(options, "maxmismatches", 2)?,
            match_cost: option_i64(options, "match_cost", 0)?,
            group_cost: option_i64(options, "group_cost", 1)?,
            mismatch_cost: option_i64(options, "mismatch_cost", 2)?,
            matrix: option_matrix(options)?,
            monge_comparator: option_string(
                options,
                "algorithm",
                "damerau_levenshtein".to_owned(),
            )?,
        })
    }

    fn qvalue(&self) -> QValue {
        QValue::from_python(self.qval)
    }

    fn prepare(&self, inputs: &[InputSequence]) -> PyResult<Vec<PreparedSequence>> {
        let qvalue = match self.kind {
            AlgorithmKind::Matrix
            | AlgorithmKind::Mra
            | AlgorithmKind::Editex
            | AlgorithmKind::BwtrleNcd
            | AlgorithmKind::Bz2Ncd
            | AlgorithmKind::LzmaNcd
            | AlgorithmKind::ZlibNcd => QValue::Elements,
            _ => self.qvalue(),
        };
        prepare_sequences(inputs, qvalue).map_err(input_error)
    }

    fn evaluate(
        &self,
        inputs: &[InputSequence],
        prepared: &[PreparedSequence],
    ) -> PyResult<Evaluated> {
        match self.kind {
            AlgorithmKind::Levenshtein => Ok(numeric(&Levenshtein::new(), prepared)),
            AlgorithmKind::DamerauLevenshtein => Ok(numeric(
                &DamerauLevenshtein::with_restricted(self.restricted),
                prepared,
            )),
            AlgorithmKind::NeedlemanWunsch => Ok(numeric(
                &NeedlemanWunsch::with_gap_cost(self.gap_cost),
                prepared,
            )),
            AlgorithmKind::SmithWaterman => Ok(numeric(
                &SmithWaterman::with_gap_cost(self.gap_cost),
                prepared,
            )),
            AlgorithmKind::Gotoh => Ok(numeric(
                &Gotoh::with_gap_costs(self.gap_open, self.gap_ext),
                prepared,
            )),
            AlgorithmKind::StrCmp95 => Ok(numeric(
                &StrCmp95::with_long_strings(self.long_strings),
                prepared,
            )),
            AlgorithmKind::Mlipns => Ok(numeric(
                &MLIPNS::with_params(self.threshold, self.maxmismatches),
                prepared,
            )),
            AlgorithmKind::Jaro => Ok(numeric(
                &Jaro {
                    long_tolerance: self.long_tolerance,
                    external: self.external,
                },
                prepared,
            )),
            AlgorithmKind::JaroWinkler => Ok(numeric(
                &JaroWinkler {
                    long_tolerance: self.long_tolerance,
                    prefix_weight: self.prefix_weight,
                    external: self.external,
                },
                prepared,
            )),
            AlgorithmKind::Hamming => Ok(numeric(
                &Hamming::from_python(self.qval, self.truncate, self.external),
                prepared,
            )),
            AlgorithmKind::Jaccard => Ok(numeric(
                &Jaccard::from_python(self.qval, self.as_set, self.external),
                prepared,
            )),
            AlgorithmKind::Sorensen => Ok(numeric(
                &Sorensen::from_python(self.qval, self.as_set, self.external),
                prepared,
            )),
            AlgorithmKind::Tversky => {
                let algorithm = Tversky::from_python(
                    self.qval,
                    self.ks.clone(),
                    self.bias,
                    self.as_set,
                    self.external,
                );
                let score = algorithm
                    .try_similarity(prepared)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?;
                Ok(evaluated_score(&algorithm, score, prepared))
            }
            AlgorithmKind::Cosine => Ok(numeric(
                &Cosine::from_python(self.qval, self.as_set, self.external),
                prepared,
            )),
            AlgorithmKind::MongeElkan => match self.monge_comparator.as_str() {
                "jaro" => Ok(numeric(
                    &MongeElkan::from_python(
                        Jaro::default(),
                        self.symmetric,
                        self.qval,
                        self.external,
                    ),
                    prepared,
                )),
                "jaro_winkler" => Ok(numeric(
                    &MongeElkan::from_python(
                        JaroWinkler::default(),
                        self.symmetric,
                        self.qval,
                        self.external,
                    ),
                    prepared,
                )),
                "damerau_levenshtein" | "damerau" => Ok(numeric(
                    &MongeElkan::from_python(
                        DamerauLevenshtein::default(),
                        self.symmetric,
                        self.qval,
                        self.external,
                    ),
                    prepared,
                )),
                other => Err(PyValueError::new_err(format!(
                    "unsupported built-in Monge-Elkan comparator: {other}"
                ))),
            },
            AlgorithmKind::Bag => {
                let algorithm =
                    crate::algorithms::bag::Bag::from_python(self.qval, self.as_set, self.external);
                let score = algorithm
                    .try_raw_score(prepared)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?;
                Ok(evaluated_score(&algorithm, score, prepared))
            }
            AlgorithmKind::Overlap => Ok(numeric(
                &Overlap::from_python(self.qval, self.as_set, self.external),
                prepared,
            )),
            AlgorithmKind::Tanimoto => Ok(numeric(
                &Tanimoto::from_python(self.qval, self.as_set, self.external),
                prepared,
            )),
            AlgorithmKind::LCSSeq => output(&LCSSeq::new(), prepared),
            AlgorithmKind::LCSStr => {
                require_string_inputs(inputs, "LCSStr")?;
                let algorithm = LCSStr::from_python(self.qval, self.external);
                let value = algorithm.output_inputs(inputs).map_err(algorithm_error)?;
                Ok(Evaluated {
                    output: value,
                    maximum: algorithm.output_maximum(prepared),
                    mode: algorithm.output_mode(),
                })
            }
            AlgorithmKind::RatcliffObershelp => Ok(numeric(&RatcliffObershelp::new(), prepared)),
            AlgorithmKind::ArithNcd => Ok(numeric(
                &ArithNCD::with_config(self.base as u32, self.terminator),
                prepared,
            )),
            AlgorithmKind::RleNcd => {
                let algorithm = RleNcd::from_python(self.qval);
                let score = algorithm
                    .try_raw_score(prepared)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?;
                Ok(evaluated_score(&algorithm, score, prepared))
            }
            AlgorithmKind::BwtrleNcd => {
                require_string_inputs(inputs, "BWTRLE NCD")?;
                let algorithm =
                    BWTRLENCD::with_terminator(Element::Char(self.terminator.unwrap_or('\0')));
                Ok(numeric(&algorithm, prepared))
            }
            AlgorithmKind::SqrtNcd => Ok(numeric(&SqrtNcd, prepared)),
            AlgorithmKind::EntropyNcd => Ok(numeric(
                &EntropyNcd {
                    coef: self.coef,
                    base: self.base,
                },
                prepared,
            )),
            AlgorithmKind::Bz2Ncd => {
                ensure_binary_inputs(inputs, "BZ2 NCD")?;
                Ok(numeric(&Bz2Ncd, prepared))
            }
            AlgorithmKind::LzmaNcd => {
                ensure_binary_inputs(inputs, "LZMA NCD")?;
                Ok(numeric(&LzmaNcd, prepared))
            }
            AlgorithmKind::ZlibNcd => {
                ensure_binary_inputs(inputs, "ZLIB NCD")?;
                Ok(numeric(&ZlibNcd, prepared))
            }
            AlgorithmKind::Editex => {
                require_string_inputs(inputs, "Editex")?;
                Ok(numeric(
                    &Editex::new(
                        self.local,
                        self.match_cost,
                        self.group_cost,
                        self.mismatch_cost,
                        self.external,
                    ),
                    prepared,
                ))
            }
            AlgorithmKind::Mra => {
                require_string_inputs(inputs, "MRA")?;
                Ok(numeric(&MRA::new(), prepared))
            }
            AlgorithmKind::Prefix => output(&Prefix::new(), prepared),
            AlgorithmKind::Postfix => output(&Postfix::new(), prepared),
            AlgorithmKind::Length => Ok(numeric(&Length::new(), prepared)),
            AlgorithmKind::Identity => Ok(numeric(&Identity::new(), prepared)),
            AlgorithmKind::Matrix => Ok(numeric(
                &Matrix::new(
                    self.matrix.clone(),
                    self.mismatch_cost as f64,
                    self.match_cost as f64,
                    self.symmetric,
                    self.external,
                ),
                prepared,
            )),
        }
    }

    fn evaluate_from_args(
        &self,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<(Vec<InputSequence>, Evaluated)> {
        let inputs = extract_inputs(args)?;
        let prepared = self.prepare(&inputs)?;
        let evaluated = self.evaluate(&inputs, &prepared)?;
        Ok((inputs, evaluated))
    }

    fn numeric_method(&self, args: &Bound<'_, PyTuple>, method: NumericMethod) -> PyResult<f64> {
        let (_, evaluated) = self.evaluate_from_args(args)?;
        Ok(match method {
            NumericMethod::Distance => {
                output_distance(&evaluated.output, evaluated.mode, evaluated.maximum)
            }
            NumericMethod::Similarity => {
                output_similarity(&evaluated.output, evaluated.mode, evaluated.maximum)
            }
            NumericMethod::NormalizedDistance => {
                let distance =
                    output_distance(&evaluated.output, evaluated.mode, evaluated.maximum);
                if evaluated.maximum == 0.0 {
                    0.0
                } else {
                    distance / evaluated.maximum
                }
            }
            NumericMethod::NormalizedSimilarity => {
                let distance =
                    output_distance(&evaluated.output, evaluated.mode, evaluated.maximum);
                if evaluated.maximum == 0.0 {
                    1.0
                } else {
                    1.0 - distance / evaluated.maximum
                }
            }
            NumericMethod::Maximum => evaluated.maximum,
        })
    }
}

#[derive(Clone, Copy)]
enum NumericMethod {
    Distance,
    Similarity,
    NormalizedDistance,
    NormalizedSimilarity,
    Maximum,
}

#[pymethods]
impl RustAlgorithm {
    #[new]
    #[pyo3(signature = (name, qval=1, external=true, **options))]
    fn new(
        name: &str,
        qval: Option<usize>,
        external: bool,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        Self::from_options(name, qval, external, options)
    }

    #[pyo3(signature = (*sequences))]
    fn __call__(&self, py: Python<'_>, sequences: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let (inputs, evaluated) = self.evaluate_from_args(sequences)?;
        match &evaluated.output {
            AlgorithmOutput::Score(value) => Ok(PyFloat::new(py, *value).into_any().unbind()),
            AlgorithmOutput::Sequence(sequence) => sequence_to_py(
                py,
                sequence,
                &inputs,
                self.qval == Some(1) || self.kind == AlgorithmKind::LCSStr,
            ),
        }
    }

    #[pyo3(signature = (*sequences))]
    fn distance(&self, sequences: &Bound<'_, PyTuple>) -> PyResult<f64> {
        self.numeric_method(sequences, NumericMethod::Distance)
    }

    #[pyo3(signature = (*sequences))]
    fn similarity(&self, sequences: &Bound<'_, PyTuple>) -> PyResult<f64> {
        self.numeric_method(sequences, NumericMethod::Similarity)
    }

    #[pyo3(signature = (*sequences))]
    fn normalized_distance(&self, sequences: &Bound<'_, PyTuple>) -> PyResult<f64> {
        self.numeric_method(sequences, NumericMethod::NormalizedDistance)
    }

    #[pyo3(signature = (*sequences))]
    fn normalized_similarity(&self, sequences: &Bound<'_, PyTuple>) -> PyResult<f64> {
        self.numeric_method(sequences, NumericMethod::NormalizedSimilarity)
    }

    #[pyo3(signature = (*sequences))]
    fn maximum(&self, sequences: &Bound<'_, PyTuple>) -> PyResult<f64> {
        self.numeric_method(sequences, NumericMethod::Maximum)
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.kind.name()
    }

    #[getter]
    fn qval(&self) -> Option<usize> {
        self.qval
    }

    fn __repr__(&self) -> String {
        format!(
            "RustAlgorithm(name='{}', qval={:?}, external={})",
            self.kind.name(),
            self.qval,
            self.external
        )
    }
}

#[pyfunction]
#[pyo3(signature = (name, qval=1, external=true, **options))]
fn algorithm(
    name: &str,
    qval: Option<usize>,
    external: bool,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<RustAlgorithm> {
    RustAlgorithm::from_options(name, qval, external, options)
}

#[pymodule]
fn textdistance_port(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RustAlgorithm>()?;
    module.add_function(wrap_pyfunction!(algorithm, module)?)?;
    module.add("__version__", crate::VERSION)?;
    Ok(())
}

fn numeric<A: Algorithm>(algorithm: &A, sequences: &[PreparedSequence]) -> Evaluated {
    evaluated_score(algorithm, Algorithm::call(algorithm, sequences), sequences)
}

fn evaluated_score<A: Algorithm>(
    algorithm: &A,
    score: f64,
    sequences: &[PreparedSequence],
) -> Evaluated {
    Evaluated {
        output: AlgorithmOutput::Score(score),
        maximum: Algorithm::maximum(algorithm, sequences),
        mode: Algorithm::score_mode(algorithm),
    }
}

fn output<A: OutputAlgorithm>(
    algorithm: &A,
    sequences: &[PreparedSequence],
) -> PyResult<Evaluated> {
    Ok(Evaluated {
        output: algorithm.output(sequences).map_err(algorithm_error)?,
        maximum: algorithm.output_maximum(sequences),
        mode: algorithm.output_mode(),
    })
}

fn extract_inputs(args: &Bound<'_, PyTuple>) -> PyResult<Vec<InputSequence>> {
    args.iter().map(|value| input_from_object(&value)).collect()
}

fn input_from_object(value: &Bound<'_, PyAny>) -> PyResult<InputSequence> {
    if let Ok(text) = value.extract::<String>() {
        return Ok(InputSequence::Text(text));
    }
    if let Ok(bytes) = value.cast::<PyBytes>() {
        return Ok(InputSequence::Bytes(bytes.as_bytes().to_vec()));
    }
    if let Ok(list) = value.cast::<PyList>() {
        return input_from_items(list.iter().collect());
    }
    if let Ok(tuple) = value.cast::<PyTuple>() {
        return input_from_items(tuple.iter().collect());
    }

    Err(PyTypeError::new_err(
        "Rust adapter supports only str, bytes, list/tuple[int], and list/tuple[bool] inputs",
    ))
}

fn input_from_items<'py>(items: Vec<Bound<'py, PyAny>>) -> PyResult<InputSequence> {
    if items.is_empty() {
        return Ok(InputSequence::Elements(Vec::new()));
    }

    if items.iter().all(|item| item.is_instance_of::<PyBool>()) {
        return items
            .into_iter()
            .map(|item| item.extract::<bool>())
            .collect::<PyResult<Vec<_>>>()
            .map(InputSequence::Booleans);
    }

    if items.iter().all(|item| item.extract::<i64>().is_ok()) {
        return items
            .into_iter()
            .map(|item| item.extract::<i64>())
            .collect::<PyResult<Vec<_>>>()
            .map(InputSequence::Integers);
    }

    Err(PyTypeError::new_err(
        "Rust adapter requires homogeneous integer or boolean sequences",
    ))
}

fn sequence_to_py(
    py: Python<'_>,
    sequence: &Sequence,
    inputs: &[InputSequence],
    scalar_sequence: bool,
) -> PyResult<Py<PyAny>> {
    match inputs.first() {
        Some(InputSequence::Text(_))
            if scalar_sequence
                && sequence
                    .iter()
                    .all(|element| matches!(element, Element::Char(_) | Element::Text(_))) =>
        {
            let mut value = String::new();
            for element in sequence {
                match element {
                    Element::Char(character) => value.push(*character),
                    Element::Text(text) => value.push_str(text),
                    _ => unreachable!(),
                }
            }
            Ok(PyString::new(py, &value).into_any().unbind())
        }
        Some(InputSequence::Bytes(_))
            if scalar_sequence
                && sequence
                    .iter()
                    .all(|element| matches!(element, Element::Byte(_))) =>
        {
            let bytes: Vec<u8> = sequence
                .iter()
                .map(|element| match element {
                    Element::Byte(value) => *value,
                    _ => unreachable!(),
                })
                .collect();
            Ok(PyBytes::new(py, &bytes).into_any().unbind())
        }
        _ => {
            let list = PyList::empty(py);
            for element in sequence {
                list.append(element_to_py(py, element)?)?;
            }
            Ok(list.into_any().unbind())
        }
    }
}

fn element_to_py(py: Python<'_>, element: &Element) -> PyResult<Py<PyAny>> {
    Ok(match element {
        Element::Char(value) => PyString::new(py, &value.to_string()).into_any().unbind(),
        Element::Byte(value) => PyInt::new(py, *value).into_any().unbind(),
        Element::Integer(value) => PyInt::new(py, *value).into_any().unbind(),
        Element::Boolean(value) => PyBool::new(py, *value).to_owned().into_any().unbind(),
        Element::Text(value) => PyString::new(py, value).into_any().unbind(),
        Element::Gram(values) => {
            let list = PyList::empty(py);
            for value in values {
                list.append(element_to_py(py, value)?)?;
            }
            list.into_any().unbind()
        }
    })
}

fn require_string_inputs(inputs: &[InputSequence], algorithm: &str) -> PyResult<()> {
    if inputs
        .iter()
        .all(|input| matches!(input, InputSequence::Text(_)))
    {
        Ok(())
    } else {
        Err(PyTypeError::new_err(format!(
            "{algorithm} requires str inputs in the Rust adapter"
        )))
    }
}

fn ensure_binary_inputs(inputs: &[InputSequence], algorithm: &str) -> PyResult<()> {
    let all_text = inputs
        .iter()
        .all(|input| matches!(input, InputSequence::Text(_)));
    let all_bytes = inputs
        .iter()
        .all(|input| matches!(input, InputSequence::Bytes(_)));
    if all_text || all_bytes {
        Ok(())
    } else {
        Err(PyTypeError::new_err(format!(
            "{algorithm} supports only str and bytes inputs in the Rust adapter"
        )))
    }
}

fn algorithm_error(error: AlgorithmError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn input_error(error: crate::core::InputError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn reject_python_callbacks(options: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    for name in ["test_func", "sim_test"] {
        if has_option(options, name)? {
            return Err(PyTypeError::new_err(format!(
                "{name} callbacks cannot cross the Rust adapter boundary"
            )));
        }
    }
    Ok(())
}

fn has_option(options: Option<&Bound<'_, PyDict>>, name: &str) -> PyResult<bool> {
    match options {
        Some(options) => Ok(options.contains(name)?),
        None => Ok(false),
    }
}

fn option_bool(options: Option<&Bound<'_, PyDict>>, name: &str, default: bool) -> PyResult<bool> {
    match option_value(options, name)? {
        Some(value) => value.extract::<bool>(),
        None => Ok(default),
    }
}

fn option_usize(
    options: Option<&Bound<'_, PyDict>>,
    name: &str,
    default: usize,
) -> PyResult<usize> {
    match option_value(options, name)? {
        Some(value) => value.extract::<usize>(),
        None => Ok(default),
    }
}

fn option_i64(options: Option<&Bound<'_, PyDict>>, name: &str, default: i64) -> PyResult<i64> {
    match option_value(options, name)? {
        Some(value) => value.extract::<i64>(),
        None => Ok(default),
    }
}

fn option_f64(options: Option<&Bound<'_, PyDict>>, name: &str, default: f64) -> PyResult<f64> {
    match option_value(options, name)? {
        Some(value) => value.extract::<f64>(),
        None => Ok(default),
    }
}

fn option_optional_f64(options: Option<&Bound<'_, PyDict>>, name: &str) -> PyResult<Option<f64>> {
    match option_value(options, name)? {
        Some(value) if value.is_none() => Ok(None),
        Some(value) => Ok(Some(value.extract::<f64>()?)),
        None => Ok(None),
    }
}

fn option_vec_f64(options: Option<&Bound<'_, PyDict>>, name: &str) -> PyResult<Option<Vec<f64>>> {
    match option_value(options, name)? {
        Some(value) if value.is_none() => Ok(None),
        Some(value) => Ok(Some(value.extract::<Vec<f64>>()?)),
        None => Ok(None),
    }
}

fn option_matrix(
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Option<BTreeMap<Vec<PreparedSequence>, f64>>> {
    let Some(value) = option_value(options, "mat")? else {
        return Ok(None);
    };
    if value.is_none() {
        return Ok(None);
    }

    let matrix = value.cast::<PyDict>().map_err(|_| {
        PyTypeError::new_err("mat must be a dictionary keyed by input-sequence tuples")
    })?;
    let mut parsed = BTreeMap::new();
    for (key, score) in matrix.iter() {
        let key = key.cast::<PyTuple>().map_err(|_| {
            PyTypeError::new_err("mat keys must be tuples of supported input sequences")
        })?;
        let inputs = key
            .iter()
            .map(|input| input_from_object(&input))
            .collect::<PyResult<Vec<_>>>()?;
        let prepared = prepare_sequences(&inputs, QValue::Elements).map_err(input_error)?;
        parsed.insert(prepared, score.extract::<f64>()?);
    }
    Ok(Some(parsed))
}

fn option_char(options: Option<&Bound<'_, PyDict>>, name: &str) -> PyResult<Option<char>> {
    let Some(value) = option_value(options, name)? else {
        return Ok(None);
    };
    if value.is_none() {
        return Ok(None);
    }
    let text = value.extract::<String>()?;
    let mut chars = text.chars();
    let Some(character) = chars.next() else {
        return Err(PyValueError::new_err(format!("{name} must not be empty")));
    };
    if chars.next().is_some() {
        return Err(PyValueError::new_err(format!(
            "{name} must contain exactly one Unicode character"
        )));
    }
    Ok(Some(character))
}

fn option_string(
    options: Option<&Bound<'_, PyDict>>,
    name: &str,
    default: String,
) -> PyResult<String> {
    match option_value(options, name)? {
        Some(value) => value.extract::<String>(),
        None => Ok(default),
    }
}

fn option_value<'py>(
    options: Option<&Bound<'py, PyDict>>,
    name: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    match options {
        Some(options) => options.get_item(name),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{AlgorithmKind, RustAlgorithm};
    use pyo3::prelude::*;
    use pyo3::types::{PyBytes, PyDict, PyList, PyTuple};

    #[test]
    fn algorithm_aliases_are_stable() {
        assert_eq!(
            AlgorithmKind::parse("Damerau-Levenshtein").unwrap(),
            AlgorithmKind::DamerauLevenshtein
        );
        assert_eq!(
            AlgorithmKind::parse("dice").unwrap(),
            AlgorithmKind::Sorensen
        );
        assert_eq!(
            AlgorithmKind::parse("lcs_str").unwrap(),
            AlgorithmKind::LCSStr
        );
    }

    #[test]
    fn python_adapter_executes_common_contract() -> PyResult<()> {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let levenshtein = RustAlgorithm::from_options("levenshtein", Some(1), true, None)?;
            let text_args = PyTuple::new(py, ["test", "text"])?;

            let raw = levenshtein.__call__(py, &text_args)?;
            assert_eq!(raw.bind(py).extract::<f64>()?, 1.0);
            assert_eq!(levenshtein.distance(&text_args)?, 1.0);
            assert_eq!(levenshtein.similarity(&text_args)?, 3.0);
            assert_eq!(levenshtein.maximum(&text_args)?, 4.0);
            assert!((levenshtein.normalized_distance(&text_args)? - 0.25).abs() < f64::EPSILON);
            assert!((levenshtein.normalized_similarity(&text_args)? - 0.75).abs() < f64::EPSILON);

            let prefix = RustAlgorithm::from_options("prefix", Some(1), true, None)?;
            let prefix_output = prefix.__call__(py, &text_args)?;
            assert_eq!(prefix_output.bind(py).extract::<String>()?, "te");

            let word_prefix = RustAlgorithm::from_options("prefix", None, true, None)?;
            let word_args = PyTuple::new(py, ["alpha beta", "alpha gamma"])?;
            let word_output = word_prefix.__call__(py, &word_args)?;
            assert_eq!(
                word_output.bind(py).extract::<Vec<String>>()?,
                vec!["alpha"]
            );

            let hamming = RustAlgorithm::from_options("hamming", Some(1), true, None)?;
            let byte_args = PyTuple::new(
                py,
                [
                    PyBytes::new(py, b"abc").into_any(),
                    PyBytes::new(py, b"abd").into_any(),
                ],
            )?;
            assert_eq!(hamming.distance(&byte_args)?, 1.0);

            let integer_args = PyTuple::new(
                py,
                [
                    PyList::new(py, [1_i64, 2, 3])?.into_any(),
                    PyList::new(py, [1_i64, 4, 3])?.into_any(),
                ],
            )?;
            assert_eq!(hamming.distance(&integer_args)?, 1.0);

            let matrix_options = PyDict::new(py);
            let matrix_values = PyDict::new(py);
            let ac = PyTuple::new(py, ["A", "C"])?;
            matrix_values.set_item(&ac, -3)?;
            matrix_options.set_item("mat", &matrix_values)?;
            matrix_options.set_item("symmetric", true)?;
            let matrix =
                RustAlgorithm::from_options("matrix", Some(1), true, Some(&matrix_options))?;
            let matrix_args = PyTuple::new(py, ["A", "C"])?;
            let matrix_value = matrix.__call__(py, &matrix_args)?;
            assert_eq!(matrix_value.bind(py).extract::<f64>()?, -3.0);

            let callback_options = PyDict::new(py);
            callback_options.set_item("sim_test", py.None())?;
            assert!(
                RustAlgorithm::from_options("prefix", Some(1), true, Some(&callback_options))
                    .is_err()
            );

            Ok(())
        })
    }
}
