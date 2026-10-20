#!/usr/bin/env python3
"""Compares two JSON files (regression-pack report.json, chains.json, loss-chain.json) value by value.

Every value is addressed by its key path, written as a JSON list so a key that contains a dot
cannot be confused with nesting. A key present on one side only, a type change (1, 1.0, true and
"1" are all different) and an empty container are all reported. Numbers with a fraction or an
exponent are compared as written, so 1e400 and 2e400, or 0.1 and 0.10000000000000000001, differ.
A file with a duplicated key is refused. Values are printed in full.

Exit 0 when the files are equal, 1 when they differ, 2 when a file cannot be read as JSON.

Usage: python -I compare-report.py <a.json> <b.json>
"""
import json
import sys

USAGE = 'Usage: python -I compare-report.py <a.json> <b.json>'
ABSENT = ('absent', '')


class Literal(str):
    """A JSON number with a fraction or exponent (or NaN, Infinity), kept as written."""


def unique_keys(pairs):
    seen, duplicates = set(), set()
    for k, _ in pairs:
        if k in seen:
            duplicates.add(k)
        else:
            seen.add(k)
    if duplicates:
        raise ValueError(f'duplicated key(s) {sorted(duplicates)}')
    return dict(pairs)


def load(path):
    with open(path, encoding='utf-8') as f:
        return json.load(f, object_pairs_hook=unique_keys, parse_float=Literal, parse_constant=Literal)


def leaves(value, path=()):
    """Yields (path, (type, value as JSON)) for every scalar and every empty container."""
    if isinstance(value, dict) and value:
        for k, v in value.items():
            yield from leaves(v, path + (k,))
    elif isinstance(value, list) and value:
        for i, v in enumerate(value):
            yield from leaves(v, path + (i,))
    elif isinstance(value, Literal):
        yield path, ('float', str(value))
    else:
        yield path, (type(value).__name__, json.dumps(value, ensure_ascii=False))


def differences(a, b):
    la, lb = dict(leaves(a)), dict(leaves(b))
    return [(p, la.get(p, ABSENT), lb.get(p, ABSENT)) for p in sorted(la.keys() | lb.keys(), key=repr)
            if la.get(p, ABSENT) != lb.get(p, ABSENT)]


def main(argv):
    sys.stdout.reconfigure(encoding='utf-8', errors='backslashreplace')
    if len(argv) != 3:
        print(USAGE, file=sys.stderr)
        return 2
    try:
        a, b = load(argv[1]), load(argv[2])
        diff = differences(a, b)
    except (OSError, ValueError, RecursionError) as e:
        print(f'UNREADABLE: {e}')
        return 2
    print(f'{len(diff)} differing values')
    for path, (ta, va), (tb, vb) in diff:
        print(f'  {json.dumps(list(path), ensure_ascii=False)}')
        print(f'    a: {ta} {va}')
        print(f'    b: {tb} {vb}')
    return 1 if diff else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
