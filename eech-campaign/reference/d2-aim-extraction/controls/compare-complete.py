#!/usr/bin/env python3
"""task-only (EEDCS-44 extraction): complete-trace comparison of a candidate against an original reference, recording each tool's ACTUAL exit status.

usage: compare-complete.py <candidate.jsonl> <reference.jsonl> <result.json> [--expect-equal | --expect-different]
Reads the files only. Records sizes, event counts, the raw SHA-256 and the LF-normalised SHA-256 of both, byte equality, line-by-line equality, and the exit status of
tools/aim-trace-compare.py (strict, every scope).
"""
import hashlib
import json
import subprocess
import sys

TOOL = r'R:\eech_source_code\eech-campaign\tools\aim-trace-compare.py'


def digest(path):
    data = open(path, 'rb').read()
    return {'bytes': len(data), 'sha256_raw': hashlib.sha256(data).hexdigest(), 'sha256_lf': hashlib.sha256(data.replace(b'\r\n', b'\n')).hexdigest(),
            'lines': data.count(b'\n')}


def main():
    candidate, reference, result = sys.argv[1:4]
    expect = sys.argv[4] if len(sys.argv) > 4 else None
    a, b = digest(candidate), digest(reference)
    done = subprocess.run([sys.executable, TOOL, candidate, reference], capture_output=True, text=True, encoding='utf-8')
    la = open(candidate, 'rb').read().replace(b'\r\n', b'\n').split(b'\n')
    lb = open(reference, 'rb').read().replace(b'\r\n', b'\n').split(b'\n')
    differing = [i + 1 for i, (x, y) in enumerate(zip(la, lb)) if x != y]
    record = {
        'candidate': candidate, 'reference': reference, 'candidate_digest': a, 'reference_digest': b,
        'raw_bytes_equal': a['sha256_raw'] == b['sha256_raw'], 'lf_bytes_equal': a['sha256_lf'] == b['sha256_lf'],
        'same_line_count': len(la) == len(lb), 'differing_lines_first20': differing[:20], 'differing_line_count': len(differing) + abs(len(la) - len(lb)),
        'comparator_exit': done.returncode, 'comparator_stdout': done.stdout.strip()[-800:], 'comparator_stderr': done.stderr.strip()[-800:],
    }
    json.dump(record, open(result, 'w', encoding='utf-8'), indent=1)
    print(json.dumps({k: record[k] for k in ('raw_bytes_equal', 'lf_bytes_equal', 'differing_line_count', 'comparator_exit')}))
    equal = record['raw_bytes_equal'] and record['comparator_exit'] == 0
    if expect == '--expect-equal':
        sys.exit(0 if equal else 1)
    if expect == '--expect-different':
        sys.exit(0 if (not record['lf_bytes_equal']) and record['comparator_exit'] == 1 else 1)


main()
