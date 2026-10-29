import datetime
import hashlib
import json
from pathlib import Path
import subprocess

base = Path('R:/d2-evidence/combat-characterization-20261028')
source = Path('R:/d2-evidence/combat-reports-guarded-source-64cf0f1c5')
proof = base / 'root-r3-64cf-six-report-completion.json'
assert not proof.exists()
def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
drivers = {x['case']: x for x in read(base / 'root-r3-64cf-six-original-driver-exits.json')}
post_inputs = read(base / 'root-r3-64cf-six-post-inputs.json')
post_build = read(base / 'root-r3-64cf-six-post-build.json')
assert post_inputs['passed'] and len(post_inputs['results']) == 6
assert post_build['passed'] and len(post_build['files']) == 42
assert not subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip()
all_cases = {}
for name, driver in drivers.items():
    folder = base / 'out' / name
    run = read(folder / 'run.json')
    trace = folder / 'trace.jsonl'
    rows = [json.loads(line) for line in trace.read_text(encoding='utf-8').splitlines()]
    events = {}
    for row in rows:
        assert row['event'] not in events, 'Duplicate event'
        events[row['event']] = row
    assert driver['actual_original_driver_exit'] == 0 and run['actual_native_exit'] == 0 and run['status'] == 'completed'
    boot = events['boot']; census = events['object_census']; before = events['object_before']
    after = events['object_after']; summary = events['object_summary']; counts = events['object_aim_counts']
    assert boot['seed'] == 1 and boot['frame_ms'] == 100 and boot['warmup'] == 5 and boot['window_seconds'] == 12
    assert census['duplicate_identities'] == 0 and census['structures'] > 0
    assert sum(census['n_' + kind] for kind in ['site', 'site_updatable', 'city_building', 'pylon', 'scenic', 'segment']) == census['structures']
    assert before['alive'] and after['present'] and before['host_id'] == after['host_id']
    assert summary['done'] and summary['structures_before'] == summary['structures_after'] == census['structures']
    assert counts['asked_identify'] == census['structures'], 'Second enumeration must reuse issued identities'
    assert run['build_info_sha256'] == post_build['manifest_sha256']
    if boot['report'] == 'none':
        assert 'object_receipt' not in events and not summary['changed'] and after['alive']
        assert after['damage_level'] == before['damage_level']
    elif boot['report'] == 'damage':
        assert summary['changed'] and after['alive'] and after['damage_level'] == before['damage_level'] // 2
        assert not events['object_receipt']['killed']
    else:
        assert boot['report'] == 'destroyed' and summary['changed'] and not after['alive']
        assert after['damage_level'] == before['damage_level'] and events['object_receipt']['killed']
        assert after['alive_' + boot['kind']] < census['alive_' + boot['kind']]
    if boot['identity'] == 'own':
        assert before['host_id'].startswith('own-world/structure/')
    else:
        assert before['host_id'].startswith('eech-world:object:')
    if boot['control'] != 'none':
        control = events['object_control']
        expected = {'raise': 'cannot be repaired by a report', 'duplicate': 'already destroyed: a fact is applied once', 'unknown': 'was never issued'}[boot['control']]
        assert control['refused'] and expected in control['error']
        assert 'poison' not in control['error'].lower(), 'Secondary poisoned-state error is insufficient'
    all_cases[name] = dict(driver=driver, run=run, trace=str(trace), trace_bytes=trace.stat().st_size,
                           trace_sha256=digest(trace), rows=rows, passed=True)

def event(case, name):
    return next(x for x in all_cases['object-report-' + case + '-02']['rows'] if x['event'] == name)
assert event('site-none', 'object_before') == event('site-damage-raise', 'object_before') == event('site-destroy-duplicate', 'object_before')
assert event('segment-none', 'object_before') == event('segment-destroy', 'object_before')
segment_before = event('segment-none', 'object_census')['alive_segment']
assert event('segment-none', 'object_after')['alive_segment'] == segment_before
segment_after = event('segment-destroy', 'object_after')['alive_segment']
assert segment_after < segment_before
counts = event('segment-destroy', 'object_aim_counts')
assert counts['asked_bridge_kill'] > 0 and counts['asked_bridge_neighbour'] > 0 and counts['asked_bridge_fall'] > 0
assert not any(k.startswith('asked_bridge_') for k in event('segment-none', 'object_aim_counts'))
record = dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    status='PASS within six bounded shipping report/withhold/refusal cases; independent runtime review pending',
    runtime_source='64cf0f1c5551a2e59309a92917c32e6189d8ae60', source=str(source), source_clean=True,
    candidate_test_only_sha='a89a668ae2fa5594a93fb6bbccf46e1c9b7bff38',
    build_info_sha256=post_build['manifest_sha256'], all42_exports_match=True, all6_post_retail_manifests_match=True,
    scenarios=all_cases, bridge_alive_before=segment_before, bridge_alive_after_report=segment_after,
    actual_state_result='Withhold preserves site/segment state; damage halves actual site health; destruction changes actual alive state, with health preserved; segment destruction reaches real bridge handlers and kills a dependent segment. Own-issued structure identity accepted; all three specific primary guards refuse.',
    limitations=['These are controlled host-reported facts, not weapon-hit reports or all-six-kind mutation coverage.',
      'Own case replaces identify only; it is not an entirely independent combat/physics host.',
      'Bridge question counts cover the complete run, not report-only campaign policy counters; route/repair policy remains supported by source chain and earlier closed bridge witnesses.',
      'No Tacview generated in these focused API cases. Final ordinary40/all8/full recording comparison remains pending.',
      'Old six01 f470 access violations remain failures, retained separately; no accepted baseline or tolerance changed.'])
proof.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(dict(proof=str(proof), cases=len(all_cases), passed=True, bridge_alive_before=segment_before, bridge_alive_after=segment_after)))
