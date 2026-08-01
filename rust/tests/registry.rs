#[allow(unused_imports)]
use textdistance_port::algorithms::{
    arith_ncd, bag, bwtrle_ncd, bz2_ncd, cosine, damerau_levenshtein, editex, entropy_ncd, gotoh,
    hamming, identity, jaro, jaro_winkler, lcsseq, lcsstr, length, levenshtein, lzma_ncd, matrix,
    mlipns, monge_elkan, mra, needleman_wunsch, overlap, postfix, prefix, ratcliff_obershelp,
    rle_ncd, smith_waterman, sorensen, sqrt_ncd, strcmp95, tanimoto, tversky, zlib_ncd,
};

#[test]
fn all_algorithm_paths_are_compile_visible() {
    // The imports above intentionally cover every registered packet. If a teammate
    // removes or misnames a registry entry, this integration test fails to compile.
    assert_eq!(
        textdistance_port::algorithms::REGISTERED_ALGORITHM_MODULES,
        36
    );
}
