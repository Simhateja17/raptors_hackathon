"""SIM-10/INT-01 boundary: load the compiled Rust core, never the Python one.

Every public algorithm class delegates its actual computation to the
compiled `textdistance_rust` extension (built from `python_adapter/` via
PyO3/maturin). This module intentionally has no pure-Python fallback: if the
extension is not built, importing `textdistance` fails loudly instead of
silently running the original algorithm implementations.
"""

from __future__ import annotations

from typing import Any

try:
    import textdistance_rust as _native
except ImportError as _import_error:  # pragma: no cover - environment issue, not a code path
    raise ImportError(
        'The compiled Rust extension "textdistance_rust" is not installed. '
        'Build it with `python -m maturin build --release` in python_adapter/ '
        'and install the resulting wheel (or `maturin develop` inside a virtualenv). '
        'This package is Rust-backed and does not fall back to a pure-Python '
        'implementation.'
    ) from _import_error


def compute(name: str, config: dict, method: str, *sequences: Any) -> Any:
    """Run one common method of a Rust-backed algorithm.

    ``config`` is normally the calling instance's ``__dict__``; unrecognized
    keys are ignored by the extension. ``sequences`` are passed through
    exactly as received from the public API caller.
    """
    return _native.compute(name, config, method, list(sequences))


__version__ = _native.version()
