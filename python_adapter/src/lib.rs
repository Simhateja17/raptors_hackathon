//! SIM-10: thin Python/PyO3 compatibility adapter.
//!
//! This crate is the *only* FFI boundary between the Python package and the
//! Rust core (`textdistance-port`). It performs input conversion, algorithm
//! construction from a Python-supplied config, and output conversion. All
//! algorithm logic itself lives in `textdistance-port`; this file must not
//! reimplement any of it.
//!
//! Unsupported arbitrary Python objects (custom comparator callables, custom
//! substitution matrices, etc.) are rejected by the Python-side wrapper
//! before reaching this boundary; anything that does reach here and cannot
//! be represented in the shared `Element`/`InputSequence` model fails with a
//! clear `ValueError`/`TypeError` rather than being silently coerced.

use std::panic::{catch_unwind, AssertUnwindSafe};

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyList, PyString, PyTuple};

use textdistance_port::algorithms::{
    arith_ncd::ArithNCD, bag::Bag, bwtrle_ncd::BWTRLENCD, bz2_ncd::Bz2Ncd,
    cosine::Cosine, damerau_levenshtein::DamerauLevenshtein, editex::Editex,
    entropy_ncd::EntropyNcd, gotoh::Gotoh, hamming::Hamming, identity::Identity,
    jaccard::Jaccard, jaro::Jaro, jaro_winkler::JaroWinkler, lcsseq::LCSSeq, lcsstr::LCSStr,
    length::Length, levenshtein::Levenshtein, lzma_ncd::LzmaNcd, matrix::Matrix, mlipns::MLIPNS,
    monge_elkan::MongeElkan, mra::MRA, needleman_wunsch::NeedlemanWunsch, overlap::Overlap,
    postfix::Postfix, prefix::Prefix, ratcliff_obershelp::RatcliffObershelp, rle_ncd::RleNcd,
    smith_waterman::SmithWaterman, sorensen::Sorensen, sqrt_ncd::SqrtNcd, strcmp95::StrCmp95,
    tanimoto::Tanimoto, tversky::Tversky, zlib_ncd::ZlibNcd,
};
use textdistance_port::{
    normalize_distance, normalize_similarity, output_distance, output_similarity, Algorithm,
    AlgorithmOutput, Element, InputSequence, OutputAlgorithm, PreparedSequence, QValue,
    ScoreMode, Sequence,
};

// ---------------------------------------------------------------------
// Python <-> Rust sequence conversion
// ---------------------------------------------------------------------

/// The Python container shape an input arrived in, so the output of a
/// sequence-producing algorithm (LCSSeq/LCSStr/Prefix/Postfix) can be
/// reconstructed as the same kind of Python value the source library
/// would have returned (`str`, `bytes`, or `list`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Str,
    Bytes,
    List,
    Tuple,
}

struct Converted {
    input: InputSequence,
    shape: Shape,
}

