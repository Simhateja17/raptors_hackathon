# Five-minute demo

Run from the repository root after installing the documented Python test
dependencies:

```sh
make demo
```

The demo calls three distinct Rust-backed paths and prints the results:

- Levenshtein shows a scalar edit distance.
- Jaro-Winkler shows a similarity algorithm over Unicode-aware Rust elements.
- Prefix with `qval=2` shows sequence reconstruction through the adapter.
- Matrix with floating-point costs shows option translation across FFI.

Then show the proof gates:

```sh
make verify
```

Point out that the command first verifies every original-test hash, then runs
native Rust tests, the 114-case corpus, the seeded fuzz smoke test, and the
400-case non-external original suite. Finally, run `make benchmark` to show
the reproducible Rust/native and frozen-Python results in `bench/report.md`.
