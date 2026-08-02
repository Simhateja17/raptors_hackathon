# Poojitha's Lane 2 Behavior Cards

These cards are the behavior checklist for Poojitha's nine Lane 3 packets.
They describe the existing Python implementation in
`textdistance/algorithms/`; they are not redesigned algorithm definitions.
The Rust implementation must preserve the observable behavior recorded here.

Primary source files:

- `textdistance/algorithms/base.py`
- `textdistance/algorithms/edit_based.py`
- `textdistance/algorithms/token_based.py`
- `textdistance/algorithms/sequence_based.py`
- `textdistance/algorithms/compression_based.py`

Primary source tests:

- `tests/original/test_edit/test_hamming.py`
- `tests/original/test_token/test_jaccard.py`
- `tests/original/test_token/test_sorensen.py`
- `tests/original/test_token/test_cosine.py`
- `tests/original/test_token/test_monge_elkan.py`
- `tests/original/test_token/test_bag.py`
- `tests/original/test_sequence/test_lcsstr.py`
- `tests/original/test_compression/test_common.py`

## Shared behavior

The Python base classes establish these rules before an algorithm-specific
calculation runs:

| Python option | Behavior |
| --- | --- |
| `qval=None` or a false-y value | split strings with `str.split()` into words |
| `qval=1` | compare the original sequence elements |
| `qval>1` | compare sliding q-grams |
| `as_set=False` | repeated elements keep their counts |
| `as_set=True` | counts are reduced to the number of distinct remaining elements when a token algorithm counts a counter |
| `external=True` | the Python implementation may try registered third-party libraries; the Rust core must stay independently correct and must not call Python at runtime |

The base distance class returns zero for no/one/equal sequences and returns
the largest input length when one of several sequences is empty. The base
similarity class returns its maximum for no/one/equal sequences and zero when
one of several sequences is empty. The algorithm-specific cards below call
out exceptions, including algorithms that do not use the base quick-answer
path.

For token algorithms, the Python maximum is `1`. Their direct call is the
similarity, and distance is `1 - similarity`.

For Hamming and Bag, the direct call is the distance. Their similarity is
maximum minus distance. For RLE NCD, the direct call is a normalized distance
with maximum `1` in the Python implementation.

The Rust core receives prepared sequences. It must therefore use the shared
preparation contract and must not silently create a second, different q-gram
or word-splitting rule inside an algorithm.

## 1. Hamming

**Source:** `edit_based.py`, class `Hamming`
**Tests:** `tests/original/test_edit/test_hamming.py`
**Rust packet:** `rust/src/algorithms/edit/hamming.rs`

### Meaning

Count positions whose elements do not satisfy the configured comparison
function. Hamming accepts two or more sequences.

- `truncate=False` uses Python `zip_longest`: extra elements in longer
  sequences count as differences.
- `truncate=True` uses Python `zip`: the extra tail is ignored by the main
  calculation.
- The quick-answer check happens before that choice. Therefore an empty
  sequence paired with a non-empty sequence returns the maximum even when
  `truncate=True`.
- The default comparison is the base identity check across all elements at a
  position. A custom `test_func` is part of the Python constructor behavior.

### Expected examples

| Call | Result |
| --- | ---: |
| `Hamming()("test", "text")` | `1` |
| `Hamming()("test", "testit")` | `2` |
| `Hamming(truncate=True)("test", "testit")` | `0` |
| `Hamming()("", "abc")` | `3` |
| `Hamming(truncate=True)("", "abc")` | `3` because quick-answer runs first |
| `Hamming(qval=2)("test", "text")` | `2` |
| `Hamming()("é", "e")` | `1` Unicode scalar comparison |

Equal inputs return zero. Two empty inputs return zero. With `qval=None`, the
strings are compared as one-token word sequences when there is no whitespace;
with q-grams shorter than the requested q-value, the prepared sequence can be
empty and the normal quick-answer rules apply.

### Compatibility risks

- Python's `test_func` can accept the variable-length tuple produced by
  `zip_longest`; the Rust-facing API needs an explicit comparator shape rather
  than silently dropping this option.
- Python's common `maximum` sees the original inputs, while the direct score
  sees prepared inputs. This matters for q-grams and must be preserved by the
  adapter/common-method boundary.

## 2. Jaccard

