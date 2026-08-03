from __future__ import annotations

# app
from .. import _rust_adapter as _rust
from .base import BaseSimilarity as _BaseSimilarity
from .types import TestFunc


__all__ = [
    'lcsseq', 'lcsstr', 'ratcliff_obershelp',
    'LCSSeq', 'LCSStr', 'RatcliffObershelp',
]


class LCSSeq(_BaseSimilarity):
    """longest common subsequence similarity

    https://en.wikipedia.org/wiki/Longest_common_subsequence_problem
    """

    def __init__(
        self,
        qval: int = 1,
        test_func: TestFunc = None,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.test_func = test_func or self._ident
        self.external = external

    def __call__(self, *sequences: str) -> str:
        if self.test_func is not self._ident:
            raise NotImplementedError(
                'LCSSeq with a custom test_func is not supported by the Rust-backed port',
            )
        if not sequences:
            return ''
        return _rust.compute('lcsseq', self.__dict__, 'call', *sequences)

    def similarity(self, *sequences) -> int:
        if self.test_func is not self._ident:
            raise NotImplementedError(
                'LCSSeq with a custom test_func is not supported by the Rust-backed port',
            )
        if not sequences:
            return 0
        return _rust.compute('lcsseq', self.__dict__, 'similarity', *sequences)


class LCSStr(_BaseSimilarity):
    """longest common substring similarity
    """

    def __call__(self, *sequences: str) -> str:
        if not sequences:
            return ''
        return _rust.compute('lcsstr', self.__dict__, 'call', *sequences)

    def similarity(self, *sequences: str) -> int:
        if not sequences:
            return 0
        return _rust.compute('lcsstr', self.__dict__, 'similarity', *sequences)


class RatcliffObershelp(_BaseSimilarity):
    """Ratcliff-Obershelp similarity
    This follows the Ratcliff-Obershelp algorithm to derive a similarity
    measure:
        1. Find the length of the longest common substring in sequences.
        2. Recurse on the strings to the left & right of each this substring
           in sequences. The base case is a 0 length common substring, in which
           case, return 0. Otherwise, return the sum of the current longest
           common substring and the left & right recursed sums.
        3. Multiply this length by 2 and divide by the sum of the lengths of
           sequences.

    https://en.wikipedia.org/wiki/Gestalt_Pattern_Matching
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/ratcliff-obershelp.js
    https://xlinux.nist.gov/dads/HTML/ratcliffObershelp.html
    """

    def maximum(self, *sequences: str) -> int:
        return _rust.compute('ratcliff_obershelp', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: str) -> float:
        return _rust.compute('ratcliff_obershelp', self.__dict__, 'call', *sequences)


lcsseq = LCSSeq()
lcsstr = LCSStr()
ratcliff_obershelp = RatcliffObershelp()
