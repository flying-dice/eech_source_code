"""Audit the distinct generic-host witness. Never relax the reference checker."""
import datetime,hashlib,importlib.util,json,re
from pathlib import Path
w=Path('R:/d2-evidence/weapon-characterization-20261027')
trace=w/'out/h1-case-own/trace.jsonl'
model=w/'out/h1-case-own/trace.jsonl.model.json'
events=[json.loads(line) for line in trace.read_text(encoding='utf-8').splitlines() if line.strip()]
spec=importlib.util.spec_from_file_location('reference_case_checker','R:/eech_source_code/eech-campaign/tools/wpn-case-verify.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
assert events[-1]=={'checks':11,'event':'summary','failed':0}
boots=[e for e in events if e.get('event')=='boot']
ends=[e for e in events if e.get('event')=='run']
assert len(boots)==len(ends)==1
for k,v in dict(scenario='lebanon_retail',case='controlled_launch',seed=1,frame_ms=100,warmup=60,settle=700).items():
    assert boots[0][k]==v,(k,boots[0])
checks=[e for e in events if e.get('event')=='check']
assert [e['name'] for e in checks]==module.EXPECTED and all(e['ok'] is True for e in checks)
run=[e for e in events if e.get('scope')=='run']
hashes=[e for e in run if e.get('event')=='wpn-hash']
assert len(hashes)==1 and all(re.fullmatch('[0-9a-f]{16}',str(hashes[0][k])) for k in ('table','launch','flight','terminal','decision'))
counters={e['name']:e['value'] for e in run if e.get('event')=='wpn-counter'}
for name in ('birth_calls','move_calls','terminal_calls','launch_no_ammo','launch_created'):
    assert type(counters[name]) is int and counters[name]>0,(name,counters.get(name))
for name in ('birth_track_overflow','move_unregistered','terminal_unregistered','destroy_unregistered'):
    assert counters[name]==0,(name,counters[name])
logs=[e for e in run if e.get('event')=='wpn-log']
stored=[e for e in events if e.get('event')=='wpn-ev' and e.get('scope')=='log']
assert len(logs)==1 and logs[0]['overflow']==0
assert logs[0]['events']==logs[0]['stored']==logs[0]['drained']==len(stored)
assert [e['seq'] for e in stored]==list(range(1,len(stored)+1))
made=[e for e in events if e.get('event')=='targeted_launches']
assert len(made)==1 and made[0]['created']>=3
calls=json.loads(model.read_text(encoding='utf-8'))
assert calls['aim_model']=='own_model'
required=('weapon_birth','release','weapon_step','animate','weapon_decoy')
assert all(type(calls['calls'].get(k)) is int and calls['calls'][k]>0 for k in required)
assert counters['table_supplies']==0,'This witness deliberately supplies no EECH-private table diagnostic'
assert 'inputs IDENTICAL' in (w/'root-h1-own-post-input.log').read_text(encoding='utf-8-sig')
build=json.loads((w/'root-post-h1-own-pnever-build.json').read_text(encoding='utf-8-sig'))
original_build=json.loads((w/'root-independent-a137-clean-build.json').read_text(encoding='utf-8-sig'))
assert build['passed'] and build['manifest_sha256']==original_build['manifest_sha256']
record=dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source='a137fc3ec1963d88de5f0b8562145b358b50e111',
    actual_native_driver_exit=0,completed_utc='2026-10-28T02:33:22Z',wall_seconds=150,
    native_checks=11,all_true=True,frames=ends[0]['frames'],clock=ends[0]['elapsed'],
    generic_audit_passed=True,model_calls=calls['calls'],ordered_log_events=len(stored),ordered_complete_no_overflow=True,
    trace_bytes=trace.stat().st_size,trace_sha256=hashlib.sha256(trace.read_bytes()).hexdigest(),
    model_file_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),post1179_input_actual_exit=0,all35_exports_and_BUILD_INFO_unchanged=True,
    reference_checker_actual_exit=2,reference_checker_reason='No table_supplies activity (0); reference-only EECH table diagnostic, intentionally absent from this valid distinct host model',
    disposition='Reference checker unchanged, not claimed passed. Separate generic audit checks complete11/native geometry/log/counters and positive new-operation calls. No golden/tolerance/comparator/raw change or healthy rerun.',
    not_claimed='No generic seduction/reacquisition activity: this model never locks onto a decoy. Reference/reacquire_never cohort and focused contract tests supply separate evidence; not whole45 acceptance.',
    remaining='Same independent NEW finalproof review; remaining provider/refusal cohort and full40/all8/post3/currentCI/normalmerge45')
(w/'root-h1-generic-completion.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:record[k] for k in ('actual_native_driver_exit','native_checks','generic_audit_passed','model_calls','reference_checker_actual_exit','not_claimed')},indent=2))
