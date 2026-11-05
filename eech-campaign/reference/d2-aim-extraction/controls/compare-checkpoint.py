#!/usr/bin/env python3
"""task-only (EEDCS-44 extraction): compares the run-wide report of a lua/aim-provider.lua trace (scope `run`) with the report an original
lua/aim-characterization.lua trace took at a checkpoint (scope `checkpoint`, `at` seconds), field by field (`scope` and `at` aside).

usage: compare-checkpoint.py <provider trace> <characterization trace> <at> <result.json> [--expect-equal | --expect-different]
Records each report line that differs and a summary of the counters and hashes that moved. Reads the files only.
"""
import json
import sys


def load(path):
    return [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]


def report(events, scope, at=None):
    kept = []
    for e in events:
        if e.get('scope') == scope and (at is None or e.get('at') == at):
            f = {k: v for k, v in e.items() if k not in ('scope', 'at')}
            kept.append(f)
    return kept


def main():
    provider, characterization, at, result = sys.argv[1], sys.argv[2], float(sys.argv[3]), sys.argv[4]
    expect = sys.argv[5] if len(sys.argv) > 5 else None
    a = report(load(provider), 'run')
    b = report(load(characterization), 'checkpoint', at if at != int(at) else int(at))
    differences = []
    for i in range(max(len(a), len(b))):
        x = a[i] if i < len(a) else None
        y = b[i] if i < len(b) else None
        if x != y:
            moved = {}
            if x and y:
                for k in set(x) | set(y):
                    if x.get(k) != y.get(k):
                        moved[k] = [x.get(k), y.get(k)]
            differences.append({'line': i + 1, 'event': (x or y).get('event'), 'key': (x or y).get('key') or (x or y).get('name'), 'moved': moved if moved else {'whole': [x, y]}})
    hashes = {}
    for label, events in (('provider', a), ('reference', b)):
        for e in events:
            if e.get('event') == 'aim-hash':
                hashes[label] = {'physical': e.get('physical'), 'decision': e.get('decision')}
    record = {'provider_trace': provider, 'characterization_trace': characterization, 'checkpoint_at': at, 'provider_report_lines': len(a),
              'reference_report_lines': len(b), 'differing_lines': len(differences), 'hashes': hashes, 'differences_first30': differences[:30]}
    json.dump(record, open(result, 'w', encoding='utf-8'), indent=1)
    print(json.dumps({'provider_lines': len(a), 'reference_lines': len(b), 'differing_lines': len(differences), 'hashes': hashes}))
    if not a or not b:
        sys.exit(2)
    if expect == '--expect-equal':
        sys.exit(0 if not differences else 1)
    if expect == '--expect-different':
        sys.exit(0 if differences else 1)


main()
