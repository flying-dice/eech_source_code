"""Exactly one approved corrected-source case; immutable roots and separate actual exits."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import traceback

b, ev = Path('R:/d3-evidence'), Path('D:/EECH-Migration-Evidence')
source = '4c0b89263f41164e1c2379f3be99ac8d6c575173'
case = sys.argv[1]
assert case in ['short', 'healthy', 'withdrawal']
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
build = json.loads((b / 'city-box-corrected-4c0b89263-windows-20261106-completion.json').read_bytes())
assert build['source'] == source and build['actual_exit'] == 0
assert build['source_clearance_sha256'] == '50eee6bbaac7f114e37bc7e4fb774eb04c14c23d792f8c319df8034b0f3a964e'
profile_proof = b / 'city-box-windows-profile-isolation-4c0b-20261106.json'
assert sha(profile_proof) == '25a62d05cd15aeb4f14151c78e789a4478995d418998b00bc4cde11bc810b527'
pp = json.loads(profile_proof.read_bytes())
assert pp['source'] == source and pp['profile_isolation_passed'] and pp['new_native_prerequisite_met']
snapshot = Path(build['snapshot'])
prep = json.loads((b / 'city-box-native-inputs-20261106/preparation.json').read_bytes())
assert prep['actual_exit'] == 0
assert prep['input_tool_sha256'] == sha(snapshot / 'eech-campaign/tools/input-manifest.py')
assert prep['reference_sha256'] == sha(snapshot / 'eech-campaign/reference/lebanon-3h/inputs.json')
root = ev / 'city-box-native-roots-20261106' / dict(short='short', healthy='healthy', withdrawal='withheld-construction')[case]
assert any(x['root'] == str(root) and x['state'].startswith('VERIFIED UNUSED') for x in prep['roots'])
stem = 'city-box-' + dict(short='ordinary-short', healthy='healthy', withdrawal='withdrawn-construction')[case] + '-4c0b89263-20261106'
out, receipt = ev / stem, b / (stem + '-process.json')
assert not out.exists() and not receipt.exists()
marker = root.parent / ('CONSUMED-CITY-BOX-' + case + '-4c0b-20261106.json')
assert not marker.exists()
role = 'ordinary' if case == 'short' else 'hosted'
binary = Path(build['profiles'][role]['binary'])
assert all(sha(binary / x['path']) == x['sha256'] for x in build['profiles'][role]['payloads'])
helper = b / 'run-entity-public-native-pipe-20261105.py'
assert sha(helper) == '3da2855f8b8b67c73d4ac0e157b40c81acdf633bdf21560daa3e588f024b0582'
args = [sys.executable, '-B', '-X', 'utf8', str(helper), '--source', source, '--snapshot', str(snapshot), '--bin', str(binary), '--root', str(root), '--out', str(out), '--script', 'campaign.lua' if case == 'short' else 'entity-authority.lua', '--scenario', 'lebanon_retail']
if case == 'short':
    args += ['--arg=hours=0.001', '--arg=record=1', '--arg=record_every=10', '--arg=observe_every=1', '--arg=observe_supply=1', '--arg=acmi=' + (out / 'recording.acmi').as_posix(), '--arg=observe=' + (out / 'observations.jsonl').as_posix()]
else:
    short_proof = json.loads((b / 'city-box-ordinary-short-comparison-4c0b-20261106.json').read_bytes())
    assert short_proof['source'] == source and short_proof['all_three_complete_literal_equal']
    if case == 'withdrawal':
        healthy = json.loads((b / 'city-box-healthy-comparison-4c0b-20261106.json').read_bytes())
        assert healthy['source'] == source and healthy['actual_exit'] == 0 and healthy['new_city_expansions_before'] > 0
    literal = b / 'city-box-authority-4c0b89263-literal.lua'
    rel = 'eech-campaign/crates/eech-world/lua/entity-authority.lua'
    raw = subprocess.check_output(['git', '-C', str(snapshot), 'show', source + ':' + rel])
    if literal.exists():
        assert literal.read_bytes() == raw
    else:
        literal.write_bytes(raw)
    assert literal.read_bytes() == (binary / 'lua/entity-authority.lua').read_bytes()
    blob = subprocess.check_output(['git', '-C', str(snapshot), 'rev-parse', source + ':' + rel], text=True).strip()
    args += ['--literal-driver', str(literal), '--literal-source', source, '--literal-git-blob', blob, '--literal-sha256', sha(literal), '--arg=mode=' + ('control' if case == 'healthy' else 'withdrawn_box_state'), '--arg=profile=hosted']
state = dict(source=source, case=case, controller_pid=os.getpid(), state='running', started_utc=now(), root=str(root), output=str(out), args=args, scope='ONE exact approved ordinary short or healthy or real BOOT withdrawal; no actor/time/geometry search, fullpack or whole-D3 acceptance.')
save = lambda: receipt.write_bytes((json.dumps(state, indent=2) + '\n').encode())
marker.write_bytes((json.dumps(dict(source=source, case=case, output=str(out), started_utc=now())) + '\n').encode())
save()
try:
    with (b / (stem + '.stdout.log')).open('xb') as stdout, (b / (stem + '.stderr.log')).open('xb') as stderr:
        child = subprocess.Popen(args, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        state['pid'] = child.pid
        save()
        code = child.wait()
    state['actual_helper_exit'] = code
    c = json.loads((out / 'controller.json').read_bytes())
    state['actual_native_exit'] = c.get('actual_native_exit')
    save()
    assert code == c['actual_exit'] == c['actual_native_exit'] == 0
    assert c['verified_payloads'] == 88 and len(c['stages']) == 3 and all(x['actual_exit'] == 0 for x in c['stages'])
    if case == 'short':
        base = Path('R:/d3-evidence/model-reference-profile-recording-80fd62ea4-20261030')
        files = []
        for name in ['recording.acmi', 'observations.jsonl', 'metrics.json']:
            candidate, baseline = (out / name).read_bytes(), (base / name).read_bytes()
            files.append(dict(file=name, candidate_bytes=len(candidate), baseline_bytes=len(baseline), candidate_sha256=hashlib.sha256(candidate).hexdigest(), baseline_sha256=hashlib.sha256(baseline).hexdigest(), literal_complete_equal=candidate == baseline))
        proof = dict(source=source, candidate=str(out), baseline=str(base), native_actual_exit=0, helper_actual_exit=0, controller_actual_exit=0, pre_post_input_exits=[0, 0], payloads_unchanged=88, files=files, all_three_complete_literal_equal=all(x['literal_complete_equal'] for x in files), scope='THREE complete ordinary3.6simsec files only, not full40/all8/authority/wholeD3 acceptance.')
        p = b / 'city-box-ordinary-short-comparison-4c0b-20261106.json'
        assert not p.exists()
        p.write_bytes((json.dumps(proof, indent=2) + '\n').encode())
        assert proof['all_three_complete_literal_equal']
        state['all_three_complete_literal_equal'] = True
    else:
        rows = [json.loads(line) for line in (out / 'model.jsonl').read_text().splitlines()]
        authorities = [x for x in rows if x.get('event') == 'authority']
        assert len(authorities) == 1
        a = authorities[0]
        if case == 'healthy':
            assert a['completed'] and a['request_faults'] == a['reply_faults'] == a['case_weapons_pending'] == 0
            n = a['city_expand_before']
            assert n > 0 and a['city_expand_answered'] == a['city_expand_total'] == n + a['city_expand_after']
            parent = Path('D:/EECH-Migration-Evidence/city-state-healthy-78ef839dd-20261106')
            old_rows = [json.loads(line) for line in (parent / 'model.jsonl').read_text().splitlines()]
            old_a = next(x for x in old_rows if x.get('event') == 'authority')
            shared = sorted(set(a) & set(old_a))
            differences = {k: {'old': old_a[k], 'new': a[k]} for k in shared if old_a[k] != a[k]}
            assert set(differences) == {'city_state_before'} and old_a['city_state_before'] - a['city_state_before'] == n
            terminal = lambda p: [raw for raw in (p / 'model.jsonl').read_bytes().splitlines(keepends=True) if json.loads(raw).get('event') == 'terminal']
            t, old_t = terminal(out), terminal(parent)
            assert len(t) == len(old_t) == 1 and t == old_t
            proof = dict(source=source, actual_exit=0, candidate=str(out), parent_source='78ef839dd75d325c42acb3127d097fee400388c2', parent=str(parent), native_helper_outer_exits=[0, 0, 0], new_city_expansions_before=n, new_city_expansions_after=a['city_expand_after'], all_expansions_answered=True, common_authority_fields=len(shared), changed_common_fields=differences, only_declared_city_state_before_delta=True, new_fields=sorted(set(a)-set(old_a)), complete_terminal_literal_equal=True, complete_terminal_bytes=len(t[0]), complete_terminal_sha256=hashlib.sha256(t[0]).hexdigest(), request_faults=0, reply_faults=0, case_weapons_pending=0, scope='Finite healthy constructor positive eligibility and conformance only; no lifecycle retention, moving or wholeD3 acceptance.')
            p = b / 'city-box-healthy-comparison-4c0b-20261106.json'
            assert not p.exists()
            p.write_bytes((json.dumps(proof, indent=2) + '\n').encode())
            state['safe_facts'] = dict(new_city_expansions_before=n, complete_terminal_literal_equal=True, changed_common_field='city_state_before', request_faults=0, reply_faults=0)
        else:
            assert a['mode'] == 'withdrawn_box_state' and a['boot_stop'] is True and not a['completed']
            assert a['box_eligible'] == a['box_withdrawn'] == 1
            assert a['city_expand_total'] >= 1 and a['city_expand_before'] >= 1
            assert a['box_withdrawn_cause'] == 'city_box_building_state_withdrawn' and a['failure_marker'] == 'entity_scan.refused'
            assert '[roster.city_box.building_state]' in a['failure_line'] and 'withdrew' in a['failure_line']
            assert a['request_faults'] == 0 and a['reply_faults'] == 1 and a['first_fault'] == 'scan reply: result refused'
            assert a['physical_after_stop'] == -1
            state['safe_facts'] = {k: a[k] for k in ['completed', 'boot_stop', 'city_expand_before', 'city_expand_after', 'city_expand_total', 'city_expand_answered', 'box_eligible', 'box_withdrawn', 'box_withdrawn_cause', 'physical_after_stop', 'request_faults', 'reply_faults', 'first_fault', 'failure_marker']}
    state.update(actual_exit=0, verified_payloads=88, pre_post_inputs_actual_exit=[0, 0])
except BaseException:
    state.update(actual_exit=1, error=traceback.format_exc())
    raise
finally:
    state.update(state='terminal', completed_utc=now())
    save()
    (b / (stem + '-completion.json')).write_bytes(receipt.read_bytes())
print(json.dumps({k: state[k] for k in ['case', 'actual_exit', 'actual_native_exit', 'actual_helper_exit', 'output']}))
