# INT-06 — Benchmark Report (Suri's ten algorithm packets)

**Status: CAPTURED.** The Rust and frozen-Python baselines were run from the
same working tree on 2026-08-03. Raw results are in
`bench/results/rust_bench.json` and `bench/results/python_bench.json`.

## Scope and commands

The report covers SUR-01 through SUR-10: Overlap, Tanimoto,
Ratcliff-Obershelp, BWT-RLE NCD, MRA, Prefix, Postfix, Length, Identity, and
Matrix.

```sh
cargo bench --bench suri_bench
.venvs/g0/bin/python bench/scripts/bench_python_baseline.py
```

The Rust command calls the native crate directly. The Python command extracts
the frozen pre-port package from baseline commit `d6a68d6` with `git archive`,
so it never imports the current Rust-backed package.

## Environment

| Item | Value |
| --- | --- |
| OS | macOS Darwin 25.5.0, arm64 |
| Rust | `rustc 1.97.1`, `cargo 1.97.1` |
| Python | `3.14.6` (`.venvs/g0`) |
| Branch | `teja` |
| Baseline | `d6a68d6` |
| Rust profile | Cargo `bench` (optimized) |

## Method

Both sides run the same three short pairs — `("text", "test")`,
`("qwer", "asdf")`, and `("a" * 15, "b" * 15)` — for 4,000 repetitions
(12,000 calls per algorithm), after one warm-up pass. A separate 2,000-character
pair is timed once as a long-case signal. Rust also records allocations per
call with a counting allocator; Python records peak traced bytes with
`tracemalloc`. Those memory measures are contextual, not directly equivalent.

## Short-case results

| Algorithm | Rust s/call | Python s/call | Python/Rust |
| --- | ---: | ---: | ---: |
| Overlap | 0.000000860 | 0.000008038 | 9.3x |
| Tanimoto | 0.000000467 | 0.000008803 | 18.9x |
| Ratcliff-Obershelp | 0.000000476 | 0.000010571 | 22.2x |
| BWT-RLE NCD | 0.000007396 | 0.000058897 | 8.0x |
| MRA | 0.000000502 | 0.000024310 | 48.4x |
| Prefix | 0.000000024 | 0.000002101 | 87.5x |
| Postfix | 0.000000022 | 0.000003364 | 152.9x |
| Length | 0.000000001 | 0.000000629 | ~629x* |
| Identity | 0.000000003 | 0.000000254 | 84.7x |
| Matrix | 0.000000004 | 0.000000280 | 70.1x |

\* The simple Rust timings are near the clock-resolution floor.

## Long-case results

| Algorithm | Rust seconds | Python seconds | Observation |
| --- | ---: | ---: | --- |
| Overlap | 0.000081917 | 0.000061375 | Python 1.3x faster in this single sample |
| Tanimoto | 0.000040292 | 0.000063708 | Rust 1.6x faster |
| Ratcliff-Obershelp | 0.000099416 | 0.001604542 | Rust 16.1x faster |
| BWT-RLE NCD | 0.325393167 | 0.008011792 | Python 40.6x faster; investigate before performance sign-off |
| MRA | 0.000021750 | 0.000239041 | Rust 11.0x faster |
| Prefix | 0.000000166 | 0.000001667 | Rust 10.0x faster |
| Postfix | 0.000000167 | 0.000015250 | Rust 91.3x faster |
| Length | below timer resolution | 0.000000958 | Rust faster; not measurable precisely |
| Identity | below timer resolution | 0.000001708 | Rust faster; not measurable precisely |
| Matrix | below timer resolution | 0.000001459 | Rust faster; not measurable precisely |

The BWT-RLE long-case regression is recorded rather than hidden: the current
Rust implementation materializes and sorts many `Vec<Element>` rotations,
which is substantially more expensive than the baseline's string operations.

## Memory evidence

| Algorithm | Rust bytes/call | Python peak bytes over timed loop |
| --- | ---: | ---: |
| Overlap | 1501.3 | 3272 |
| Tanimoto | 1957.3 | 1280 |
| Ratcliff-Obershelp | 1010.7 | 1633 |
| BWT-RLE NCD | 60646.7 | 3583 |
| MRA | 512.0 | 1128 |
| Prefix | 245.3 | 744 |
| Postfix | 245.3 | 1128 |
| Length | 0.0 | 176 |
| Identity | 0.0 | 216 |
| Matrix | 0.0 | 216 |

## Files

- `rust/benches/suri_bench.rs` — native benchmark harness.
- `bench/scripts/bench_python_baseline.py` — frozen baseline runner.
- `bench/results/rust_bench.json` — raw Rust results.
- `bench/results/python_bench.json` — raw Python results.