fn convert_sequence(obj: &Bound<'_, PyAny>) -> PyResult<Converted> {
    if let Ok(text) = obj.downcast::<PyString>() {
        return Ok(Converted {
            input: InputSequence::Text(text.to_string()),
            shape: Shape::Str,
        });
    }
    if let Ok(bytes) = obj.downcast::<PyBytes>() {
        return Ok(Converted {
            input: InputSequence::Bytes(bytes.as_bytes().to_vec()),
            shape: Shape::Bytes,
        });
    }
    if let Ok(bytes) = obj.extract::<Vec<u8>>() {
        // bytearray and similar buffer-like objects.
        return Ok(Converted {
            input: InputSequence::Bytes(bytes),
            shape: Shape::Bytes,
        });
    }

    let shape = if obj.downcast::<PyTuple>().is_ok() {
        Shape::Tuple
    } else if obj.downcast::<PyList>().is_ok() {
        Shape::List
    } else {
        // Generic iterables (e.g. range, custom Sequence) are treated as
        // list-shaped for reconstruction purposes.
        Shape::List
    };

    let items: Vec<Bound<'_, PyAny>> = obj.iter()?.collect::<PyResult<_>>()?;
    if items.is_empty() {
        return Ok(Converted {
            input: InputSequence::Elements(Vec::new()),
            shape,
        });
    }

    // Homogeneous element-type detection. `bool` is checked before `int`
    // because Python `bool` is an `int` subclass.
    if items.iter().all(|item| item.is_instance_of::<PyBool>()) {
        let values: PyResult<Vec<bool>> = items.iter().map(|item| item.extract::<bool>()).collect();
        return Ok(Converted {
            input: InputSequence::Booleans(values?),
            shape,
        });
    }
    if items.iter().all(|item| item.extract::<i64>().is_ok()) {
        let values: PyResult<Vec<i64>> = items.iter().map(|item| item.extract::<i64>()).collect();
        return Ok(Converted {
            input: InputSequence::Integers(values?),
            shape,
        });
    }
    if items.iter().all(|item| item.downcast::<PyString>().is_ok()) {
        let elements: Vec<Element> = items
            .iter()
            .map(|item| Element::Text(item.extract::<String>().unwrap()))
            .collect();
        return Ok(Converted {
            input: InputSequence::Elements(elements),
            shape,
        });
    }

    Err(PyTypeError::new_err(
        "unsupported sequence element type: the Rust port only supports str, bytes, \
         and homogeneous sequences of int, bool, or str",
    ))
}

fn element_to_pyobject(py: Python<'_>, element: &Element) -> PyResult<PyObject> {
    Ok(match element {
        Element::Char(c) => PyString::new_bound(py, &c.to_string()).into_any().unbind(),
        Element::Text(value) => PyString::new_bound(py, value).into_any().unbind(),
        Element::Integer(value) => value.into_py(py),
        Element::Boolean(value) => value.into_py(py),
        Element::Byte(value) => value.into_py(py),
        Element::Gram(inner) => {
            let items: PyResult<Vec<PyObject>> =
                inner.iter().map(|e| element_to_pyobject(py, e)).collect();
            PyList::new_bound(py, items?).into_any().unbind()
        }
    })
}

fn sequence_to_pyobject(py: Python<'_>, sequence: &Sequence, shape: Shape) -> PyResult<PyObject> {
    match shape {
        Shape::Str => {
            let mut text = String::new();
            for element in sequence {
                match element {
                    Element::Char(c) => text.push(*c),
                    Element::Text(value) => text.push_str(value),
                    other => {
                        return Err(PyValueError::new_err(format!(
                            "cannot reconstruct a str result from element {other:?}"
                        )))
                    }
                }
            }
            Ok(PyString::new_bound(py, &text).into_any().unbind())
        }
        Shape::Bytes => {
            let mut bytes = Vec::with_capacity(sequence.len());
            for element in sequence {
                match element {
                    Element::Byte(value) => bytes.push(*value),
                    other => {
                        return Err(PyValueError::new_err(format!(
                            "cannot reconstruct a bytes result from element {other:?}"
                        )))
                    }
                }
            }
            Ok(PyBytes::new_bound(py, &bytes).into_any().unbind())
        }
        Shape::List => {
            let items: PyResult<Vec<PyObject>> = sequence
                .iter()
                .map(|element| element_to_pyobject(py, element))
                .collect();
            Ok(PyList::new_bound(py, items?).into_any().unbind())
        }
        Shape::Tuple => {
            let items: PyResult<Vec<PyObject>> = sequence
                .iter()
                .map(|element| element_to_pyobject(py, element))
                .collect();
            Ok(PyTuple::new_bound(py, items?).into_any().unbind())
        }
    }
}

// ---------------------------------------------------------------------
// Config extraction helpers
// ---------------------------------------------------------------------

fn dict_get<'py>(config: &Bound<'py, PyDict>, key: &str) -> Option<Bound<'py, PyAny>> {
    config.get_item(key).ok().flatten()
}