**Source:** `token_based.py`, class `Jaccard`
**Tests:** `tests/original/test_token/test_jaccard.py`
**Rust packet:** `rust/src/algorithms/token/jaccard.rs`

### Meaning

Build a `Counter` for each prepared sequence. Compute the counter intersection
using the minimum count for each element and the counter union using the
maximum count. Then return:

```text
intersection_count / union_count
```

When `as_set=False`, counts are summed. When `as_set=True`, each counter's
distinct keys are counted after the intersection/union operation.

### Explicit repeated-token examples

For `"aaaa"` and `"aa"`:

- multiset intersection is `a:2`, union is `a:4`, so the result is `2/4 = 0.5`;
- set mode sees one shared key and one union key, so the result is `1.0`.

### Expected examples

| Call | Result |
| --- | ---: |
| `Jaccard()("test", "text")` | `3/5 = 0.6` |
| `Jaccard()("nelson", "neilsen")` | `5/8 = 0.625` |
| `Jaccard()("decide", "resize")` | `3/9 = 0.333333...` |
| `Jaccard()("aaaa", "aa")` | `0.5` |
| `Jaccard(as_set=True)("aaaa", "aa")` | `1.0` |
| `Jaccard(qval=2)("test", "text")` | `0.2` |
| `Jaccard()("", "")` | `1.0` quick answer |
| `Jaccard()("", "abc")` | `0.0` quick answer |

`Jaccard(ks=[...])` is not a Python option; equality with
`Tversky(ks=[1, 1])` is a cross-algorithm invariant tested by the original
suite. The implementation must support two or more sequences through the
counter operations.

## 3. Sørensen/Dice

**Source:** `token_based.py`, class `Sorensen`
**Tests:** `tests/original/test_token/test_sorensen.py`
**Rust packet:** `rust/src/algorithms/token/sorensen.rs`

### Meaning

Use the same counter intersection as Jaccard. Let `total_count` be the sum of
the counted sizes of all prepared sequences. Return:

```text
2 * intersection_count / total_count
```

`as_set=False` sums repeated counts; `as_set=True` counts distinct keys.
The exported Python aliases `sorensen`, `sorensen_dice`, and `dice` all point
to this behavior.

### Expected examples

| Call | Result |
| --- | ---: |
| `Sorensen()("test", "text")` | `6/8 = 0.75` |
| `Sorensen()("aaaa", "aa")` | `4/6 = 0.666666...` |
| `Sorensen(as_set=True)("aaaa", "aa")` | `1.0` |
| `Sorensen(qval=2)("test", "text")` | `1/3 = 0.333333...` |
| `Sorensen()("", "")` | `1.0` quick answer |
| `Sorensen()("", "abc")` | `0.0` quick answer |

The original tests compare this result with `Tversky(ks=[0.5, 0.5])` for
both multiset and set modes.

## 4. Tversky

**Source:** `token_based.py`, class `Tversky`
**Cross-checks:** Jaccard and Sørensen original tests; no separate source
test file exists in this checkout
**Rust packet:** `rust/src/algorithms/token/tversky.rs`

### Meaning

After counter preparation, let `I` be the intersection count and `S_i` be the
count of sequence `i`. With coefficients `k_i`, when there are not exactly two
sequences or no bias is configured, calculate:

```text
R = I + sum(k_i * (S_i - I))
similarity = I / R
```

The default `ks` is an unlimited stream of `1`s. An explicitly empty false-y
`ks` also falls back to that default. For exactly two sequences with a bias,
the source uses:

```text
a = min(S1, S2)
b = max(S1, S2)
c = I + bias
R = alpha * beta * (a - b) + b * beta
similarity = c / (R + c)
```

The two coefficients are consumed in order as `alpha`, then `beta`.

### Expected examples

| Call | Result |
| --- | ---: |
| `Tversky()("test", "text")` | `0.6` |
| `Tversky(ks=[1, 1])("test", "text")` | `0.6`, same as Jaccard |
| `Tversky(ks=[0.5, 0.5])("test", "text")` | `0.75`, same as Sørensen |
| `Tversky(ks=[2, 1], bias=0.5)("ab", "ac")` | `3/7 = 0.428571...` |
| `Tversky(as_set=True)("aaaa", "aa")` | `1.0` |
| `Tversky()("", "")` | `1.0` quick answer |
| `Tversky()("", "abc")` | `0.0` quick answer |

