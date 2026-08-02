# Simha Teja behavior cards

These cards freeze the observable Python behavior before the Rust packets are
implemented. The Rust implementation receives prepared sequences from the
shared contract in [`docs/API_CONTRACT.md`](../API_CONTRACT.md): `qval=None`
means whitespace-separated words, `qval=1` means individual elements, and
`qval>1` means sliding q-grams. Python strings must therefore be represented by
Unicode scalar values, not UTF-8 bytes.

The Python implementation accepts custom `test_func`/`sim_func` callbacks for
some algorithms. Those callbacks cannot cross the Rust boundary. The adapter
must use the built-in equality/comparator path or return the documented
unsupported-comparator error; the native algorithm behavior below assumes the
default equality comparator unless stated otherwise.

## SIM-01 — Levenshtein

- Source: `textdistance/algorithms/edit_based.py`, `Levenshtein`.
- Original tests: `tests/test_edit/test_levenshtein.py`.
- Inputs: exactly two prepared sequences; default `qval=1`, optional custom
  element comparator in Python, and an `external` acceleration flag that does
  not change the Rust result.
- Raw result: minimum unit-cost insertions, deletions, and substitutions. A
  matching pair costs zero; a non-matching pair costs one.
- Empty/equal behavior: `distance([], []) = 0`; `distance([], x) = len(x)`;
  equal non-empty sequences have distance zero.
- Expected values: `test/text = 1`, `test/tset = 2`, `test/qwe = 4`,
  `test/testit = 2`, `test/tet = 1`.
- Risks: compare `Element` values, including `char`, byte, integer, and gram
  values, rather than comparing UTF-8 bytes or stringified values.

## SIM-02 — Damerau-Levenshtein

- Source: `textdistance/algorithms/edit_based.py`,
  `DamerauLevenshtein`.
- Original tests: `tests/test_edit/test_damerau_levenshtein.py`.
- Inputs: exactly two prepared sequences; `restricted=True` by default,
  optional `restricted=False`, `qval`, comparator, and `external` flag.
- Raw result: unit-cost insertion, deletion, substitution, and adjacent
  transposition. Restricted mode permits a transposition only when the two
  swapped elements are adjacent and each element is touched once. Unrestricted
  mode uses the last-seen positions matrix and permits repeated transposition
  participation.
- Empty/equal behavior: same distance quick answers as Levenshtein; an empty
  side costs the length of the other side and equal inputs cost zero.
- Expected values: restricted `test/tset = 1`, `ab/ba = 1`, `ab/bca = 3`,
  `abcd/bdac = 4`; unrestricted `ab/bca = 2` and `abcd/bdac = 3`.
- Risks: preserve the distinction between restricted and unrestricted modes;
  the source's unrestricted examples are the regression guard.

## SIM-03 — Needleman-Wunsch

- Source: `textdistance/algorithms/edit_based.py`, `NeedlemanWunsch`.
- Original tests: `tests/test_edit/test_needleman_wunsch.py`.
- Inputs: two prepared sequences; `gap_cost` defaults to `1.0`; the default
  similarity function is equality (`+1` for equal, `0` for different), while
  tests also pass a built-in `Matrix` or Python callback.
- Raw result: global alignment similarity. Initialize the first row/column to
  `-gap_cost * gap_count`; each cell is the maximum of diagonal similarity,
  deletion, and insertion. The score is the bottom-right cell.
- Common methods: `maximum = max(len(left), len(right))`,
  `minimum = -maximum * gap_cost`, `distance = -similarity`, and the source
  normalized formulas use the `[minimum, maximum]` range.
- Empty/equal behavior: this implementation performs the DP directly (its
  source fast-path call is commented out). Empty/empty is zero; an empty side
  is scored as the required leading/trailing gaps (`-len(other) * gap_cost`),
  and equal inputs receive their diagonal similarity score.
- Expected values: with `sim_ident`, `GATTACA/GCATGCU = 0`; with gap five,
  `CGATATCAG/TGACGSTGC = -5`, `AGACTAGTTAC/TGACGSTGC = -7`, and
  `AGACTAGTTAC/CGAGACGT = -15`. With the documented nucleotide matrix and
  gap five, `AGACTAGTTAC/CGAGACGT = 16`.