fn get_bool(config: &Bound<'_, PyDict>, key: &str, default: bool) -> PyResult<bool> {
    match dict_get(config, key) {
        Some(value) if !value.is_none() => value.extract(),
        _ => Ok(default),
    }
}

fn get_f64(config: &Bound<'_, PyDict>, key: &str, default: f64) -> PyResult<f64> {
    match dict_get(config, key) {
        Some(value) if !value.is_none() => value.extract(),
        _ => Ok(default),
    }
}

fn get_i64(config: &Bound<'_, PyDict>, key: &str, default: i64) -> PyResult<i64> {
    match dict_get(config, key) {
        Some(value) if !value.is_none() => value.extract(),
        _ => Ok(default),
    }
}

fn get_opt_f64_vec(config: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<Vec<f64>>> {
    match dict_get(config, key) {
        Some(value) if !value.is_none() => {
            let items: Vec<Bound<'_, PyAny>> = value.iter()?.collect::<PyResult<_>>()?;
            let values: PyResult<Vec<f64>> = items.iter().map(|item| item.extract()).collect();
            Ok(Some(values?))
        }
        _ => Ok(None),
    }
}

/// `qval` follows the Python convention: missing key -> `default`, explicit
/// `None` -> word-splitting, non-negative int -> element/n-gram mode.
fn get_qval(config: &Bound<'_, PyDict>, default: Option<usize>) -> PyResult<Option<usize>> {
    match dict_get(config, "qval") {
        None => Ok(default),
        Some(value) if value.is_none() => Ok(None),
        Some(value) => Ok(Some(value.extract::<i64>()?.max(0) as usize)),
    }
}

// ---------------------------------------------------------------------
// Panic containment
// ---------------------------------------------------------------------

/// A handful of algorithms (Tversky, Bag, RLE NCD) report invalid-option
/// errors by panicking inside the shared `Algorithm` trait's infallible
/// methods (their fallible `try_*` counterparts are reserved for native
/// Rust callers). Turning a caught panic into a `ValueError` keeps the
/// Python boundary from ever segfaulting or aborting on bad input.
fn run_guarded<T>(f: impl FnOnce() -> T) -> PyResult<T> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "the Rust algorithm reported an invalid input or option".to_owned());
        PyValueError::new_err(message)
    })
}

// ---------------------------------------------------------------------
// Algorithm construction
// ---------------------------------------------------------------------

/// Algorithms whose Python source never runs `_get_sequences` at all and
/// therefore always compare whole, un-split elements regardless of any
/// `qval` attribute the instance happens to carry.
fn always_elements(name: &str) -> bool {
    matches!(name, "strcmp95" | "editex" | "mra" | "matrix" | "length" | "identity")
}

