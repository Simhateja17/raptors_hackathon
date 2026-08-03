# Rust-port decisions

## Language and runtime boundary

The algorithm implementations live in a standalone Rust crate. Python keeps
the original public class and singleton names, but each supported algorithm
delegates through the compiled `textdistance_port` PyO3 module. The adapter
contains argument normalization, option translation, and result
reconstruction; it does not implement distance algorithms.

The package has no pure-Python fallback. If the extension is missing,
`textdistance/_rust_adapter.py` raises an explicit build/setup error. This
keeps verification honest: a passing Python test cannot silently execute the
old implementation.

## Input contract

The Rust core accepts Unicode scalar values for Python strings, bytes as
`u8`, homogeneous integer and boolean sequences, and prepared Rust elements
for native tests. `qval=None` is word preparation, `qval=1` is element
preparation, and larger q-values are n-grams. Unsupported values are rejected
at the adapter boundary instead of being stringified.

## Callbacks and matrix scoring

Arbitrary Python callbacks cannot safely cross the FFI boundary. The adapter
rejects unsupported `test_func`/`sim_test` callbacks and maps the built-in
comparators used by Monge-Elkan to named Rust strategies. Matrix values are
copied into Rust-owned data at construction time; floating-point match and
mismatch costs remain supported.

## Compression behavior

Compression crates are pinned through `Cargo.lock`. Arithmetic NCD uses exact
rational arithmetic when intermediate values fit and a bounded entropy-style
approximation for very long inputs so adversarial Unicode input cannot panic
from `u128` overflow. The fixed corpus and original tests cover the observable
distance values.

## Verification policy

`tests/original/` is a byte-for-byte snapshot and is never edited. The
manifest check, native Rust tests, 114-case differential corpus, deterministic
fuzz smoke test, and benchmark commands are independent evidence layers.
`make test` runs the complete non-external original suite. `make test-external`
also runs optional-provider comparisons; in the current Python 3.14/RapidFuzz
3.14.5 environment, three tests expose a RapidFuzz large-integer sequence
coercion mismatch. The frozen pure-Python implementation produces the same
values as the Rust port for that pair, so the port is not changed to imitate
the provider quirk.
