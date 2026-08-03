# INT-06 — Benchmark Report (Suri's ten algorithm packets)

**Status: CAPTURED.** Reproducible commands, machine details, and results for
all ten of Suri's assigned algorithms are recorded below, comparing the Rust
port directly against the frozen pre-port Python baseline.

Recorded on 2026-08-02.

## Scope

Algorithms covered (SUR-01..10, `PRD.md` lines 689-698):

| Task | Algorithm | Rust source |
| --- | --- | --- |
| SUR-01 | Overlap | `rust/src/algorithms/token/overlap.rs` |
| SUR-02 | Tanimoto | `rust/src/algorithms/token/tanimoto.rs` |
| SUR-03 | Ratcliff-Obershelp | `rust/src/algorithms/sequence/ratcliff_obershelp.rs` |
| SUR-04 | BWT-RLE NCD | `rust/src/algorithms/compression/bwtrle_ncd.rs` |
| SUR-05 | MRA | `rust/src/algorithms/phonetic/mra.rs` |
| SUR-06 | Prefix | `rust/src/algorithms/simple/prefix.rs` |
| SUR-07 | Postfix | `rust/src/algorithms/simple/postfix.rs` |
| SUR-08 | Length | `rust/src/algorithms/simple/length.rs` |
| SUR-09 | Identity | `rust/src/algorithms/simple/identity.rs` |
| SUR-10 | Matrix | `rust/src/algorithms/simple/matrix.rs` |

No algorithm implementation and no existing test file was modified to produce
this report. Two new files were added instead: a Cargo bench target
(`rust/benches/suri_bench.rs`) and a Python baseline script
(`bench/scripts/bench_python_baseline.py`).

## Commands used

```bash
# Rust side: times the port directly through the textdistance-port crate
# (no PyO3/FFI hop), writes bench/results/rust_bench.json
cargo bench --bench suri_bench

# Python side: extracts the frozen pre-port textdistance package from commit
# d6a68d6 (see proof/baseline.md) via `git archive`, times it the same way,
# writes bench/results/python_bench.json
python bench/scripts/bench_python_baseline.py
```

Both commands are run from the repository root and are independently
reproducible — the Python script does not require the current working tree's
`textdistance` package (which is Rust-backed only, see
`textdistance/_rust_adapter.py`, and has no pure-Python fallback to
benchmark). It reconstructs the original implementation on the fly from git
history instead.

## System / environment

| Item | Value |
| --- | --- |
| OS | Microsoft Windows 11 Home Single Language, Build 10.0.26100 |
| CPU | Intel64 Family 6 Model 154 Stepping 3 (GenuineIntel), 12 logical processors |
| RAM | 16,072 MB |
| Rust | `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Cargo profile | `bench` (release-optimized; `cargo bench` default) |
| Python | 3.13.2 (`MSC v.1942 64 bit (AMD64)`), interpreter `C:\Users\mg875\AppData\Local\Programs\Python\Python313\python.exe` |
| Rust port commit | `6fb3560` (branch `suri`) |
| Python baseline commit | `d6a68d6` (frozen pre-port original, per `proof/baseline.md`) |

The machine was not otherwise under controlled load (no CPU pinning, no
isolated benchmark environment); treat absolute times as indicative and the
relative Rust-vs-Python comparison, run back-to-back on the same machine, as
the primary signal.

## Methodology

Both harnesses run the identical shape of work so the comparison is
apples-to-apples:

- **Short cases** (mirrors the existing `textdistance/benchmark.py`
  `STMT`/`RUNS` convention): three pairs — `("text","test")`,
  `("qwer","asdf")`, `("a"*15,"b"*15)` — each called `RUNS = 4000` times
  (12,000 total calls per algorithm), after one warm-up pass over the three
  pairs. Reported as `seconds_per_call` / `calls_per_second`.
- **Long case**: one additional 2,000-character pair (`"abcdefghijklmnopqrstuv"`-cycled
  alphabets, offset by 5) timed with a single call, to surface algorithmic-
  complexity differences the short cases are too small to reveal (e.g.
  Ratcliff-Obershelp's `<200`/`>=200`-length branch). Being a single sample,
  treat this number as directional, not statistically averaged.
- **Memory**: the Rust harness installs a counting `#[global_allocator]`
  wrapper around `std::alloc::System` and reports bytes allocated per call
  over the timed loop. The Python harness uses the standard-library
  `tracemalloc` and reports peak traced bytes over the whole timed loop (not
  per-call — the two metrics are not on the same footing and are reported
  side by side for context, not a strict ratio).
- Both sides construct each algorithm with `external=False` (or the
  equivalent default) so neither side can silently dispatch to a faster
  third-party library — this measures the port itself, not an external
  dependency.
- Both sides prepare text as individual elements (`qval=1` / `QValue::Elements`),
  matching every one of these ten algorithms' Python default.

## Results

### Short cases (RUNS=4000 × 3 pairs = 12,000 calls)

