//! ZLIB NCD (Normalized Compression Distance).
//!
//! Source: `textdistance/algorithms/compression_based.py::ZLIBNCD`, via
//! `_BinaryNCDBase` and `_NCDBase`. Full behavior card:
//! `docs/behavior-cards/manasa/zlib-ncd.md`. Dependency:
//! `docs/dependency-notes/manasa.md`.

use crate::core::{Algorithm, Element, PreparedSequence, ScoreMode};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

/// Source: `codecs.encode(data, 'zlib_codec')[2:]` strips the RFC 1950
/// zlib stream header before measuring length. `ZlibEncoder` (not
/// `DeflateEncoder`, which omits the header/trailer) produces that same
/// framing. `Compression::default()` matches Python's `zlib` default
/// level (`Z_DEFAULT_COMPRESSION`, level 6).
const HEADER_STRIP: usize = 2;

/// Convert prepared elements to raw bytes, matching `_BinaryNCDBase`: `str`
/// input is UTF-8 encoded, `bytes` input passes through unchanged.
fn to_bytes(elements: &[Element]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(elements.len());
    for element in elements {
        match element {
            Element::Char(c) => {
                let mut buf = [0u8; 4];
                bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
            Element::Byte(b) => bytes.push(*b),
            other => panic!("ZLIB NCD only supports character or byte sequences, got {other:?}"),
        }
    }
    bytes
}

fn get_size(data: &[u8]) -> f64 {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .expect("in-memory write cannot fail");
    let compressed = encoder.finish().expect("in-memory finish cannot fail");
    compressed.len().saturating_sub(HEADER_STRIP) as f64
}

/// All orderings of `items`, matching Python's `itertools.permutations`.
fn permutations_of<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() {
        return vec![vec![]];
    }
    let mut result = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let chosen = rest.remove(i);
        for mut perm in permutations_of(&rest) {
            perm.insert(0, chosen.clone());
            result.push(perm);
        }
    }
    result
}

/// ZLIB NCD. See `docs/behavior-cards/manasa/zlib-ncd.md`.
#[derive(Default)]
pub struct ZlibNcd;

impl Algorithm for ZlibNcd {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64 {
        // Source: `_NCDBase.__call__` — `if not sequences: return 0`. No
        // `quick_answer` shortcut anywhere in the NCD family.
        if sequences.is_empty() {
            return 0.0;
        }

        let byte_seqs: Vec<Vec<u8>> = sequences.iter().map(|s| to_bytes(s)).collect();

        let mut concat_len = f64::INFINITY;
        for permutation in permutations_of(&byte_seqs) {
            let concatenated: Vec<u8> = permutation.into_iter().flatten().collect();
            concat_len = concat_len.min(get_size(&concatenated));
        }

        let compressed_lens: Vec<f64> = byte_seqs.iter().map(|s| get_size(s)).collect();
        let max_len = compressed_lens.iter().cloned().fold(f64::MIN, f64::max);
        if max_len == 0.0 {
            return 0.0;
        }
        let min_len = compressed_lens.iter().cloned().fold(f64::MAX, f64::min);
        let n = sequences.len() as f64;

        (concat_len - min_len * (n - 1.0)) / max_len
    }

    // Source: `_NCDBase.maximum` always returns `1`.
    fn maximum(&self, _sequences: &[PreparedSequence]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Distance
    }
}
