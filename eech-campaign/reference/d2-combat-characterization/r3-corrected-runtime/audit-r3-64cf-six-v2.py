"""Bounded audit of the six actual shipping R3 report cases; v1 retained separately."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess

BASE = Path('R:/d2-evidence/combat-characterization-20261028')
SOURCE = Path('R:/d2-evidence/combat-reports-guarded-source-64cf0f1c5')
CASES = {
    'site-none': ('site', 'none', 'none', 'reference'),
    'site-damage-raise': ('site', 'damage', 'raise', 'reference'),
    'site-destroy-duplicate': ('site', 'destroyed', 'duplicate', 'reference'),
    'site-own-destroy-unknown': ('site', 'destroyed', 'unknown', 'own'),
    'segment-none': ('segment', 'none', 'none', 'reference'),
    'segment-destroy': ('segment', 'destroyed', 'none', 'reference'),
}
EXPECTED = {'object-report-' + name + '-02': config for name, config in CASES.items()}
KINDS = ['site', 'site_updatable', 'city_building', 'pylon', 'scenic', 'segment']

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def driver_set(items):
    assert len(items) == len(EXPECTED), 'Exactly six driver records required'
    assert {x['case'] for x in items} == set(EXPECTED), 'Exact expected case set required'
    return {x['case']: x for x in items}

def validate_case(name, driver, run, rows, build_sha):
    kind, report, control, identity = EXPECTED[name]
    sequence = ['boot', 'object_census', 'object_before']
    if report != 'none':
        sequence.append('object_receipt')
    sequence += ['object_after', 'object_summary', 'object_aim_counts']
    if control != 'none':
        sequence.append('object_control')
    assert [x['event'] for x in rows] == sequence, 'Exact ordered event sequence required'
    events = {x['event']: x for x in rows}
    assert driver['case'] == name and driver['actual_original_driver_exit'] == 0
    assert run['actual_native_exit'] == 0 and run['status'] == 'completed'
    boot = events['boot']; census = events['object_census']; before = events['object_before']
    after = events['object_after']; summary = events['object_summary']; counts = events['object_aim_counts']
    for key, value in [('kind', kind), ('report', report), ('control', control), ('identity', identity)]:
        assert boot[key] == run[key] == value, 'Boot/run/expected configuration mismatch'
    assert before['kind'] == kind and boot['case'] == 'object_report'
    config = {'scenario': 'lebanon_retail', 'frame_ms': 100, 'warmup': 5, 'window_seconds': 12}
    assert boot['seed'] == 1
    arguments = dict(x.split('=', 1) for x in run['arguments'].split()[1:])
    for key, value in config.items():
        assert boot[key] == value and arguments[key] == str(value), 'Actual command configuration mismatch'
    for key, value in [('kind', kind), ('report', report), ('control', control), ('identity', identity)]:
        assert arguments[key] == value
    assert Path(arguments['root']).resolve() == Path(run['root']).resolve()
    assert Path(arguments['out']).resolve() == (Path(run['out']) / 'trace.jsonl').resolve()
    assert census['duplicate_identities'] == 0 and census['structures'] > 0
    assert sum(census['n_' + k] for k in KINDS) == census['structures']
    assert all(census['n_' + k] > 0 for k in KINDS)
    assert before['alive'] is True and after['present'] is True and before['host_id'] == after['host_id']
    assert summary['done'] is True and summary['report'] == report
    assert summary['structures_before'] == summary['structures_after'] == census['structures']
    assert summary['alive_before'] == before['alive'] and summary['alive_after'] == after['alive']
    assert summary['damage_before'] == before['damage_level'] and summary['damage_after'] == after['damage_level']
    assert counts['asked_identify'] == census['structures'], 'Second enumeration must reuse identities'
    assert run['build_info_sha256'] == build_sha
    if report == 'none':
        assert summary['changed'] is False and after['alive'] is True
        assert after['damage_level'] == before['damage_level']
    else:
        receipt = events['object_receipt']
        assert receipt['host_id'] == before['host_id'] == after['host_id'], 'Receipt identity mismatch'
        assert receipt['damage_level_before'] == before['damage_level'], 'Receipt before-state mismatch'
        assert receipt['damage_level_after'] == after['damage_level'], 'Receipt after-state mismatch'
        assert receipt['entity_type'] == {'site': 29, 'segment': 26}[kind] and receipt['rostered'] is False
        assert summary['changed'] is True
        if report == 'damage':
            assert after['alive'] is True and after['damage_level'] == before['damage_level'] // 2
            assert receipt['killed'] is False
        else:
            assert after['alive'] is False and receipt['killed'] is True
            assert after['damage_level'] == before['damage_level']
            assert after['alive_' + kind] < census['alive_' + kind]
    prefix = 'own-world/structure/' if identity == 'own' else 'eech-world:object:'
    assert before['host_id'].startswith(prefix)
    if control != 'none':
        row = events['object_control']
        expected = {'raise': 'cannot be repaired by a report', 'duplicate': 'already destroyed: a fact is applied once', 'unknown': 'was never issued'}[control]
        assert row['control'] == control and row['refused'] is True and expected in row['error']
        assert 'poison' not in row['error'].lower(), 'Secondary poisoned-state error is insufficient'
    return events

def main():
    proof = BASE / 'root-r3-64cf-six-report-completion-v2.json'
    assert not proof.exists(), 'Preserve existing audit'
    drivers = driver_set(read(BASE / 'root-r3-64cf-six-original-driver-exits.json'))
    inputs = read(BASE / 'root-r3-64cf-six-post-inputs.json')
    build = read(BASE / 'root-r3-64cf-six-post-build.json')
    assert inputs['passed'] and {x['case'] for x in inputs['results']} == set(EXPECTED)
    assert all(x['actual_exit'] == 0 for x in inputs['results'])
    assert build['passed'] and len(build['files']) == 42
    assert not subprocess.check_output(['git', '-C', str(SOURCE), 'status', '--porcelain'], text=True).strip()
    cases = {}; events = {}
    for name, driver in drivers.items():
        folder = BASE / 'out' / name
        run = read(folder / 'run.json'); trace = folder / 'trace.jsonl'
        assert trace.stat().st_size < 32768, 'Bounded case trace required'
        rows = [json.loads(line) for line in trace.read_text(encoding='utf-8').splitlines()]
        events[name] = validate_case(name, driver, run, rows, build['manifest_sha256'])
        cases[name] = dict(driver=driver, run=run, trace=str(trace), trace_bytes=trace.stat().st_size,
                           trace_sha256=digest(trace), rows=rows, passed=True)
    def event(case, event_name):
        return events['object-report-' + case + '-02'][event_name]
    assert event('site-none', 'object_before') == event('site-damage-raise', 'object_before') == event('site-destroy-duplicate', 'object_before')
    assert event('segment-none', 'object_before') == event('segment-destroy', 'object_before')
    before = event('segment-none', 'object_census')['alive_segment']
    assert event('segment-none', 'object_after')['alive_segment'] == before
    after = event('segment-destroy', 'object_after')['alive_segment']
    assert after < before
    counts = event('segment-destroy', 'object_aim_counts')
    assert all(counts['asked_bridge_' + k] > 0 for k in ['kill', 'neighbour', 'fall'])
    assert not any(k.startswith('asked_bridge_') for k in event('segment-none', 'object_aim_counts'))
    record = dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        status='PASS within six bounded shipping report/withhold/refusal cases; independent runtime review pending',
        runtime_source='64cf0f1c5551a2e59309a92917c32e6189d8ae60', source=str(SOURCE), source_clean=True,
        candidate_test_only_sha='a89a668ae2fa5594a93fb6bbccf46e1c9b7bff38',
        auditor=str(Path(__file__)), auditor_sha256=digest(Path(__file__)),
        build_info_sha256=build['manifest_sha256'], all42_exports_match=True, all6_post_retail_manifests_match=True,
        scenarios=cases, bridge_alive_before=before, bridge_alive_after_report=after,
        actual_state_result='Withhold preserves actual site/segment state; damage halves actual site health; destruction changes alive state while retaining health; segment report reaches bridge handlers and kills a dependent segment. Own-issued identity accepted; all three specific primary guards refuse.',
        superseded_helper='audit-r3-64cf-six.py and root-r3-64cf-six-report-completion.json retained unchanged. v1 did not enforce complete case set, row order or receipt correlation; these are helper defects, not raw native-run defects.',
        limitations=['Controlled reported facts, not weapon-hit reports or all-six-kind mutation coverage.',
          'Own case replaces identify only, not the whole combat host.',
          'Bridge counters cover the whole window, not report-only policy; earlier closed bridge witnesses and source support route/repair policy.',
          'No Tacview in these probes; final ordinary40/full8 and D2 delivery remain pending.',
          'Old f470 six01 crashes remain failures. No native rerun, normalization, baseline or tolerance change.'])
    proof.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8', newline='\n')
    print(json.dumps(dict(proof=str(proof), passed=True, cases=len(cases))))

if __name__ == '__main__':
    main()