- Risks: this is a similarity-native algorithm with negative scores; do not
  force it into a non-negative edit-distance formula.

## SIM-04 — Smith-Waterman

- Source: `textdistance/algorithms/edit_based.py`, `SmithWaterman`.
- Original tests: `tests/test_edit/test_smith_waterman.py`.
- Inputs: two prepared sequences; `gap_cost` defaults to `1.0`; equality is
  the default similarity function and a matrix/custom comparator is supported
  by the Python API.
- Raw result: local alignment similarity. Each cell is the maximum of zero,
  diagonal similarity, deletion, and insertion. The final score is the
  bottom-right cell in this implementation, not the maximum over all cells.
- Common methods: `maximum = min(len(left), len(right))`; similarity is the
  raw non-negative score and distance is `maximum - similarity`.
- Empty/equal behavior: an empty side has similarity zero; equal inputs use the
  maximum fast path; mismatching alignment paths may floor at zero.
- Expected values: with the nucleotide matrix and gap five,
  `AGACTAGTTAC/CGAGACGT = 26`; with equality `GATTACA/GCATGCU = 0`,
  `AGACTAGTTAC/TGACGSTGC = 1`, and
  `AGACTAGTTAC/CGAGACGT = 0`.
- Risks: preserve the zero reset and the `min`-length maximum; these differ
  materially from Needleman-Wunsch.

## SIM-05 — Gotoh

- Source: `textdistance/algorithms/edit_based.py`, `Gotoh`.
- Original tests: `tests/test_edit/test_gotoh.py`.
- Inputs: two prepared sequences; `gap_open` defaults to `1`, `gap_ext`
  defaults to `0.4`, plus `qval` and a similarity function.
- Raw result: global alignment with affine gaps. Keep three DP states: a
  diagonal/match state, a gap in the left sequence, and a gap in the right
  sequence. Opening a gap costs `gap_open`; extending it costs `gap_ext`.
  Return the maximum terminal state.
- Common methods: source `maximum = min(len(left), len(right))` and
  `minimum = -min(len(left), len(right))`; the tested raw score is a
  similarity and may be fractional or negative.
- Empty/equal behavior: this implementation also performs the affine DP
  directly rather than using the BaseSimilarity fast path. Empty/empty is
  zero; an empty side receives the initialized affine gap score, and equal
  inputs follow the diagonal similarity states.
- Expected values with `sim_ident`: `GATTACA/GCATGCU = 0` for open/ext `1/1`;
  with open/ext `1/.5`, `AGACTAGTTAC/TGACGSTGC = 1.5` and
  `AGACTAGTTAC/CGAGACGT = 1`; with open/ext `5/5`, the latter is `-15`.
- Risks: do not collapse open and extension penalties into one linear gap;
  terminal state selection and first-row/column initialization are part of the
  compatibility behavior.

## SIM-06 — StrCmp95

- Source: `textdistance/algorithms/edit_based.py`, `StrCmp95`.
- Original tests: `tests/test_edit/test_strcmp95.py`.
- Inputs: string-like sequences only; `long_strings=False` by default. The
  implementation strips leading/trailing whitespace and uppercases both
  strings before matching. `maximum` is always `1`.
- Raw result: similarity in `[0, 1]` based on positional matches within a
  search range, transposition count, special phonetic/recognition-pair
  weights, a prefix boost of up to four non-digit characters, and an optional
  long-string adjustment.
- Empty/equal behavior: equal normalized strings return `1`; an empty versus
  non-empty input returns `0`; no common characters returns `0`.
- Expected values within the original `isclose` tolerance: `MARTHA/MARHTA =
  0.9611111111111111`, `DWAYNE/DUANE = 0.873`, `DIXON/DICKSONX =
  0.839333333`, and `TEST/TEXT = 0.9066666666666666`.
- Risks: preserve trim/uppercase behavior, the asymmetric prefix/long-string
  guards, and floating-point tolerance. This is not ordinary Jaro-Winkler.

