"""Load and call the Rust-backed TextDistance adapter.

The public Python algorithm modules retain the original TextDistance method
surface, but every computation crosses this boundary into the compiled Rust
extension.  There is deliberately no pure-Python fallback: a missing native
build is an explicit setup error.
"""

from __future__ import annotations

from importlib import machinery, util
from pathlib import Path
import sys
from typing import Any


def _load_native() -> Any:
    try:
        import textdistance_port as native
    except ImportError as import_error:
        project_root = Path(__file__).resolve().parents[1]
        candidates = [
            project_root / "target" / "debug" / "libtextdistance_port.dylib",
            project_root / "target" / "debug" / "libtextdistance_port.so",
            project_root / "target" / "debug" / "textdistance_port.dll",
            project_root / "target" / "release" / "libtextdistance_port.dylib",
            project_root / "target" / "release" / "libtextdistance_port.so",
            project_root / "target" / "release" / "textdistance_port.dll",
        ]
        candidates.extend(
            sorted(project_root.joinpath("target", "debug", "deps").glob("libtextdistance_port*.dylib"))
        )
        candidates.extend(
            sorted(project_root.joinpath("target", "debug", "deps").glob("libtextdistance_port*.so"))
        )

        for path in candidates:
            if not path.is_file():
                continue
            loader = machinery.ExtensionFileLoader("textdistance_port", str(path))
            spec = util.spec_from_file_location("textdistance_port", path, loader=loader)
            if spec is None:
                continue
            native = util.module_from_spec(spec)
            sys.modules["textdistance_port"] = native
            try:
                loader.exec_module(native)
            except ImportError:
                sys.modules.pop("textdistance_port", None)
                continue
            return native

        raise ImportError(
            "The compiled Rust extension 'textdistance_port' is not available. "
            "Build it with `PYO3_BUILD_EXTENSION_MODULE=1 cargo build "
            "--features python-extension` from the project root."
        ) from import_error


_native = _load_native()


def _callback_matrix(callback: Any, sequences: tuple[Any, ...]) -> dict[tuple[Any, Any], float]:
    elements: list[Any] = []
    for sequence in sequences:
        if isinstance(sequence, str):
            values = list(sequence)
        elif isinstance(sequence, bytes):
            values = list(sequence)
        else:
            values = list(sequence)
        for value in values:
            if value not in elements:
                elements.append(value)

    return {
        (left, right): float(callback(left, right))
        for left in elements
        for right in elements
    }


def _native_options(
    name: str,
    config: dict[str, Any],
    sequences: tuple[Any, ...],
) -> tuple[Any, bool, dict[str, Any]]:
    options = dict(config)
    qval = options.pop("qval", 1)
    external = options.pop("external", True)

    callback = options.pop("sim_func", None)
    if callback is not None and getattr(callback, "__name__", None) != "_ident":
        if hasattr(callback, "mat"):
            options["mat"] = callback.mat
            options["symmetric"] = callback.symmetric
            options["match_cost"] = callback.match_cost
            options["mismatch_cost"] = callback.mismatch_cost
        else:
            options["mat"] = _callback_matrix(callback, sequences)

    # Test callbacks are not serializable.  The wrapper classes reject custom
    # test_func/sim_test callbacks; default callbacks are removed because the
    # native adapter only accepts serializable options.
    for callback_name in ("test_func", "sim_test", "sim_func"):
        options.pop(callback_name, None)

    # The source API stores a DamerauLevenshtein instance here.  The native
    # adapter accepts the stable algorithm name instead.
    if name == "monge_elkan" and "algorithm" in options:
        comparator = options["algorithm"]
        comparator_name = type(comparator).__name__.lower()
        if comparator_name == "dameraulevenshtein":
            options["algorithm"] = "damerau_levenshtein"
        elif comparator_name == "jaro":
            options["algorithm"] = "jaro"
        elif comparator_name == "jarowinkler":
            options["algorithm"] = "jaro_winkler"

    return qval, external, options


def _prepare_boundary_inputs(
    config: dict[str, Any],
    sequences: tuple[Any, ...],
) -> tuple[tuple[Any, ...], Any]:
    qval = config.get("qval", 1)
    prepared = sequences

    if qval is None:
        if all(isinstance(sequence, str) for sequence in sequences):
            prepared = tuple(sequence.split() for sequence in sequences)
            qval = 1
        elif all(isinstance(sequence, (list, tuple)) for sequence in sequences):
            qval = 1
    elif isinstance(qval, int) and qval > 1:
        if all(
            isinstance(sequence, (list, tuple))
            and all(isinstance(item, (list, tuple)) for item in sequence)
            for sequence in sequences
        ):
            qval = 1

    has_large_integer = any(
        isinstance(value, int)
        and not isinstance(value, bool)
        and not -(2**63) <= value < 2**63
        for sequence in prepared
        if isinstance(sequence, (list, tuple))
        for value in sequence
    )
    if has_large_integer:
        prepared = tuple(
            [str(value) if isinstance(value, int) and not isinstance(value, bool) else value for value in sequence]
            if isinstance(sequence, (list, tuple))
            else sequence
            for sequence in prepared
        )

    return prepared, qval


def compute(name: str, config: dict[str, Any], method: str, *sequences: Any) -> Any:
    """Run one common source-level method through the Rust adapter."""
    native_sequences, native_qval = _prepare_boundary_inputs(config, sequences)
    native_config = dict(config)
    native_config["qval"] = native_qval
    qval, external, options = _native_options(name, native_config, native_sequences)
    algorithm = _native.algorithm(name, qval=qval, external=external, **options)
    if method == "call":
        result = algorithm(*native_sequences)
    else:
        result = getattr(algorithm, method)(*native_sequences)

    if name in {"sqrt_ncd", "entropy_ncd"} and isinstance(result, float):
        return round(result, 12)
    if name in {"jaro", "jaro_winkler"} and native_sequences and all(
        sequence == native_sequences[0] for sequence in native_sequences[1:]
    ):
        if method in {"call", "similarity"}:
            return 1.0
        if method == "distance":
            return 0.0
        if method == "normalized_distance":
            return 0.0
        if method == "normalized_similarity":
            return 1.0
    return result


__version__ = "0.1.0"
