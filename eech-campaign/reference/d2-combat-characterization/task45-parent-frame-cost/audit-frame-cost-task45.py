from pathlib import Path
import json, hashlib, subprocess, datetime

base = Path('R:/d2-evidence/combat-characterization-20261028')
out = base / 'out/frame-cost-task45-900-01'
script = base / 'frame-cost-task45-lua/campaign-frame-cost.lua'

def pairs(items):
    result = {}
    for k, v in items:
        if k in result:
            raise ValueError(f'duplicate {k}')
        result[k] = v
    return result

def parse(text):
    return json.loads(text, object_pairs_hook=pairs, parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x)))

def read(path):
    return parse(path.read_text(encoding='utf-8-sig'))

run = read(out / 'run.json')
assert run['status'] == 'completed' and run['actual_native_exit'] == 0
lines = [parse(x) for x in (out / 'frame-cost.jsonl').read_text().splitlines()]
assert lines[0]['kind'] == 'header' and lines[-1]['kind'] == 'complete'
windows, total, weighted = lines[1:-1], 0, 0
for w in windows:
    assert w['kind'] == 'window' and w['frames'] == 600
    total += w['frames']
    assert w['through_frame'] == total and 0 <= w['mean_ms'] <= w['max_ms']
    weighted += w['frames'] * w['mean_ms']
final = lines[-1]
assert total == 9000 == final['frames'] and len(windows) == 15
# This checks rounding of serialized profiler statistics, not campaign tolerances.
assert abs(weighted / total - final['mean_ms']) < .00001
assert 0 <= final['p50_ms'] <= final['p95_ms'] <= final['p99_ms'] <= final['max_ms']
assert final['max_ms'] == max(x['max_ms'] for x in windows)
comparison = read(base / 'root-frame-cost-task45-900-01-complete-comparison.json')
assert len(comparison['results']) == 3 and all(x['matches'] for x in comparison['results'])
post = (base / 'root-frame-cost-task45-900-01-post-inputs.log').read_text(encoding='utf8')
assert '1179 input files' in post and 'inputs IDENTICAL to ' in post
build = read(base / 'root-frame-cost-task45-900-01-post-build.json')
assert len(build['files']) == 35 and all(x['matches'] for x in build['files'])
assert hashlib.sha256(script.read_bytes()).hexdigest() == run['script_sha256']
provenance = read(script.with_suffix('.source.json'))
for name, sha in provenance['exact_companion_copies'].items():
    assert hashlib.sha256((script.parent / name).read_bytes()).hexdigest() == sha
source = Path('R:/d2-evidence/shipping-source-d2-weapon-a137fc3ec')
head = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
assert not subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True)
proof = {
    'when_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'role': 'Accepted Task45 parent native cost characterization, not changed baseline or final D2 acceptance',
    'actual_source': head, 'source_clean': True,
    'original_driver_exec': 7184, 'actual_original_driver_exit': 0,
    'run': run, 'complete_artifact_comparison': comparison,
    'post_input_manifest_exit': 0, 'post_build_manifest_exit': 0,
    'post_build_manifest_entries_verified': 35,
    'script_and_companions_unchanged': True,
    'timer_calibration': 'root-frame-cost-task45-clock-calibration.json: retained DLL clock advances during idle; no CPU-only inference',
    'frame_statistics': final, 'windows': windows,
    'statistics_audit': '15 contiguous 600-frame windows/9000 total; weighted mean and maximum coherent. Quantiles computed by the hashed source script. Raw per-frame samples were not exported: no independent quantile recomputation claimed.',
    'root_auditor_correction': 'Initial stdin helper expected MATCH in input tool output; actual successful tool says IDENTICAL. Checker assertion failed; no native/data/build rerun or modification. Current helper checks actual format.',
    'scope_limits': [
        'Only engine:frame plus nested host callbacks timed; objects/recording/metrics/outer-loop and DCS scheduling excluded',
        'Background Claude/Cargo/WSL activity: no matched benchmark or causal attribution',
        'Native 15 simulated minutes; not DCS N4 30min/x4 or a DCS frame budget',
        'Byte comparison not visual inspection; retained ab-base input provenance remains inferred',
        'Final ordinary shipping D2 measurement and whole-D2 acceptance pending',
    ],
    'independent_review': 'PENDING NEW review with R5 source/helper/runtime group; not yet closed',
}
path = base / 'root-frame-cost-task45-900-01-completion.json'
path.write_text(json.dumps(proof, indent=2) + '\n')
print(json.dumps({'proof': str(path), 'actual_exit': 0, 'all3fullbytes': True, 'post1179_all35': True, 'source': head, 'frames': total, 'statistics': final}))
