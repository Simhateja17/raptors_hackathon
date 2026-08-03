# Adapter issues resolved during Lane 4 integration

The initial corpus review recorded two adapter mismatches. Both are fixed in
the current working tree and have direct regression probes.

## Prefix/Postfix output for non-default q-values

The adapter now derives sequence output from the prepared Rust elements. Word
q-values return a Python list of strings, and n-gram q-values return a Python
list of tuples, matching the original package. For example:

```pycon
>>> td.Prefix(qval=None)('one two three', 'one two four')
['one', 'two']
>>> td.Prefix(qval=2)('testing', 'tester')
[('t', 'e'), ('e', 's'), ('s', 't')]
```

## Plain integer lists versus bytes

The adapter recognizes actual Python `bytes` objects as byte sequences and
keeps ordinary `list[int]` / `tuple[int]` inputs as integer sequences. For
example:

```pycon
>>> td.Prefix()([1, 2, 3, 4], [1, 2, 5, 6])
[1, 2]
```

These behaviors are also covered by the unchanged original tests and the
Lane 4 corpus/probe runs.