fn build_scalar_algorithm(name: &str, config: &Bound<'_, PyDict>) -> PyResult<Box<dyn Algorithm>> {
    Ok(match name {
        "hamming" => {
            let qval = get_qval(config, Some(1))?;
            let truncate = get_bool(config, "truncate", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Hamming::from_python(qval, truncate, external))
        }
        "levenshtein" => Box::new(Levenshtein::new()),
        "damerau_levenshtein" => {
            let restricted = get_bool(config, "restricted", true)?;
            Box::new(DamerauLevenshtein::with_restricted(restricted))
        }
        "jaro" => {
            let long_tolerance = get_bool(config, "long_tolerance", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Jaro {
                long_tolerance,
                external,
            })
        }
        "jaro_winkler" => {
            let long_tolerance = get_bool(config, "long_tolerance", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(JaroWinkler {
                long_tolerance,
                prefix_weight: 0.1,
                external,
            })
        }
        "strcmp95" => {
            let long_strings = get_bool(config, "long_strings", false)?;
            Box::new(StrCmp95::with_long_strings(long_strings))
        }
        "needleman_wunsch" => {
            let gap_cost = get_f64(config, "gap_cost", 1.0)?;
            Box::new(NeedlemanWunsch::with_gap_cost(gap_cost))
        }
        "gotoh" => {
            let gap_open = get_f64(config, "gap_open", 1.0)?;
            let gap_ext = get_f64(config, "gap_ext", 0.4)?;
            Box::new(Gotoh::with_gap_costs(gap_open, gap_ext))
        }
        "smith_waterman" => {
            let gap_cost = get_f64(config, "gap_cost", 1.0)?;
            Box::new(SmithWaterman::with_gap_cost(gap_cost))
        }
        "mlipns" => {
            let threshold = get_f64(config, "threshold", 0.25)?;
            let maxmismatches = get_i64(config, "maxmismatches", 2)?.max(0) as usize;
            Box::new(MLIPNS::with_params(threshold, maxmismatches))
        }
        "jaccard" => {
            let qval = get_qval(config, Some(1))?;
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Jaccard::from_python(qval, as_set, external))
        }
        "sorensen" => {
            let qval = get_qval(config, Some(1))?;
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Sorensen::from_python(qval, as_set, external))
        }
        "tversky" => {
            let qval = get_qval(config, Some(1))?;
            let ks = get_opt_f64_vec(config, "ks")?;
            let bias = match dict_get(config, "bias") {
                Some(value) if !value.is_none() => Some(value.extract::<f64>()?),
                _ => None,
            };
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Tversky::from_python(qval, ks, bias, as_set, external))
        }
        "overlap" => {
            let qval = get_qval(config, Some(1))?;
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Overlap::from_python(qval, as_set, external))
        }
        "cosine" => {
            let qval = get_qval(config, Some(1))?;
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Cosine::from_python(qval, as_set, external))
        }
        "tanimoto" => {
            let qval = get_qval(config, Some(1))?;
            let as_set = get_bool(config, "as_set", false)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Tanimoto::from_python(qval, as_set, external))
        }
        "monge_elkan" => {
            let qval = get_qval(config, Some(1))?;
            let symmetric = get_bool(config, "symmetric", false)?;
            let external = get_bool(config, "external", true)?;
            // Only the default inner algorithm (unrestricted-default
            // Damerau-Levenshtein) is supported; a genuinely custom
            // `algorithm=` object is rejected by the Python wrapper before
            // reaching this boundary.
            Box::new(MongeElkan::from_python(
                DamerauLevenshtein::default(),
                symmetric,
                qval,
                external,
            ))
        }
        "bag" => {
            let qval = get_qval(config, Some(1))?;
            let external = get_bool(config, "external", true)?;
            Box::new(Bag::from_python(qval, false, external))
        }
        "ratcliff_obershelp" => Box::new(RatcliffObershelp::new()),
        "arith_ncd" => {
            let base = get_i64(config, "base", 2)?.max(2) as u32;
            let terminator = match dict_get(config, "terminator") {
                Some(value) if !value.is_none() => {
                    let text = value.extract::<String>()?;
                    text.chars().next()
                }
                _ => None,
            };
            Box::new(ArithNCD::with_config(base, terminator))
        }
        "rle_ncd" => {
            let qval = get_qval(config, Some(1))?;
            Box::new(RleNcd::from_python(qval))
        }
        "bwtrle_ncd" => {
            let terminator = match dict_get(config, "terminator") {
                Some(value) if !value.is_none() => {
                    let text = value.extract::<String>()?;
                    text.chars().next().unwrap_or('\0')
                }
                _ => '\0',
            };
            Box::new(BWTRLENCD::with_terminator(Element::Char(terminator)))
        }
        "sqrt_ncd" => Box::new(SqrtNcd),
        "entropy_ncd" => {
            let coef = get_f64(config, "coef", 1.0)?;
            let base = get_f64(config, "base", 2.0)?;
            Box::new(EntropyNcd { coef, base })
        }
        "bz2_ncd" => Box::new(Bz2Ncd),
        "lzma_ncd" => Box::new(LzmaNcd),
        "zlib_ncd" => Box::new(ZlibNcd),
        "mra" => Box::new(MRA::new()),
        "editex" => {
            let local = get_bool(config, "local", false)?;
            let match_cost = get_i64(config, "match_cost", 0)?;
            let group_cost = get_i64(config, "group_cost", 1)?;
            let mismatch_cost = get_i64(config, "mismatch_cost", 2)?;
            let external = get_bool(config, "external", true)?;
            Box::new(Editex::new(
                local,
                match_cost,
                group_cost,
                mismatch_cost,
                external,
            ))
        }
        "length" => Box::new(Length),
        "identity" => Box::new(Identity),
        "matrix" => {
            let mismatch_cost = get_f64(config, "mismatch_cost", 0.0)?;
            let match_cost = get_f64(config, "match_cost", 1.0)?;
            let symmetric = get_bool(config, "symmetric", true)?;
            let external = get_bool(config, "external", true)?;
            // A custom `mat=` lookup table is rejected by the Python wrapper
            // before reaching this boundary; only the identity-fallback
            // configuration is supported here.
            Box::new(Matrix::new(
                None,
                mismatch_cost,
                match_cost,
                symmetric,
                external,
            ))
        }
        other => {
            return Err(PyValueError::new_err(format!(
                "unknown or unsupported algorithm: {other}"
            )))
        }
    })
}