The implementation must preserve the multi-sequence branch and must not
always force the two-sequence biased formula.

## 5. Cosine

**Source:** `token_based.py`, class `Cosine`
**Tests:** `tests/original/test_token/test_cosine.py`
**Rust packet:** `rust/src/algorithms/token/cosine.rs`

### Meaning

Use the counter intersection count and the counted size of each sequence.
For `n` sequences, calculate:

```text
intersection_count / (product(sequence_count_i) ** (1 / n))
```

The product is formed from all sequences, not just the first two. Repeated
counts are included unless `as_set=True`.

### Expected examples

| Call | Result |
| --- | ---: |
| `Cosine()("test", "text")` | `3/4 = 0.75` |
| `Cosine()("nelson", "neilsen")` | `5/sqrt(42)` |
| `Cosine()("aaaa", "aa")` | `1/sqrt(2) = 0.707106...` |
| `Cosine(as_set=True)("aaaa", "aa")` | `1.0` |
| `Cosine(qval=2)("test", "text")` | `1/3 = 0.333333...` |
| `Cosine()("", "")` | `1.0` quick answer |
| `Cosine()("", "abc")` | `0.0` quick answer |

## 6. Monge-Elkan

**Source:** `token_based.py`, class `MongeElkan`
**Tests:** `tests/original/test_token/test_monge_elkan.py`
**Rust packet:** `rust/src/algorithms/token/monge_elkan.rs`

### Meaning

The constructor accepts:

- an underlying similarity algorithm, defaulting to the shared
  `DamerauLevenshtein()` instance;
- `symmetric=False` by default;
- `qval`, defaulting to `1`;
- `external`, accepted by the base compatibility surface.

For the non-symmetric path, `_calc(first, *remaining)` visits each element of
the first sequence. For every remaining sequence it finds the best underlying
algorithm similarity against that element's candidate matches, appends those
best values, then returns exactly:

```text
sum(best_values) / len(first_sequence) / len(best_values)
```

That second division is part of the source behavior, even though it is not the
usual presentation of an average. Empty first sequences return `0` from
`_calc`. The symmetric path computes `_calc` for every permutation of the
prepared sequences and averages those results.

### Expected examples

Using the original `jaro_winkler` singleton as the underlying algorithm:

| Call | Result |
| --- | ---: |
| `MongeElkan(algorithm=jaro_winkler)(["Niall"], ["Neal"])` | `0.805` |
| `MongeElkan(algorithm=jaro_winkler)(["Niall"], ["Nigel"])` | `0.7866666666666667` |

Additional behavior to preserve:

- `symmetric=True` averages both directions for two sequences and all
  permutations for more sequences;
- `qval` prepares the outer token sequences before matching;
- equal/one/empty inputs use the base similarity quick-answer path before
  q-value preparation;
- `maximum` is delegated to the underlying algorithm and then expanded using
  the source's per-token maximum loop, so it is not always `1`.

### Compatibility risk

The Python `algorithm` option is a polymorphic object with a `.similarity`
method. The frozen Rust trait does not yet define the equivalent callback
boundary. The packet must not silently remove this option; the exact Rust
adapter shape needs a shared API decision before final integration.

## 7. Bag

**Source:** `token_based.py`, class `Bag`
**Tests:** `tests/original/test_token/test_bag.py`
**Rust packet:** `rust/src/algorithms/token/bag.rs`

### Meaning

Bag does not call `quick_answer`. It builds counters, intersects all counters
with minimum counts, subtracts that intersection from each sequence, and
returns the largest counted remainder:

```text
max(count(sequence_i - intersection) for every sequence_i)
```

The Python `as_set` flag is inherited from the base object even though Bag's
constructor does not declare it explicitly; if set on the instance, the
remainder counts become distinct-key counts.

### Expected examples

| Call | Result |
| --- | ---: |
| `Bag()("qwe", "qwe")` | `0` |
| `Bag()("qwe", "erty")` | `3` |
| `Bag()("qwe", "ewq")` | `0` |
| `Bag()("qwe", "rtys")` | `4` |
| `Bag()("aaaa", "aa")` | `2` multiset remainder |
| `bag = Bag(); bag.as_set = True; bag("aaaa", "aa")` | `1` distinct remainder key; `Bag` inherits the attribute but does not accept it in its constructor |
| `Bag()("", "")` | `0` through the empty remainders |