## SIM-07 — MLIPNS

- Source: `textdistance/algorithms/edit_based.py`, `MLIPNS`.
- Original tests: `tests/test_edit/test_mlipns.py`.
- Inputs: one or more prepared sequences; `threshold=0.25`,
  `maxmismatches=2`, and `qval=1` by default. The source first computes a
  Hamming mismatch count, then tests progressively reduced lengths while the
  mismatch budget remains.
- Raw result: binary similarity (`1.0` or `0.0`), with `maximum=1`. Return
  `1` when the mismatch ratio is within `threshold` during the allowed
  mismatch iterations; otherwise return `0`.
- Empty/equal behavior: `''/''`, `a/a`, and equal non-empty inputs return `1`;
  `a/''` and `''/a` return `0`.
- Expected values: `ab/a = 1`, `abc/abcde = 1`, `abcg/abcdefg = 0`,
  `Tomato/Tamato = 1`, and `ato/Tam = 1`.
- Risks: preserve the binary output and the source's use of maximum sequence
  length plus the Hamming result; do not replace it with a conventional
  normalized Hamming similarity.

## SIM-08 — Arithmetic NCD

- Source: `textdistance/algorithms/compression_based.py`, `ArithNCD` and
  `_NCDBase`.
- Original tests: `tests/test_compression/test_arith_ncd.py`.
- Inputs: one or more sequences; `base=2`, `terminator=None`, and `qval=1` by
  default. The compressor builds exact `Fraction` probability intervals from
  combined character counts, optionally adds one terminator symbol, and uses
  the bit length approximation `ceil(log(numerator, base))` as compressed
  size.
- Raw result: for every permutation of multiple inputs, compress the joined
  data and choose the shortest concatenation. Return
  `(concat_size - min(individual_sizes) * (n - 1)) / max(individual_sizes)`;
  no input returns zero and all-zero individual sizes return zero.
- Empty/equal behavior: equal inputs are not forced to a generic distance
  quick answer; compression-size semantics determine the result. The tested
  identical case `test/test` returns `1`.
- Expected values: `arith_ncd('test', 'nani') = 2.1666666666666665` within
  `isclose`; with `terminator='\\x00'`, `_make_probs('lol', 'lal')` gives
  `l=(0, 4/7)` and `_compress('BANANA')` has numerator `1525`.
- Risks: preserve exact probability ordering and the configured logarithm base;
  compressor output is numeric compatibility data, not a generic normalized
  edit score.

## SIM-09 — LCS sequence

- Source: `textdistance/algorithms/sequence_based.py`, `LCSSeq`.
- Original tests: `tests/test_sequence/test_lcsseq.py`.
- Inputs: one or more sequences; `qval=1` by default; optional equality
  callback in Python. The source returns the subsequence itself from `__call__`
  and uses its length for `similarity`.
- Raw result: for two sequences, dynamic programming followed by backtracking.
  On a tie, backtracking moves upward before moving left. For more than two
  sequences, recursively remove a final element and choose the longest result
  in sequence order. If any sequence is empty, return an empty sequence.
- Empty/equal behavior: no arguments returns an empty sequence; equal inputs
  return that sequence; an empty side returns an empty sequence.
- Expected values: `ab/cd = ''`, `abcd/abcd = 'abcd'`, `test/text = 'tet'`,
  `thisisatest/testing123testing = 'tsitest'`, `DIXON/DICKSONX = 'DION'`,
  and `random exponential/layer activation = 'ratia'`. Three-way examples:
  `a/b/c = ''`, `a/a/a = 'a'`, and `test/text/tempest = 'tet'`.
- Risks: preserve the returned sequence and tie-breaking, not only its length;
  Rust `Element` values must be reconstructed by the adapter without Python
  string coercion.

## Cross-packet review checklist

- Native tests must cover normal, empty, equal, Unicode, integer/byte inputs
  where applicable, and q-gram preparation.
- Algorithms must consume prepared sequences and must not duplicate q-value or
  word-splitting logic.
- Floating-point assertions use the source test tolerance (`isclose` where the
  original test uses it).
- No Rust core code may import Python or call the original implementation.