fn scalar_qvalue(name: &str, config: &Bound<'_, PyDict>) -> PyResult<QValue> {
    if always_elements(name) {
        return Ok(QValue::Elements);
    }
    let qval = get_qval(config, Some(1))?;
    Ok(QValue::from_python(qval))
}

// ---------------------------------------------------------------------
// Sequence-output algorithms: LCSSeq, LCSStr, Prefix, Postfix
// ---------------------------------------------------------------------

enum SequenceAlgorithm {
    LCSSeq(LCSSeq),
    LCSStr(LCSStr),
    Prefix(Prefix),
    Postfix(Postfix),
}

fn build_sequence_algorithm(
    name: &str,
    config: &Bound<'_, PyDict>,
) -> PyResult<Option<(SequenceAlgorithm, QValue)>> {
    Ok(match name {
        "lcsseq" => {
            let qval = get_qval(config, Some(1))?;
            Some((
                SequenceAlgorithm::LCSSeq(LCSSeq::new()),
                QValue::from_python(qval),
            ))
        }
        "lcsstr" => {
            let qval = get_qval(config, Some(1))?;
            let external = get_bool(config, "external", true)?;
            Some((
                SequenceAlgorithm::LCSStr(LCSStr::from_python(qval, external)),
                QValue::from_python(qval),
            ))
        }
        "prefix" => {
            let qval = get_qval(config, Some(1))?;
            Some((
                SequenceAlgorithm::Prefix(Prefix::new()),
                QValue::from_python(qval),
            ))
        }
        "postfix" => {
            let qval = get_qval(config, Some(1))?;
            Some((
                SequenceAlgorithm::Postfix(Postfix::new()),
                QValue::from_python(qval),
            ))
        }
        _ => None,
    })
}

impl SequenceAlgorithm {
    fn as_output_algorithm(&self) -> &dyn OutputAlgorithm {
        match self {
            Self::LCSSeq(alg) => alg,
            Self::LCSStr(alg) => alg,
            Self::Prefix(alg) => alg,
            Self::Postfix(alg) => alg,
        }
    }
}

// ---------------------------------------------------------------------
// Postfix needs its inputs reversed before the shared `Prefix::call`-style
// logic runs, and reversed back afterwards; `Postfix::call` in the core
// crate already does this internally given `PreparedSequence`s, so no
// special casing is needed here beyond routing to it.
// ---------------------------------------------------------------------

fn apply_scalar_method(
    algorithm: &dyn Algorithm,
    method: &str,
    prepared: &[PreparedSequence],
) -> PyResult<f64> {
    run_guarded(|| match method {
        "call" => algorithm.call(prepared),
        "distance" => algorithm.distance(prepared),
        "similarity" => algorithm.similarity(prepared),
        "normalized_distance" => algorithm.normalized_distance(prepared),
        "normalized_similarity" => algorithm.normalized_similarity(prepared),
        "maximum" => algorithm.maximum(prepared),
        _ => f64::NAN,
    })
}