Unlike the token similarity classes, Bag's maximum is the base maximum of the
input sequences and its direct result is a distance.

## 8. LCSStr

**Source:** `sequence_based.py`, class `LCSStr`
**Tests:** `tests/original/test_sequence/test_lcsstr.py`
**Rust packet:** `rust/src/algorithms/sequence/lcsstr.rs`

### Meaning

Return the longest contiguous common substring, not merely its length.
`similarity` returns the length of the returned substring.

The source has three important branches:

1. If any input is empty, return an empty value.
2. With no inputs, return an empty value; with one input, return that input
   immediately, before q-value preparation.
3. With two inputs whose prepared maximum length is below `200`, use
   `difflib.SequenceMatcher.find_longest_match`. Otherwise, or with more than
   two inputs, use the custom search.

The standard two-input branch has deterministic tie-breaking: retain the first
maximum encountered while scanning the first sequence from left to right and
the second sequence in its source order. The custom branch chooses the first
shortest input, searches candidate lengths from longest to shortest, and then
returns the first candidate window that occurs in every sequence.

### Expected examples and tie cases

| Call | Returned substring |
| --- | --- |
| `LCSStr()("ab", "abcd")` | `"ab"` |
| `LCSStr()("abcd", "bc")` | `"bc"` |
| `LCSStr()("abcd", "ef")` | `""` |
| `LCSStr()("ababa", "babab")` | `"abab"` (first tie in the first input) |
| `LCSStr()("abc", "axc")` | `"a"` (earlier than the equally long `"c"`) |
| `LCSStr()("abc", "axc", "zabc")` | `"a"` through the custom multi-input search |
| `LCSStr()("MYTEST" * 100, "TEST")` | `"TEST"` |
| `LCSStr()("abcd")` | `"abcd"` |
| `LCSStr()("", "abc")` | `""` |

For q-values, the source prepares only the multi-input/two-input branch. A
single non-empty input is returned unchanged even when `qval` is `2` or `3`.
For example, `LCSStr(qval=2)("abcd")` returns `"abcd"`, while
`LCSStr(qval=2)("test", "text")` returns the matching q-gram sequence
`[("t", "e")]` in the Python representation.

### Compatibility risk

The Python call returns a string/list sequence, while the shared Rust
`Algorithm` trait exposes a numeric raw score. The Rust packet needs an
algorithm-specific substring-returning method, with the trait score equal to
the returned sequence length; the adapter must convert that result back to the
Python-visible type.

## 9. RLE NCD

**Source:** `compression_based.py`, classes `_NCDBase` and `RLENCD`
**Tests:** `tests/original/test_compression/test_common.py`
**Rust packet:** `rust/src/algorithms/compression/rle_ncd.rs`

### Meaning

The compressor groups consecutive equal elements. Its exact run encoding is:

| Input run | Encoded output |
| --- | --- |
| `A` | `A` |
| `AA` | `AA` |
| `AAA` | `3A` |
| `ABBB` | `A3B` |
| `AAAAA` | `5A` |
| `AABBAAA` | `AABB3A` |

The NCD calculation uses every permutation of the input sequences, joins or
concatenates each permutation, chooses the smallest compressed concatenation,
then applies:

```text
(smallest_concat_size - smallest_single_size * (number_of_sequences - 1))
/ largest_single_size
```

With no sequences, return `0`. If every compressed input size is zero, return
`0`. There is no quick-answer equality shortcut, so equal inputs do not always
produce zero.

### Expected examples

| Call | Result |
| --- | ---: |
| `rle_ncd("", "")` | `0` |
| `rle_ncd("A", "A")` | `1.0` |
| `rle_ncd("AA", "AA")` | `0.0` |
| `rle_ncd("AAA", "AAA")` | `0.0` |
| `rle_ncd("ABBB", "ABBB")` | `1.0` |
| `rle_ncd("test", "test")` | `1.0` |
| `rle_ncd("test", "text")` | `1.0` |

### Compatibility risks

- The source compressor constructs strings (`str(count) + element` and
  `''.join(...)`), so non-string elements are not silently accepted.
- With `qval>1`, Python q-grams are tuples; passing them into `RLENCD` reaches
  the string join and raises a `TypeError`. The Rust boundary must preserve
  this unsupported-input behavior as a clear error rather than silently
  stringifying q-grams.
