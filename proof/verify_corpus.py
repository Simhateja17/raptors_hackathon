"""INT-04: replay the frozen differential corpus under proof/corpus/.

This script proves the Rust-backed `textdistance` package still produces the
values recorded in the corpus. It only imports `textdistance`, which (since
INT-01) is wired to the compiled `textdistance_rust` PyO3 extension and never
calls the original pure-Python algorithm implementations. Running this script
therefore requires the Rust build, not the original Python implementation.

Usage:
    python proof/verify_corpus.py
Exit code is 0 iff every case in every corpus file matches.
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import textdistance as td  # noqa: E402

CORPUS_DIR = Path(__file__).parent / 'corpus'

ALGORITHMS = {
    'overlap': td.Overlap,
    'tanimoto': td.Tanimoto,
    'ratcliff_obershelp': td.RatcliffObershelp,
    'bwtrle_ncd': td.BWTRLENCD,
    'mra': td.MRA,
    'prefix': td.Prefix,
    'postfix': td.Postfix,
    'length': td.Length,
    'identity': td.Identity,
    'matrix': td.Matrix,
}


def decode_input(value):
    """Reverse the corpus's JSON-safe encoding of non-JSON-native inputs."""
    if isinstance(value, dict) and '__bytes_hex__' in value:
        return bytes.fromhex(value['__bytes_hex__'])
    if isinstance(value, list):
        return [decode_input(v) for v in value]
    return value


def encode_output(value):
    """Mirror the corpus generator's encoding so live results are comparable."""
    if isinstance(value, bytes):
        return value.hex()
    return value


def values_match(actual, expected) -> bool:
    if isinstance(expected, float) and isinstance(actual, (int, float)):
        if math.isinf(expected) or math.isinf(actual):
            return actual == expected
        return math.isclose(actual, expected, rel_tol=1e-9, abs_tol=1e-12)
    return actual == expected


def run_case(cls, case: dict) -> list[str]:
    """Return a list of mismatch descriptions (empty means the case passed)."""
    inputs = [decode_input(v) for v in case['inputs']]
    options = case.get('options', {})
    alg = cls(**options)
    failures = []
    for method, expected in case['expected'].items():
        try:
            if method == 'call':
                actual = encode_output(alg(*inputs))
            else:
                actual = getattr(alg, method)(*inputs)
        except Exception as error:  # noqa: BLE001 - surfaced as a mismatch, not a crash
            failures.append(f'{method}: raised {type(error).__name__}: {error}')
            continue
        if not values_match(actual, expected):
            failures.append(f'{method}: expected {expected!r}, got {actual!r}')
    return failures


def main() -> int:
    total = 0
    failed = 0
    for name, cls in ALGORITHMS.items():
        path = CORPUS_DIR / f'{name}.json'
        if not path.exists():
            print(f'MISSING corpus file: {path}')
            failed += 1
            continue
        payload = json.loads(path.read_text(encoding='utf-8'))
        for case in payload['cases']:
            total += 1
            failures = run_case(cls, case)
            if failures:
                failed += 1
                print(f'FAIL {name}::{case["id"]}')
                for line in failures:
                    print(f'    {line}')

    passed = total - failed
    print(f'\n{passed}/{total} corpus cases passed ({failed} failed)')
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
