from __future__ import annotations


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
