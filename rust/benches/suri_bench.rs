//! INT-06 benchmark harness for Suri's ten assigned algorithm packets
//! (Overlap, Tanimoto, Ratcliff-Obershelp, BWT-RLE NCD, MRA, Prefix, Postfix,
//! Length, Identity, Matrix).
//!
//! This is a plain `harness = false` Cargo bench: no algorithm implementation
//! or existing test file is touched. It times each algorithm directly through
//! the `textdistance-port` crate (no PyO3/FFI hop) and writes a machine-
//! readable result file that `bench/scripts/build_report.py` turns into the
//! human-readable report.
//!
//! Run with: `cargo bench --bench suri_bench`

use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use textdistance_port::algorithms::compression::bwtrle_ncd::BWTRLENCD;
use textdistance_port::algorithms::phonetic::mra::MRA;
use textdistance_port::algorithms::sequence::ratcliff_obershelp::RatcliffObershelp;
use textdistance_port::algorithms::simple::identity::Identity;
use textdistance_port::algorithms::simple::length::Length;
use textdistance_port::algorithms::simple::matrix::Matrix;
use textdistance_port::algorithms::simple::postfix::Postfix;
use textdistance_port::algorithms::simple::prefix::Prefix;
use textdistance_port::algorithms::token::overlap::Overlap;
use textdistance_port::algorithms::token::tanimoto::Tanimoto;
use textdistance_port::{
    prepare_sequences, InputSequence, OutputAlgorithm, PreparedSequence, QValue,
};

/// Counting allocator: gives byte/allocation-count evidence for the "memory
/// usage (if available)" requirement without adding a profiling dependency.
struct CountingAllocator;

static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Matches the Python-side `textdistance/benchmark.py` STMT/RUNS convention
/// so the two sides measure the same shape of work.
const RUNS: u32 = 4000;

fn short_cases() -> Vec<(String, String)> {
    vec![
        ("text".to_string(), "test".to_string()),
        ("qwer".to_string(), "asdf".to_string()),
        ("a".repeat(15), "b".repeat(15)),
    ]
}

/// One longer, less repetitive pair used to surface algorithmic-complexity
/// differences (e.g. Ratcliff-Obershelp's <200 vs >=200 length branch) that
/// the short cases are too small to reveal.
fn long_case() -> (String, String) {
    let left: String = (0..2000)
        .map(|i| char::from(b'a' + (i % 23) as u8))
        .collect();
    let right: String = (0..2000)
        .map(|i| char::from(b'a' + ((i + 5) % 23) as u8))
        .collect();
    (left, right)
}

fn prepare(pairs: &[(String, String)]) -> Vec<Vec<PreparedSequence>> {
    pairs
        .iter()
        .map(|(a, b)| {
            prepare_sequences(
                &[
                    InputSequence::Text(a.clone()),
                    InputSequence::Text(b.clone()),
                ],
                QValue::Elements,
            )
            .expect("text inputs always prepare under QValue::Elements")
        })
        .collect()
}

struct AlgoResult {
    algorithm: String,
    total_calls: u64,
    total_seconds: f64,
    seconds_per_call: f64,
    calls_per_second: f64,
    bytes_allocated_per_call: f64,
    allocations_per_call: f64,
    long_case_seconds: f64,
}

fn bench_short<A: OutputAlgorithm>(
    name: &str,
    algo: &A,
    cases: &[Vec<PreparedSequence>],
) -> AlgoResult {
    // Warm up (page faults, branch predictor, first allocations) before the
    // timed and the allocation-counted sections.
    for case in cases {
        black_box(algo.output(case).ok());
    }

    let alloc_before = ALLOCATED_BYTES.load(Ordering::Relaxed);
    let calls_before = ALLOC_CALLS.load(Ordering::Relaxed);

    let start = Instant::now();
    for _ in 0..RUNS {
        for case in cases {
            black_box(algo.output(case).ok());
        }
    }
    let elapsed = start.elapsed();

    let alloc_after = ALLOCATED_BYTES.load(Ordering::Relaxed);
    let calls_after = ALLOC_CALLS.load(Ordering::Relaxed);

    let total_calls = RUNS as u64 * cases.len() as u64;
    let total_seconds = elapsed.as_secs_f64();
    let seconds_per_call = total_seconds / total_calls as f64;

    let long = long_case();
    let long_prepared = prepare(std::slice::from_ref(&long));
    let long_start = Instant::now();
    black_box(algo.output(&long_prepared[0]).ok());
    let long_elapsed = long_start.elapsed().as_secs_f64();

    AlgoResult {
        algorithm: name.to_string(),
        total_calls,
        total_seconds,
        seconds_per_call,
        calls_per_second: 1.0 / seconds_per_call,
        bytes_allocated_per_call: (alloc_after - alloc_before) as f64 / total_calls as f64,
        allocations_per_call: (calls_after - calls_before) as f64 / total_calls as f64,
        long_case_seconds: long_elapsed,
    }
}

fn to_json(results: &[AlgoResult]) -> String {
    let mut out = String::from("{\n  \"runs\": ");
    out.push_str(&RUNS.to_string());
    out.push_str(",\n  \"cases_per_run\": 3,\n  \"algorithms\": [\n");
    for (i, r) in results.iter().enumerate() {
        out.push_str(&format!(
            "    {{\n      \"algorithm\": \"{}\",\n      \"total_calls\": {},\n      \"total_seconds\": {:.9},\n      \"seconds_per_call\": {:.9},\n      \"calls_per_second\": {:.2},\n      \"bytes_allocated_per_call\": {:.2},\n      \"allocations_per_call\": {:.3},\n      \"long_case_seconds\": {:.9}\n    }}",
            r.algorithm,
            r.total_calls,
            r.total_seconds,
            r.seconds_per_call,
            r.calls_per_second,
            r.bytes_allocated_per_call,
            r.allocations_per_call,
            r.long_case_seconds,
        ));
        if i + 1 < results.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn main() {
    let cases = prepare(&short_cases());

    let mut results = Vec::new();
    results.push(bench_short("overlap", &Overlap::default(), &cases));
    results.push(bench_short("tanimoto", &Tanimoto::default(), &cases));
    results.push(bench_short(
        "ratcliff_obershelp",
        &RatcliffObershelp::new(),
        &cases,
    ));
    results.push(bench_short("bwtrle_ncd", &BWTRLENCD::new(), &cases));
    results.push(bench_short("mra", &MRA::new(), &cases));
    results.push(bench_short("prefix", &Prefix::new(), &cases));
    results.push(bench_short("postfix", &Postfix::new(), &cases));
    results.push(bench_short("length", &Length::new(), &cases));
    results.push(bench_short("identity", &Identity::new(), &cases));
    results.push(bench_short("matrix", &Matrix::default(), &cases));

    for r in &results {
        println!(
            "{:<20} {:>12.9} s/call  {:>14.1} calls/s  {:>10.1} bytes/call  long-case {:>10.9} s",
            r.algorithm,
            r.seconds_per_call,
            r.calls_per_second,
            r.bytes_allocated_per_call,
            r.long_case_seconds,
        );
    }

    let json = to_json(&results);
    fs::create_dir_all("bench/results").expect("bench/results is created ahead of the run");
    fs::write("bench/results/rust_bench.json", json).expect("write bench/results/rust_bench.json");
}
