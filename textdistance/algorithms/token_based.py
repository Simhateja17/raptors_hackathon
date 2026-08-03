from __future__ import annotations

# built-in
from itertools import repeat
from typing import Sequence

# app
from .. import _rust_adapter as _rust
from .base import Base as _Base, BaseSimilarity as _BaseSimilarity
from .edit_based import DamerauLevenshtein


__all__ = [
    'Jaccard', 'Sorensen', 'Tversky',
    'Overlap', 'Cosine', 'Tanimoto', 'MongeElkan', 'Bag',

    'jaccard', 'sorensen', 'tversky', 'sorensen_dice', 'dice',
    'overlap', 'cosine', 'tanimoto', 'monge_elkan', 'bag',
]


class Jaccard(_BaseSimilarity):
    """
    Compute the Jaccard similarity between the two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 1, where 1 means equal,
    and 0 totally different.

    https://en.wikipedia.org/wiki/Jaccard_index
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/jaccard.js
    """

    def __init__(
        self,
        qval: int = 1,
        as_set: bool = False,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.as_set = as_set
        self.external = external

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('jaccard', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('jaccard', self.__dict__, 'call', *sequences)


class Sorensen(_BaseSimilarity):
    """
    Compute the Sorensen distance between the two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 1, where 1 means equal,
    and 0 totally different.

    https://en.wikipedia.org/wiki/S%C3%B8rensen%E2%80%93Dice_coefficient
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/dice.js
    """

    def __init__(self, qval: int = 1, as_set: bool = False, external: bool = True) -> None:
        self.qval = qval
        self.as_set = as_set
        self.external = external

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('sorensen', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('sorensen', self.__dict__, 'call', *sequences)


class Tversky(_BaseSimilarity):
    """
    Compute the Tversky index for two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 1, where 1 means equal,
    and 0 totally different

    https://en.wikipedia.org/wiki/Tversky_index
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/tversky.js
    """

    def __init__(
        self,
        qval: int = 1,
        ks: Sequence[float] = None,
        bias: float | None = None,
        as_set: bool = False,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.ks = ks or repeat(1)
        self.bias = bias
        self.as_set = as_set
        self.external = external

    def _rust_config(self) -> dict:
        # `self.ks` is either the source's infinite `repeat(1)` sentinel
        # (meaning "no custom coefficients") or a concrete sequence the
        # caller supplied; only the latter can cross the Rust boundary.
        config = dict(self.__dict__)
        if isinstance(self.ks, repeat):
            config.pop('ks', None)
        else:
            config['ks'] = list(self.ks)
        return config

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('tversky', self._rust_config(), 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('tversky', self._rust_config(), 'call', *sequences)


class Overlap(_BaseSimilarity):
    """
    Compute the Overlap coefficient for two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 1, where 1 means equal,
    and 0 totally different

    https://en.wikipedia.org/wiki/Overlap_coefficient
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/overlap.js
    """

    def __init__(
        self,
        qval: int = 1,
        as_set: bool = False,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.as_set = as_set
        self.external = external

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('overlap', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('overlap', self.__dict__, 'call', *sequences)


class Cosine(_BaseSimilarity):
    """
    Compute the Cosine similarity (Ochiai coefficient) for two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 1, where 1 means equal,
    and 0 totally different

    https://en.wikipedia.org/wiki/Cosine_similarity
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/cosine.js
    """

    def __init__(
        self,
        qval: int = 1,
        as_set: bool = False,
        external: bool = True,
    ) -> None:
        self.qval = qval
        self.as_set = as_set
        self.external = external

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('cosine', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('cosine', self.__dict__, 'call', *sequences)


class Tanimoto(Jaccard):
    """
    Compute the Tanimoto distance between two sequences.
    They should contain hashable items.
    The return value is a float between -inf and 0, where 0 means equal,
    and -inf totally different

    This is identical to the Jaccard similarity coefficient
    and the Tversky index for alpha=1 and beta=1.

    https://en.wikipedia.org/wiki/Jaccard_index#Tanimoto_similarity_and_distance
    """

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('tanimoto', self.__dict__, 'call', *sequences)


class MongeElkan(_BaseSimilarity):
    """
    Compute the Monge Elkan distance between two sequences.
    They should contain hashable items.
    The return value is a float between 0 and 2, where 2 means equal,
    and 0 totally different.

    https://www.academia.edu/200314/Generalized_Monge-Elkan_Method_for_Approximate_Text_String_Comparison
    http://www.cs.cmu.edu/~wcohen/postscript/kdd-2003-match-ws.pdf
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/monge-elkan.js
    """
    _damerau_levenshtein = DamerauLevenshtein()

    def __init__(
        self,
        algorithm=_damerau_levenshtein,
        symmetric: bool = False,
        qval: int = 1,
        external: bool = True,
    ) -> None:
        self.algorithm = algorithm
        self.symmetric = symmetric
        self.qval = qval
        self.external = external

    def _check_supported(self) -> None:
        default = type(self.algorithm).__name__ in {
            'DamerauLevenshtein',
            'Jaro',
            'JaroWinkler',
        }
        if not default:
            raise NotImplementedError(
                'MongeElkan supports the built-in Damerau-Levenshtein, Jaro, '
                'and Jaro-Winkler comparators in the Rust-backed port',
            )

    def maximum(self, *sequences: Sequence) -> float:
        self._check_supported()
        return _rust.compute('monge_elkan', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> float:
        self._check_supported()
        return _rust.compute('monge_elkan', self.__dict__, 'call', *sequences)


class Bag(_Base):
    """
    Compute the Bag distance between two sequences.
    They should contain hashable items.
    The return value is a float between 0 and N, where 0 means equal,
    and N totally different. N would, at most, be the length of the
    longest sequence in the comparison.

    http://www-db.disi.unibo.it/research/papers/SPIRE02.pdf
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/bag.js
    """

    def __call__(self, *sequences: Sequence) -> float:
        return _rust.compute('bag', self.__dict__, 'call', *sequences)


bag = Bag()
cosine = Cosine()
dice = Sorensen()
jaccard = Jaccard()
monge_elkan = MongeElkan()
overlap = Overlap()
sorensen = Sorensen()
sorensen_dice = Sorensen()
# sorensen_dice = Tversky(ks=[.5, .5])
tanimoto = Tanimoto()
tversky = Tversky()
