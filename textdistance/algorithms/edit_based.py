from __future__ import annotations

# built-in
from typing import Sequence, TypeVar

# app
from .. import _rust_adapter as _rust
from .base import Base as _Base, BaseSimilarity as _BaseSimilarity
from .types import SimFunc, TestFunc


__all__ = [
    'Hamming', 'MLIPNS',
    'Levenshtein', 'DamerauLevenshtein',
    'Jaro', 'JaroWinkler', 'StrCmp95',
    'NeedlemanWunsch', 'Gotoh', 'SmithWaterman',

    'hamming', 'mlipns',
    'levenshtein', 'damerau_levenshtein',
    'jaro', 'jaro_winkler', 'strcmp95',
    'needleman_wunsch', 'gotoh', 'smith_waterman',
]
T = TypeVar('T')


class Hamming(_Base):
    """
    Compute the Hamming distance between the two or more sequences.
    The Hamming distance is the number of differing items in ordered sequences.

    https://en.wikipedia.org/wiki/Hamming_distance
    """

    def __init__(
        self,
        qval: int = 1,
        test_func: TestFunc | None = None,
        truncate: bool = False,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.test_func = test_func or self._ident
        self.truncate = truncate
        self.external = external

    def __call__(self, *sequences: Sequence[object]) -> int:
        if self.test_func is not self._ident:
            raise NotImplementedError(
                'Hamming with a custom test_func is not supported by the Rust-backed port',
            )
        return _rust.compute('hamming', self.__dict__, 'call', *sequences)


class Levenshtein(_Base):
    """
    Compute the absolute Levenshtein distance between the two sequences.
    The Levenshtein distance is the minimum number of edit operations necessary
    for transforming one sequence into the other. The edit operations allowed are:

        * deletion:     ABC -> BC, AC, AB
        * insertion:    ABC -> ABCD, EABC, AEBC..
        * substitution: ABC -> ABE, ADC, FBC..

    https://en.wikipedia.org/wiki/Levenshtein_distance
    TODO: https://gist.github.com/kylebgorman/1081951/9b38b7743a3cb5167ab2c6608ac8eea7fc629dca
    """

    def __init__(
        self,
        qval: int = 1,
        test_func: TestFunc | None = None,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.test_func = test_func or self._ident
        self.external = external

    def __call__(self, s1: Sequence[T], s2: Sequence[T]) -> int:
        if self.test_func is not self._ident:
            raise NotImplementedError(
                'Levenshtein with a custom test_func is not supported by the Rust-backed port',
            )
        return _rust.compute('levenshtein', self.__dict__, 'call', s1, s2)


class DamerauLevenshtein(_Base):
    """
    Compute the absolute Damerau-Levenshtein distance between the two sequences.
    The Damerau-Levenshtein distance is the minimum number of edit operations necessary
    for transforming one sequence into the other. The edit operations allowed are:

        * deletion:      ABC -> BC, AC, AB
        * insertion:     ABC -> ABCD, EABC, AEBC..
        * substitution:  ABC -> ABE, ADC, FBC..
        * transposition: ABC -> ACB, BAC

    If `restricted=False`, it will calculate unrestricted distance,
    where the same character can be touched more than once.
    So the distance between BA and ACB is 2: BA -> AB -> ACB.

    https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
    """

    def __init__(
        self,
        qval: int = 1,
        test_func: TestFunc | None = None,
        external: bool = True,
        restricted: bool = True,
    ) -> None:
        self.qval = qval
        self.test_func = test_func or self._ident
        self.external = external
        self.restricted = restricted

    def __call__(self, s1: Sequence[T], s2: Sequence[T]) -> int:
        if self.test_func is not self._ident:
            raise NotImplementedError(
                'DamerauLevenshtein with a custom test_func is not supported by '
                'the Rust-backed port',
            )
        return _rust.compute('damerau_levenshtein', self.__dict__, 'call', s1, s2)

    def _pure_python_restricted(self, s1: Sequence[T], s2: Sequence[T]) -> int:
        config = dict(self.__dict__, restricted=True)
        return _rust.compute('damerau_levenshtein', config, 'call', s1, s2)

    def _pure_python_unrestricted(self, s1: Sequence[T], s2: Sequence[T]) -> int:
        config = dict(self.__dict__, restricted=False)
        return _rust.compute('damerau_levenshtein', config, 'call', s1, s2)


class JaroWinkler(_BaseSimilarity):
    """
    Computes the Jaro-Winkler measure between two strings.
    The Jaro-Winkler measure is designed to capture cases where two strings
    have a low Jaro score, but share a prefix.
    and thus are likely to match.

    https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/jaro.js
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/jaro-winkler.js
    """

    def __init__(
        self,
        long_tolerance: bool = False,
        winklerize: bool = True,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.long_tolerance = long_tolerance
        self.winklerize = winklerize
        self.external = external

    def maximum(self, *sequences: Sequence[object]) -> int:
        name = 'jaro_winkler' if self.winklerize else 'jaro'
        return _rust.compute(name, self.__dict__, 'maximum', *sequences)

    def __call__(self, s1: Sequence[T], s2: Sequence[T], prefix_weight: float = 0.1) -> float:
        name = 'jaro_winkler' if self.winklerize else 'jaro'
        config = dict(self.__dict__, prefix_weight=prefix_weight)
        return _rust.compute(name, config, 'call', s1, s2)


class Jaro(JaroWinkler):
    def __init__(
        self,
        long_tolerance: bool = False,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        super().__init__(
            long_tolerance=long_tolerance,
            winklerize=False,
            qval=qval,
            external=external,
        )


class NeedlemanWunsch(_BaseSimilarity):
    """
    Computes the Needleman-Wunsch measure between two strings.
    The Needleman-Wunsch generalizes the Levenshtein distance and considers global
    alignment between two strings. Specifically, it is computed by assigning
    a score to each alignment between two input strings and choosing the
    score of the best alignment, that is, the maximal score.
    An alignment between two strings is a set of correspondences between the
    characters of between them, allowing for gaps.

    https://en.wikipedia.org/wiki/Needleman%E2%80%93Wunsch_algorithm
    """

    def __init__(
        self,
        gap_cost: float = 1.0,
        sim_func: SimFunc = None,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.gap_cost = gap_cost
        if sim_func:
            self.sim_func = sim_func
        else:
            self.sim_func = self._ident
        self.external = external

    def minimum(self, *sequences: Sequence[object]) -> float:
        return -max(map(len, sequences)) * self.gap_cost

    def maximum(self, *sequences: Sequence[object]) -> float:
        return _rust.compute('needleman_wunsch', self.__dict__, 'maximum', *sequences)

    def distance(self, *sequences: Sequence[object]) -> float:
        """Get distance between sequences
        """
        return -1 * self.similarity(*sequences)

    def normalized_distance(self, *sequences: Sequence[object]) -> float:
        """Get distance from 0 to 1
        """
        minimum = self.minimum(*sequences)
        maximum = self.maximum(*sequences)
        if maximum == 0:
            return 0
        return (self.distance(*sequences) - minimum) / (maximum - minimum)

    def normalized_similarity(self, *sequences: Sequence[object]) -> float:
        """Get similarity from 0 to 1
        """
        minimum = self.minimum(*sequences)
        maximum = self.maximum(*sequences)
        if maximum == 0:
            return 1
        return (self.similarity(*sequences) - minimum) / (maximum * 2)

    def __call__(self, s1: Sequence[T], s2: Sequence[T]) -> float:
        return _rust.compute('needleman_wunsch', self.__dict__, 'call', s1, s2)


class SmithWaterman(_BaseSimilarity):
    """
    Computes the Smith-Waterman measure between two strings.
    The Smith-Waterman algorithm performs local sequence alignment;
    that is, for determining similar regions between two strings.
    Instead of looking at the total sequence, the Smith-Waterman algorithm compares
    segments of all possible lengths and optimizes the similarity measure.

    https://en.wikipedia.org/wiki/Smith%E2%80%93Waterman_algorithm
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/smith-waterman.js
    """

    def __init__(
        self,
        gap_cost: float = 1.0,
        sim_func: SimFunc = None,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.gap_cost = gap_cost
        self.sim_func = sim_func or self._ident
        self.external = external

    def maximum(self, *sequences: Sequence[object]) -> int:
        return _rust.compute('smith_waterman', self.__dict__, 'maximum', *sequences)

    def __call__(self, s1: Sequence[T], s2: Sequence[T]) -> float:
        return _rust.compute('smith_waterman', self.__dict__, 'call', s1, s2)


class Gotoh(NeedlemanWunsch):
    """Gotoh score
    Gotoh's algorithm is essentially Needleman-Wunsch with affine gap
    penalties:
    https://www.cs.umd.edu/class/spring2003/cmsc838t/papers/gotoh1982.pdf
    """

    def __init__(
        self,
        gap_open: int = 1,
        gap_ext: float = 0.4,
        sim_func: SimFunc = None,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.gap_open = gap_open
        self.gap_ext = gap_ext
        if sim_func:
            self.sim_func = sim_func
        else:
            self.sim_func = self._ident
        self.external = external

    def minimum(self, *sequences: Sequence[object]) -> int:
        return -min(map(len, sequences))

    def maximum(self, *sequences: Sequence[object]) -> int:
        return _rust.compute('gotoh', self.__dict__, 'maximum', *sequences)

    def __call__(self, s1: Sequence[T], s2: Sequence[T]) -> float:
        return _rust.compute('gotoh', self.__dict__, 'call', s1, s2)


class StrCmp95(_BaseSimilarity):
    """strcmp95 similarity

    http://cpansearch.perl.org/src/SCW/Text-JaroWinkler-0.1/strcmp95.c
    """
    sp_mx: tuple[tuple[str, str], ...] = (
        ('A', 'E'), ('A', 'I'), ('A', 'O'), ('A', 'U'), ('B', 'V'), ('E', 'I'),
        ('E', 'O'), ('E', 'U'), ('I', 'O'), ('I', 'U'), ('O', 'U'), ('I', 'Y'),
        ('E', 'Y'), ('C', 'G'), ('E', 'F'), ('W', 'U'), ('W', 'V'), ('X', 'K'),
        ('S', 'Z'), ('X', 'S'), ('Q', 'C'), ('U', 'V'), ('M', 'N'), ('L', 'I'),
        ('Q', 'O'), ('P', 'R'), ('I', 'J'), ('2', 'Z'), ('5', 'S'), ('8', 'B'),
        ('1', 'I'), ('1', 'L'), ('0', 'O'), ('0', 'Q'), ('C', 'K'), ('G', 'J'),
    )

    def __init__(self, long_strings: bool = False, external: bool = True) -> None:
        self.long_strings = long_strings
        self.external = external

    def maximum(self, *sequences: Sequence[object]) -> int:
        return _rust.compute('strcmp95', self.__dict__, 'maximum', *sequences)

    def __call__(self, s1: str, s2: str) -> float:
        return _rust.compute('strcmp95', self.__dict__, 'call', s1, s2)


class MLIPNS(_BaseSimilarity):
    """
    Compute the Hamming distance between the two or more sequences.
    The Hamming distance is the number of differing items in ordered sequences.

    http://www.sial.iias.spb.su/files/386-386-1-PB.pdf
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/mlipns.js
    """

    def __init__(
        self, threshold: float = 0.25,
        maxmismatches: int = 2,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.threshold = threshold
        self.maxmismatches = maxmismatches
        self.external = external

    def maximum(self, *sequences: Sequence[object]) -> int:
        return _rust.compute('mlipns', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence[object]) -> float:
        return _rust.compute('mlipns', self.__dict__, 'call', *sequences)


hamming = Hamming()
levenshtein = Levenshtein()
damerau = damerau_levenshtein = DamerauLevenshtein()
jaro = Jaro()
jaro_winkler = JaroWinkler()
needleman_wunsch = NeedlemanWunsch()
smith_waterman = SmithWaterman()
gotoh = Gotoh()
strcmp95 = StrCmp95()
mlipns = MLIPNS()