- `qval=None` creates word lists and remains string-token based; its behavior
  must be tested separately from character mode.
- The source uses all sequence permutations, so concatenation order is part of
  the result even though the final value is symmetric.

## Lane 2 API-gap register

These are concrete gaps between an observable Python call and the currently
frozen Rust trait. They are recorded here so the adapter owner can resolve them
without guessing. Monge-Elkan and LCSStr remain blocked in Lane 3 until their
shared adapter contract is resolved.

| Area | Original Python call | Expected output/error | Proposed Rust representation |
| --- | --- | --- | --- |
| Common `maximum` | `Hamming(qval=2).maximum("test", "text")` | `4`; the source maximum uses the original inputs, while prepared q-grams have length `3` | Adapter call object carrying original lengths alongside prepared sequences, or a shared `maximum` context owned by Simha |
| Common `maximum` | `rle_ncd.maximum("test", "text")` | `1`, even though the generic Rust default maximum is sequence length | `RLENCD`-specific `maximum` override or a common score-context rule |
| Hamming comparator | `Hamming(test_func=lambda *items: True)("a", "b")` | `0`; the custom function declares every aligned position equal | `Arc<dyn Fn(&[Option<&Element>]) -> bool + Send + Sync>` or an adapter-owned comparator callback |
| Hamming missing values | `Hamming(test_func=lambda *items: any(item is None for item in items))("a", "ab")` | `1`; the second `zip_longest` position contains Python `None` and `"b"` | Comparator receives `Option<&Element>` values, preserving the `None` fill marker |
| Tversky short biased coefficients | `Tversky(ks=[1], bias=0.5)("a", "b")` | `ValueError: not enough values to unpack (expected 2, got 1)` | Fallible `try_similarity`/`Result` API with a `MissingCoefficient` error; do not silently invent `beta` |
| Tversky zero denominator | `Tversky(ks=[0, 0])("a", "b")` | `ZeroDivisionError` | Fallible score API with a `ZeroDenominator` error |
| Bag no arguments | `Bag()()` | `IndexError: tuple index out of range` | Fallible `try_raw_score` with an `EmptyInputList` error; the current numeric trait cannot carry this error |
| Monge-Elkan algorithm option | `MongeElkan(algorithm=textdistance.jaro_winkler)(["Niall"], ["Neal"])` | `0.805` | Shared similarity-algorithm object/trait with `similarity`, `maximum`, and prepared-element support; blocked pending Simha's adapter decision |
| LCSStr returned value | `LCSStr()("ababa", "babab")` | `"abab"`, not just numeric length `4` | Owned substring method returning `PreparedSequence` plus trait raw score `4`; adapter converts the sequence back to Python's return type; blocked pending Simha |
| RLE unsupported q-grams | `RLENCD(qval=2)("test", "text")` | `TypeError: sequence item 0: expected str instance, tuple found` | `Result<f64, RleError::UnsupportedElement>` at the adapter boundary; never stringify `Element::Gram` |
| RLE unsupported integers | `RLENCD()([1, 2], [1, 2])` | `TypeError: sequence item 0: expected str instance, int found` | Same fallible RLE API, rejecting `Byte`, `Integer`, and `Boolean` elements clearly |
| Optional external dispatch | `Jaccard(external=True)("test", "text")` | `0.6`; third-party dispatch is an optimization, not a different answer | Preserve an accepted `external` option in the compatibility layer, but keep the Rust core independent and ignore dispatch as required by the main PRD |
| Native test discovery | `cargo test --test poojitha` for the owned native harness | Cargo reports `error: no test target named poojitha` because the frozen `Cargo.toml` declares only `core_contract` and `registry` | Simha should add an explicit `[[test]]` entry for `rust/tests/poojitha.rs` (or an equivalent shared test-discovery rule); until then, run the owned harness with the documented direct `rustc --test` command |

These are recorded as coordination items rather than changed or hidden. The
main PRD requires shared API ownership and prohibits an algorithm owner from
silently changing the frozen core contract. Lane 3 therefore implements every
valid prepared-sequence calculation in the owned files, while the listed
fallible/callback/return-value cases remain explicit handoff items.

The native test-discovery row is a build-configuration gap, not an algorithm
behavior change. It is recorded here because the main PRD requires focused
native tests while the current shared Cargo manifest does not discover the
owned harness, and the owner boundary forbids changing that manifest in this
packet.
