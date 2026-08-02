from __future__ import annotations

# built-in
from typing import Sequence, TypeVar

# app
from .. import _rust_adapter as _rust
from .base import Base as _Base, BaseSimilarity as _BaseSimilarity


__all__ = [
    'MRA', 'Editex',
    'mra', 'editex',
]
T = TypeVar('T')


class MRA(_BaseSimilarity):
    """Western Airlines Surname Match Rating Algorithm comparison rating
    https://en.wikipedia.org/wiki/Match_rating_approach
    https://github.com/Yomguithereal/talisman/blob/master/src/metrics/mra.js
    """

    def maximum(self, *sequences: str) -> int:
        return _rust.compute('mra', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: str) -> int:
        return _rust.compute('mra', self.__dict__, 'call', *sequences)


class Editex(_Base):
    """
    https://anhaidgroup.github.io/py_stringmatching/v0.3.x/Editex.html
    http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.14.3856&rep=rep1&type=pdf
    http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.18.2138&rep=rep1&type=pdf
    https://github.com/chrislit/blob/master/abydos/distance/_editex.py
    https://habr.com/ru/post/331174/ (RUS)
    """
    groups: tuple[frozenset[str], ...] = (
        frozenset('AEIOUY'),
        frozenset('BP'),
        frozenset('CKQ'),
        frozenset('DT'),
        frozenset('LR'),
        frozenset('MN'),
        frozenset('GJ'),
        frozenset('FPV'),
        frozenset('SXZ'),
        frozenset('CSZ'),
    )
    ungrouped = frozenset('HW')  # all letters in alphabet that not presented in `grouped`

    def __init__(
        self,
        local: bool = False,
        match_cost: int = 0,
        group_cost: int = 1,
        mismatch_cost: int = 2,
        groups: tuple[frozenset[str], ...] = None,
        ungrouped: frozenset[str] = None,
        external: bool = True,
    ) -> None:
        # Ensure that match_cost <= group_cost <= mismatch_cost
        self.match_cost = match_cost
        self.group_cost = max(group_cost, self.match_cost)
        self.mismatch_cost = max(mismatch_cost, self.group_cost)
        self.local = local
        self.external = external

        if groups is not None:
            if ungrouped is None:
                raise ValueError('`ungrouped` argument required with `groups`')
            self.groups = groups
            self.ungrouped = ungrouped
        self.grouped = frozenset.union(*self.groups)

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('editex', self.__dict__, 'maximum', *sequences)

    def __call__(self, s1: str, s2: str) -> float:
        if 'groups' in self.__dict__ or 'ungrouped' in self.__dict__:
            raise NotImplementedError(
                'Editex with custom groups/ungrouped is not supported by the Rust-backed port',
            )
        return _rust.compute('editex', self.__dict__, 'call', s1, s2)


mra = MRA()
editex = Editex()
