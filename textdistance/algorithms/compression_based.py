from __future__ import annotations

import math
from collections import Counter
from fractions import Fraction

# app
from .. import _rust_adapter as _rust
from .base import Base as _Base


__all__ = [
    'ArithNCD', 'LZMANCD', 'BZ2NCD', 'RLENCD', 'BWTRLENCD', 'ZLIBNCD',
    'SqrtNCD', 'EntropyNCD',

    'bz2_ncd', 'lzma_ncd', 'arith_ncd', 'rle_ncd', 'bwtrle_ncd', 'zlib_ncd',
    'sqrt_ncd', 'entropy_ncd',
]


class _NCDBase(_Base):
    """Normalized compression distance (NCD)

    https://articles.orsinium.dev/other/ncd/
    https://en.wikipedia.org/wiki/Normalized_compression_distance#Normalized_compression_distance
    """
    qval = 1
    _rust_name = ''

    def __init__(self, qval: int = 1) -> None:
        self.qval = qval

    def maximum(self, *sequences) -> int:
        return 1

    def __call__(self, *sequences) -> float:
        return _rust.compute(self._rust_name, self.__dict__, 'call', *sequences)


class _BinaryNCDBase(_NCDBase):

    def __init__(self) -> None:
        pass


class ArithNCD(_NCDBase):
    """Arithmetic coding

    https://github.com/gw-c/arith
    http://www.drdobbs.com/cpp/data-compression-with-arithmetic-encodin/240169251
    https://en.wikipedia.org/wiki/Arithmetic_coding
    """
    _rust_name = 'arith_ncd'

    def __init__(self, base: int = 2, terminator: str | None = None, qval: int = 1) -> None:
        self.base = base
        self.terminator = terminator
        self.qval = qval

    # These private helpers remain part of the original compatibility surface.
    # Public calls still go through the Rust adapter below.
    def _make_probs(self, *sequences):
        sequences = self._get_counters(*sequences)
        counts = self._sum_counters(*sequences)
        if self.terminator is not None:
            counts[self.terminator] = 1
        total_letters = sum(counts.values())

        probabilities = {}
        cumulative_count = 0
        for char, current_count in counts.most_common():
            probabilities[char] = (
                Fraction(cumulative_count, total_letters),
                Fraction(current_count, total_letters),
            )
            cumulative_count += current_count
        return probabilities

    def _get_range(self, data, probs):
        if self.terminator is not None:
            if self.terminator in data:
                data = data.replace(self.terminator, '')
            data += self.terminator

        start = Fraction(0, 1)
        width = Fraction(1, 1)
        for char in data:
            probability_start, probability_width = probs[char]
            start += probability_start * width
            width *= probability_width
        return start, start + width

    def _compress(self, data):
        probabilities = self._make_probs(data)
        start, end = self._get_range(data=data, probs=probabilities)
        output_fraction = Fraction(0, 1)
        output_denominator = 1
        while not (start <= output_fraction < end):
            output_numerator = 1 + ((start.numerator * output_denominator) // start.denominator)
            output_fraction = Fraction(output_numerator, output_denominator)
            output_denominator *= 2
        return output_fraction

    def _get_size(self, data):
        numerator = self._compress(data).numerator
        if numerator == 0:
            return 0
        return math.ceil(math.log(numerator, self.base))


class RLENCD(_NCDBase):
    """Run-length encoding

    https://en.wikipedia.org/wiki/Run-length_encoding
    """
    _rust_name = 'rle_ncd'


class BWTRLENCD(RLENCD):
    """
    https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform
    https://en.wikipedia.org/wiki/Run-length_encoding
    """
    _rust_name = 'bwtrle_ncd'

    def __init__(self, terminator: str = '\0') -> None:
        self.terminator = terminator


# -- NORMAL COMPRESSORS -- #


class SqrtNCD(_NCDBase):
    """Square Root based NCD

    Size of compressed data equals to sum of square roots of counts of every
    element in the input sequence.
    """
    _rust_name = 'sqrt_ncd'

    def __init__(self, qval: int = 1) -> None:
        self.qval = qval

    def _compress(self, data):
        return {element: math.sqrt(count) for element, count in Counter(data).items()}

    def _get_size(self, data):
        return sum(self._compress(data).values())


class EntropyNCD(_NCDBase):
    """Entropy based NCD

    Get Entropy of input sequence as a size of compressed data.

    https://en.wikipedia.org/wiki/Entropy_(information_theory)
    https://en.wikipedia.org/wiki/Entropy_encoding
    """
    _rust_name = 'entropy_ncd'

    def __init__(self, qval: int = 1, coef: int = 1, base: int = 2) -> None:
        self.qval = qval
        self.coef = coef
        self.base = base

    def _compress(self, data):
        total_count = len(data)
        entropy = 0.0
        for element_count in Counter(data).values():
            probability = element_count / total_count
            entropy -= probability * math.log(probability, self.base)
        assert entropy >= 0
        return entropy

    def _get_size(self, data):
        return self.coef + self._compress(data)


# -- BINARY COMPRESSORS -- #


class BZ2NCD(_BinaryNCDBase):
    """
    https://en.wikipedia.org/wiki/Bzip2
    """
    _rust_name = 'bz2_ncd'


class LZMANCD(_BinaryNCDBase):
    """
    https://en.wikipedia.org/wiki/LZMA
    """
    _rust_name = 'lzma_ncd'


class ZLIBNCD(_BinaryNCDBase):
    """
    https://en.wikipedia.org/wiki/Zlib
    """
    _rust_name = 'zlib_ncd'


arith_ncd = ArithNCD()
bwtrle_ncd = BWTRLENCD()
bz2_ncd = BZ2NCD()
lzma_ncd = LZMANCD()
rle_ncd = RLENCD()
zlib_ncd = ZLIBNCD()
sqrt_ncd = SqrtNCD()
entropy_ncd = EntropyNCD()
