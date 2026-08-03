from __future__ import annotations

# built-in
from typing import Sequence

# app
from .. import _rust_adapter as _rust
from .base import Base as _Base, BaseSimilarity as _BaseSimilarity
from .types import SimFunc


__all__ = [
    'Prefix', 'Postfix', 'Length', 'Identity', 'Matrix',
    'prefix', 'postfix', 'length', 'identity', 'matrix',
]


class Prefix(_BaseSimilarity):
    """prefix similarity
    """

    def __init__(self, qval: int = 1, sim_test: SimFunc = None) -> None:
        self.qval = qval
        self.sim_test = sim_test or self._ident

    def __call__(self, *sequences: Sequence) -> Sequence:
        if self.sim_test is not self._ident:
            raise NotImplementedError(
                'Prefix with a custom sim_test is not supported by the Rust-backed port',
            )
        if not sequences:
            return ''
        return _rust.compute('prefix', self.__dict__, 'call', *sequences)

    def similarity(self, *sequences: Sequence) -> int:
        if self.sim_test is not self._ident:
            raise NotImplementedError(
                'Prefix with a custom sim_test is not supported by the Rust-backed port',
            )
        if not sequences:
            return 0
        return _rust.compute('prefix', self.__dict__, 'similarity', *sequences)


class Postfix(Prefix):
    """postfix similarity
    """

    def __call__(self, *sequences: Sequence) -> Sequence:
        if self.sim_test is not self._ident:
            raise NotImplementedError(
                'Postfix with a custom sim_test is not supported by the Rust-backed port',
            )
        if not sequences:
            return ''
        return _rust.compute('postfix', self.__dict__, 'call', *sequences)


class Length(_Base):
    """Length distance
    """

    def __call__(self, *sequences: Sequence) -> int:
        return _rust.compute('length', self.__dict__, 'call', *sequences)


class Identity(_BaseSimilarity):
    """Identity similarity
    """

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('identity', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> int:
        return _rust.compute('identity', self.__dict__, 'call', *sequences)


class Matrix(_BaseSimilarity):
    """Matrix similarity
    """

    def __init__(
        self,
        mat=None,
        mismatch_cost: int = 0,
        match_cost: int = 1,
        symmetric: bool = True,
        external: bool = True,
    ) -> None:
        self.mat = mat
        self.mismatch_cost = mismatch_cost
        self.match_cost = match_cost
        self.symmetric = symmetric

    def maximum(self, *sequences: Sequence) -> int:
        return _rust.compute('matrix', self.__dict__, 'maximum', *sequences)

    def __call__(self, *sequences: Sequence) -> int:
        if self.mat:
            raise NotImplementedError(
                'Matrix with a custom mat= lookup table is not supported by the Rust-backed port',
            )
        return _rust.compute('matrix', self.__dict__, 'call', *sequences)


prefix = Prefix()
postfix = Postfix()
length = Length()
identity = Identity()
matrix = Matrix()