fn respond_with_output(
    py: Python<'_>,
    output: AlgorithmOutput,
    maximum: f64,
    mode: ScoreMode,
    method: &str,
    shape: Shape,
) -> PyResult<PyObject> {
    let distance = output_distance(&output, mode, maximum);
    let similarity = output_similarity(&output, mode, maximum);

    match method {
        "call" => {
            let empty = Sequence::new();
            let sequence = output.sequence().unwrap_or(&empty);
            sequence_to_pyobject(py, sequence, shape)
        }
        "distance" => Ok(distance.into_py(py)),
        "similarity" => Ok(similarity.into_py(py)),
        "normalized_distance" => Ok(normalize_distance(distance, maximum).into_py(py)),
        "normalized_similarity" => Ok(normalize_similarity(distance, maximum).into_py(py)),
        "maximum" => Ok(maximum.into_py(py)),
        other => Err(PyValueError::new_err(format!("unknown method: {other}"))),
    }
}

// ---------------------------------------------------------------------
// Top-level entry point
// ---------------------------------------------------------------------

/// Compute one common method (`call`, `distance`, `similarity`,
/// `normalized_distance`, `normalized_similarity`, or `maximum`) for a
/// named public algorithm, using only the Rust core for the computation.
///
/// `config` is expected to be the calling Python instance's `__dict__`
/// (or a subset of it); unrecognized keys are ignored. `sequences` are the
/// raw Python arguments the source library's method would have received.
#[pyfunction]
fn compute(
    py: Python<'_>,
    name: &str,
    config: &Bound<'_, PyDict>,
    method: &str,
    sequences: Vec<Bound<'_, PyAny>>,
) -> PyResult<PyObject> {
    let converted: PyResult<Vec<Converted>> =
        sequences.iter().map(convert_sequence).collect();
    let converted = converted?;
    let shape = converted.first().map(|c| c.shape).unwrap_or(Shape::List);
    let inputs: Vec<InputSequence> = converted.into_iter().map(|c| c.input).collect();

    if let Some((sequence_algorithm, qvalue)) = build_sequence_algorithm(name, config)? {
        let prepared = textdistance_port::prepare_sequences(&inputs, qvalue)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;

        // LCSStr has bespoke early-return rules that run on the *raw*
        // (unprepared) inputs: empty before q-value preparation, and a
        // single input passed through unchanged regardless of `qval`. Every
        // common method (call/distance/similarity/normalized_*) ultimately
        // derives from that same early-return-aware result in the source
        // library, so route all of them through it, not just `call`.
        if let SequenceAlgorithm::LCSStr(lcsstr) = &sequence_algorithm {
            let output = run_guarded(|| lcsstr.output_inputs(&inputs))?
                .map_err(|error| PyValueError::new_err(error.to_string()))?;
            let maximum = run_guarded(|| lcsstr.output_maximum(&prepared))?;
            let mode = lcsstr.output_mode();
            return respond_with_output(py, output, maximum, mode, method, shape);
        }

        let output_algorithm = sequence_algorithm.as_output_algorithm();
        let output = run_guarded(|| output_algorithm.output(&prepared))?
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let maximum = run_guarded(|| output_algorithm.output_maximum(&prepared))?;
        let mode = output_algorithm.output_mode();
        return respond_with_output(py, output, maximum, mode, method, shape);
    }

    let algorithm = build_scalar_algorithm(name, config)?;
    let qvalue = scalar_qvalue(name, config)?;
    let prepared = textdistance_port::prepare_sequences(&inputs, qvalue)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    let result = apply_scalar_method(algorithm.as_ref(), method, &prepared)?;
    Ok(result.into_py(py))
}

#[pyfunction]
fn version() -> &'static str {
    textdistance_port::VERSION
}

#[pymodule]
fn textdistance_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