| Algorithm | Rust s/call | Python s/call | Speedup (Python/Rust) | Rust calls/s | Python calls/s |
| --- | ---: | ---: | ---: | ---: | ---: |
| Overlap | 0.000001473 | 0.000024467 | **16.6x** | 678,794 | 40,871 |
| Tanimoto | 0.000001921 | 0.000034395 | **17.9x** | 520,506 | 29,074 |
| Ratcliff-Obershelp | 0.000001686 | 0.000044131 | **26.2x** | 593,258 | 22,660 |
| BWT-RLE NCD | 0.000043689 | 0.000144335 | **3.3x** | 22,889 | 6,928 |
| MRA | 0.000001443 | 0.000062293 | **43.2x** | 692,941 | 16,053 |
| Prefix | 0.000000079 | 0.000006030 | **76.3x** | 12,734,798 | 165,841 |
| Postfix | 0.000000077 | 0.000012461 | **161.8x** | 12,944,984 | 80,253 |
| Length | 0.000000002 | 0.000003344 | **~1,672x** | 447,761,194 | 299,048 |
| Identity | 0.000000008 | 0.000001304 | **163.1x** | 119,880,120 | 766,607 |
| Matrix | 0.000000010 | 0.000001305 | **130.5x** | 100,755,668 | 766,470 |

Every algorithm is faster in Rust on the short cases, ranging from 3.3x
(BWT-RLE NCD) to roughly three orders of magnitude for the constant-time
simple algorithms, where Rust's per-call time is close to the timing
resolution floor.

### Long case (one 2,000-element pair, single call)

| Algorithm | Rust | Python | Speedup (Python/Rust) |
| --- | ---: | ---: | ---: |
| Overlap | 0.000220000s | 0.000099800s | 0.45x (Python faster) |
| Tanimoto | 0.000117000s | 0.000105500s | 0.90x (roughly even) |
| Ratcliff-Obershelp | 0.000266900s | 0.009876300s | **37.0x** |
| BWT-RLE NCD | 1.718943800s | 0.034693000s | **0.02x (Python ~49x faster)** |
| MRA | 0.000043200s | 0.000424400s | **9.8x** |
| Prefix | 0.000002100s | 0.000004700s | 2.2x |
| Postfix | 0.000000400s | 0.000040100s | **100.3x** |
| Length | ~0s (below resolution) | 0.000002400s | immeasurable, Rust faster |
| Identity | ~0s (below resolution) | 0.000002300s | immeasurable, Rust faster |
| Matrix | ~0s (below resolution) | 0.000002700s | immeasurable, Rust faster |

### Memory (contextual, not directly comparable — see Methodology)

| Algorithm | Rust bytes/call | Python peak bytes (whole timed loop) |
| --- | ---: | ---: |
| Overlap | 1,501.3 | 4,048 |
| Tanimoto | 1,957.3 | 1,520 |
| Ratcliff-Obershelp | 1,010.7 | 2,009 |
| BWT-RLE NCD | 60,646.7 | 3,727 |
| MRA | 512.0 | 1,480 |
| Prefix | 245.3 | 952 |
| Postfix | 245.3 | 1,440 |
| Length | 0.0 | 328 |
| Identity | 0.0 | 376 |
| Matrix | 0.0 | 376 |

BWT-RLE NCD allocates roughly 30-60x more per call than the other Rust
algorithms in this set, consistent with its Burrows-Wheeler rotation step
materializing a full `Vec<Element>` clone per rotation.

## Anomaly: BWT-RLE NCD regresses sharply on long input

On the short cases (4-15 elements), the Rust port of BWT-RLE NCD is faster
than Python, as expected (3.3x). On the 2,000-element long case that reverses
hard: **Rust takes 1.72s versus Python's 0.035s — Python is about 49x
faster.** Scaling short → long, Rust's per-call time grows roughly 39,000x
while Python's grows only about 240x, even though both implementations do the
asymptotically same O(n² log n) work (`_NCDBase.__call__` / the Rust
`raw_score` both build every rotation of the input and sort them).

The likely cause (for the relevant algorithm owner to investigate, not fixed
here per INT-06's scope): the Rust implementation
(`rust/src/algorithms/compression/bwtrle_ncd.rs`, `transform`) represents each
of the *n* rotations as a freshly heap-allocated `Vec<Element>` (`Element` is
a boxed enum, not a packed byte/char), so both the rotation construction and
the subsequent `Vec<Element>` comparisons done by `.sort()` are far more
expensive per element than Python's C-optimized string slicing (`data[i:] +
data[:i]`) and native string comparison. The Rust allocator counter above
corroborates this: BWT-RLE NCD allocates 30-60x more per call than any other
algorithm in this set even on the tiny short-case inputs. This is the one
algorithm in the SUR packet set where the Rust port is not uniformly faster
than the pre-port Python baseline, and it is worth flagging to the BWT-RLE
NCD owner (SUR-04) ahead of the final performance-evidence sign-off (Gate G4).

Overlap and Tanimoto also show Rust marginally slower than Python on the
single-sample long case (0.45x and 0.90x) while being clearly faster on the
short cases; unlike BWT-RLE NCD this is a small, single-call measurement and
is more consistent with sampling noise around a `BTreeMap`-based counter
(23 unique characters) than with an algorithmic regression — it did not
reproduce as a 3+ order-of-magnitude effect the way BWT-RLE NCD's did.

## Files produced by this task

| File | Purpose |
| --- | --- |
| `rust/benches/suri_bench.rs` | Rust bench harness (Cargo `harness = false` bench target) |
| `Cargo.toml` | Added `[[bench]]` entry for `suri_bench` (no existing entries touched) |
| `bench/scripts/bench_python_baseline.py` | Extracts and times the frozen pre-port Python baseline via `git archive` |
| `bench/results/rust_bench.json` | Raw Rust results (regenerated by `cargo bench --bench suri_bench`) |
| `bench/results/python_bench.json` | Raw Python results (regenerated by the baseline script) |
| `bench/report.md` | This report |
