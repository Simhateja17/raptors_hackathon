//! Compile-visible algorithm registry.
//!
//! Each path is a replaceable implementation packet. Algorithm owners replace the
//! placeholder file without editing this shared registry.

#[path = "edit/levenshtein.rs"]
pub mod levenshtein;

#[path = "edit/damerau_levenshtein.rs"]
pub mod damerau_levenshtein;

#[path = "edit/needleman_wunsch.rs"]
pub mod needleman_wunsch;

#[path = "edit/smith_waterman.rs"]
pub mod smith_waterman;

#[path = "edit/gotoh.rs"]
pub mod gotoh;

#[path = "edit/strcmp95.rs"]
pub mod strcmp95;

#[path = "edit/mlipns.rs"]
pub mod mlipns;

#[path = "edit/jaro.rs"]
pub mod jaro;

#[path = "edit/jaro_winkler.rs"]
pub mod jaro_winkler;

#[path = "edit/hamming.rs"]
pub mod hamming;

#[path = "token/jaccard.rs"]
pub mod jaccard;

#[path = "token/sorensen.rs"]
pub mod sorensen;

#[path = "token/tversky.rs"]
pub mod tversky;

#[path = "token/cosine.rs"]
pub mod cosine;

#[path = "token/monge_elkan.rs"]
pub mod monge_elkan;

#[path = "token/bag.rs"]
pub mod bag;

#[path = "token/overlap.rs"]
pub mod overlap;

#[path = "token/tanimoto.rs"]
pub mod tanimoto;

#[path = "sequence/lcsseq.rs"]
pub mod lcsseq;

#[path = "sequence/lcsstr.rs"]
pub mod lcsstr;

#[path = "sequence/ratcliff_obershelp.rs"]
pub mod ratcliff_obershelp;

#[path = "compression/arith_ncd.rs"]
pub mod arith_ncd;

#[path = "compression/rle_ncd.rs"]
pub mod rle_ncd;

#[path = "compression/bwtrle_ncd.rs"]
pub mod bwtrle_ncd;

#[path = "compression/sqrt_ncd.rs"]
pub mod sqrt_ncd;

#[path = "compression/entropy_ncd.rs"]
pub mod entropy_ncd;

#[path = "compression/bz2_ncd.rs"]
pub mod bz2_ncd;

#[path = "compression/lzma_ncd.rs"]
pub mod lzma_ncd;

#[path = "compression/zlib_ncd.rs"]
pub mod zlib_ncd;

#[path = "phonetic/editex.rs"]
pub mod editex;

#[path = "phonetic/mra.rs"]
pub mod mra;

#[path = "simple/prefix.rs"]
pub mod prefix;

#[path = "simple/postfix.rs"]
pub mod postfix;

#[path = "simple/length.rs"]
pub mod length;

#[path = "simple/identity.rs"]
pub mod identity;

#[path = "simple/matrix.rs"]
pub mod matrix;

/// edit algorithm modules.
pub mod edit {
    pub use super::{
        damerau_levenshtein, gotoh, hamming, jaro, jaro_winkler, levenshtein, mlipns,
        needleman_wunsch, smith_waterman, strcmp95,
    };
}

/// token algorithm modules.
pub mod token {
    pub use super::{bag, cosine, jaccard, monge_elkan, overlap, sorensen, tanimoto, tversky};
}

/// sequence algorithm modules.
pub mod sequence {
    pub use super::{lcsseq, lcsstr, ratcliff_obershelp};
}

/// compression algorithm modules.
pub mod compression {
    pub use super::{
        arith_ncd, bwtrle_ncd, bz2_ncd, entropy_ncd, lzma_ncd, rle_ncd, sqrt_ncd, zlib_ncd,
    };
}

/// phonetic algorithm modules.
pub mod phonetic {
    pub use super::{editex, mra};
}

/// simple algorithm modules.
pub mod simple {
    pub use super::{identity, length, matrix, postfix, prefix};
}

/// Number of algorithm packets registered in the scaffold.
pub const REGISTERED_ALGORITHM_MODULES: usize = 36;
