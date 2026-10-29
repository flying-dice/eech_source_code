"""Recompute a bounded native frame-cost recording from its raw samples.

This audits timing evidence only. Native/controller exits, artifact equality,
input/build/source identity and background load remain separate evidence.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('non-finite JSON constant: ' + value)


def number(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def same(actual, expected, name):
    require(number(actual) and math.isclose(actual, expected, rel_tol=5.1e-9, abs_tol=1e-9), name + ' differs from raw samples')


def audit(path, expected_frames):
    require(type(expected_frames) is int and 0 < expected_frames <= 200000, 'invalid expected frame count')
    require(path.stat().st_size <= 32 * 1024 * 1024, 'timing evidence exceeds bound')
    samples, window, windows = [], [], []
    header = complete = None
    awaiting_window = False
    with path.open(encoding='utf-8') as stream:
        for line_number, line in enumerate(stream, 1):
            require(line_number <= 400010 and len(line) < 8192, 'record bound exceeded')
            row = json.loads(line, object_pairs_hook=unique_object, parse_constant=reject_constant)
            require(isinstance(row, dict), 'record must be an object')
            require(complete is None, 'record after completion')
            kind = row.get('kind')
            if header is None:
                require(line_number == 1 and kind == 'header', 'missing initial header')
                require(row.get('timer') == 'os.clock' and row.get('unit') == 'ms', 'unexpected timer or unit')
                require(type(row.get('window_frames')) is int and row['window_frames'] == 600, 'unexpected window size')
                require(row.get('raw_frame_samples') is True, 'raw samples not declared')
                header = row
            elif kind == 'frame':
                require(not awaiting_window, 'missing full-window record')
                require(type(row.get('frame')) is int and row['frame'] == len(samples) + 1, 'noncontiguous frame sequence')
                require(len(samples) < expected_frames and number(row.get('elapsed_ms')), 'invalid or excessive sample')
                samples.append(row['elapsed_ms'])
                window.append(row['elapsed_ms'])
                awaiting_window = len(window) == 600
            elif kind == 'window':
                require(bool(window) and (len(window) == 600 or len(samples) == expected_frames), 'unexpected partial window')
                require(type(row.get('through_frame')) is int and row['through_frame'] == len(samples), 'wrong window frontier')
                require(type(row.get('frames')) is int and row['frames'] == len(window), 'wrong window size')
                same(row.get('mean_ms'), sum(window) / len(window), 'window mean')
                same(row.get('max_ms'), max(window), 'window maximum')
                windows.append(row)
                window, awaiting_window = [], False
            elif kind == 'complete':
                require(len(samples) == expected_frames and not window and not awaiting_window, 'incomplete samples/windows')
                require(type(row.get('frames')) is int and row['frames'] == expected_frames, 'wrong final frame count')
                ordered = sorted(samples)
                calculated = {'mean_ms': sum(samples) / len(samples), 'max_ms': ordered[-1]}
                for name, fraction in (('p50_ms', .5), ('p95_ms', .95), ('p99_ms', .99)):
                    calculated[name] = ordered[max(1, math.ceil(fraction * len(samples))) - 1]
                for name, value in calculated.items():
                    same(row.get(name), value, name)
                complete = row
            else:
                raise ValueError('unexpected record kind: ' + str(kind))
    require(complete is not None, 'missing normal-completion record')
    return {'passed': True, 'frames': len(samples), 'windows': len(windows), 'recomputed': calculated,
            'samples_file': str(path), 'samples_bytes': path.stat().st_size,
            'samples_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'limits': 'Elapsed native engine:frame only, quantized os.clock under recorded load; no DCS callback/scheduling budget or causal benchmark. Original native/controller completion and default artifact/input/build/source checks required separately.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('samples', type=Path)
    parser.add_argument('--expected-frames', type=int, default=9000)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    require(not args.out.exists(), 'refusing to overwrite an audit')
    result = audit(args.samples, args.expected_frames)
    args.out.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8', newline='\n')
    print(json.dumps(result))
