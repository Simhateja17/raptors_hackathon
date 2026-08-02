"""INT-06 baseline benchmark: pure-Python original TextDistance algorithms.

Extracts the frozen baseline commit's ``textdistance`` package (commit
d6a68d6, see proof/baseline.md) into a temporary directory with
``git archive`` and times it the same way as the Rust harness
(rust/benches/suri_bench.rs): the same three short pairs, RUNS=4000
repetitions, plus one longer pair to surface algorithmic-complexity
differences.

This deliberately never imports the current working tree's ``textdistance``
package: that package is Rust-backed only (see textdistance/_rust_adapter.py)
and raises ImportError instead of falling back to a pure-Python
implementation, so it cannot serve as the "original Python" side of this
comparison.

Run with: python bench/scripts/bench_python_baseline.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tarfile
import tempfile
import time
import tracemalloc
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BASELINE_COMMIT = 'd6a68d6'
RUNS = 4000

SHORT_CASES = [
    ('text', 'test'),
    ('qwer', 'asdf'),
    ('a' * 15, 'b' * 15),
]


def long_case() -> tuple[str, str]:
    left = ''.join(chr(ord('a') + (i % 23)) for i in range(2000))
    right = ''.join(chr(ord('a') + ((i + 5) % 23)) for i in range(2000))
    return left, right


def extract_baseline(dest: Path) -> None:
    """Export the frozen baseline commit's textdistance/ tree into ``dest``."""
    archive = subprocess.run(
        ['git', 'archive', BASELINE_COMMIT, 'textdistance'],
        cwd=REPO_ROOT,
        capture_output=True,
        check=True,
    ).stdout
    archive_path = dest / 'baseline.tar'
    archive_path.write_bytes(archive)
    with tarfile.open(archive_path) as tar:
        tar.extractall(dest)
    archive_path.unlink()


def bench_one(name: str, factory, cases: list[tuple[str, str]]) -> dict:
    func = factory()
    for a, b in cases:
        func(a, b)  # warm up

    tracemalloc.start()
    start = time.perf_counter()
    for _ in range(RUNS):
        for a, b in cases:
            func(a, b)
    elapsed = time.perf_counter() - start
    _current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()

    total_calls = RUNS * len(cases)
    seconds_per_call = elapsed / total_calls

    left, right = long_case()
    long_start = time.perf_counter()
    func(left, right)
    long_elapsed = time.perf_counter() - long_start

    return {
        'algorithm': name,
        'total_calls': total_calls,
        'total_seconds': elapsed,
        'seconds_per_call': seconds_per_call,
        'calls_per_second': 1.0 / seconds_per_call,
        'traced_peak_bytes_over_timed_loop': peak,
        'long_case_seconds': long_elapsed,
    }


def main() -> None:
    with tempfile.TemporaryDirectory(prefix='textdistance-baseline-') as tmp:
        tmp_path = Path(tmp)
        extract_baseline(tmp_path)
        sys.path.insert(0, str(tmp_path))
        for mod_name in list(sys.modules):
            if mod_name == 'textdistance' or mod_name.startswith('textdistance.'):
                del sys.modules[mod_name]

        from textdistance.algorithms.compression_based import BWTRLENCD
        from textdistance.algorithms.phonetic import MRA
        from textdistance.algorithms.sequence_based import RatcliffObershelp
        from textdistance.algorithms.simple import (
            Identity,
            Length,
            Matrix,
            Postfix,
            Prefix,
        )
        from textdistance.algorithms.token_based import Overlap, Tanimoto

        assert 'textdistance_rust' not in sys.modules

        algorithms = [
            ('overlap', lambda: Overlap(external=False)),
            ('tanimoto', lambda: Tanimoto(external=False)),
            ('ratcliff_obershelp', lambda: RatcliffObershelp(external=False)),
            ('bwtrle_ncd', lambda: BWTRLENCD()),
            ('mra', lambda: MRA(external=False)),
            ('prefix', lambda: Prefix()),
            ('postfix', lambda: Postfix()),
            ('length', lambda: Length(external=False)),
            ('identity', lambda: Identity(external=False)),
            ('matrix', lambda: Matrix(external=False)),
        ]

        results = [bench_one(name, factory, SHORT_CASES) for name, factory in algorithms]

    for r in results:
        print(
            f"{r['algorithm']:<20} {r['seconds_per_call']:.9f} s/call  "
            f"{r['calls_per_second']:>14.1f} calls/s  "
            f"long-case {r['long_case_seconds']:.9f} s",
        )

    out_dir = REPO_ROOT / 'bench' / 'results'
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / 'python_bench.json').write_text(
        json.dumps(
            {
                'runs': RUNS,
                'cases_per_run': len(SHORT_CASES),
                'baseline_commit': BASELINE_COMMIT,
                'python_version': sys.version,
                'algorithms': results,
            },
            indent=2,
        ),
        encoding='utf8',
    )


if __name__ == '__main__':
    main()
