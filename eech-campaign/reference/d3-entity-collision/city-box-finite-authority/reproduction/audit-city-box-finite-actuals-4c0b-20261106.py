"""Read retained actuals; no native rerun, private geometry or postfatal engine API."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import sys

b, ev = Path('R:/d3-evidence'), Path('D:/EECH-Migration-Evidence')
source = '4c0b89263f41164e1c2379f3be99ac8d6c575173'
sha = lambda raw: hashlib.sha256(raw).hexdigest()
build = json.loads((b / 'city-box-corrected-4c0b89263-windows-20261106-completion.json').read_bytes())
assert build['source'] == source and build['actual_exit'] == 0
snapshot = Path(build['snapshot'])
roles = {}
for name, row in build['profiles'].items():
    assert row['actual_exit'] == 0 and row['verified_payloads'] == 88
    binary = Path(row['binary'])
    assert all(sha((binary / x['path']).read_bytes()) == x['sha256'] for x in row['payloads'])
    roles[name] = dict(actual_build_exit=0, payloads_rehashed=88, build_info_sha256=row['build_info_sha256'])
cases, manifests, input_blocks = {}, [], []
for case, stem in [('short', 'city-box-ordinary-short'), ('healthy', 'city-box-healthy'), ('withdrawal', 'city-box-withdrawn-construction')]:
    stem += '-4c0b89263-20261106'
    completion_path = b / (stem + '-completion.json')
    outer = json.loads(completion_path.read_bytes())
    assert outer['source'] == source and outer['actual_exit'] == outer['actual_helper_exit'] == outer['actual_native_exit'] == 0
    out = Path(outer['output'])
    controller_path = out / 'controller.json'
    c = json.loads(controller_path.read_bytes())
    assert c['actual_exit'] == c['actual_native_exit'] == 0 and c['verified_payloads'] == 88
    assert len(c['stages']) == 3 and all(x['actual_exit'] == 0 for x in c['stages'])
    files = []
    for label in ['pre-inputs', 'post-inputs']:
        p = out / (label + '.json')
        raw = p.read_bytes()
        m = json.loads(raw)
        assert m['files'] == len(m['inputs']) == 1179 and m['bytes'] == 102239559
        # Preserve the literal complete serialized1179-entry array; no subset or normalized projection.
        text = raw.decode('utf8')
        start = text.index(':', text.index('"inputs"')) + 1
        while text[start].isspace():
            start += 1
        value, end = json.JSONDecoder().raw_decode(text, start)
        assert value == m['inputs']
        block = text[start:end].encode('utf8')
        input_blocks.append(block)
        manifests.append(raw)
        files.append(dict(path=str(p), bytes=len(raw), sha256=sha(raw), full_input_array_bytes=len(block), full_input_array_sha256=sha(block), manifest_files=1179, combined_sha256=m['combined_sha256']))
    cases[case] = dict(native_helper_outer_exits=[0, 0, 0], completion=str(completion_path), completion_sha256=sha(completion_path.read_bytes()), controller=str(controller_path), controller_sha256=sha(controller_path.read_bytes()), output=str(out), payloads_verified=88, pre_post_actual_exits=[0, 0], full_input_manifests=files)
assert len(manifests) == 6 and all(raw == input_blocks[0] for raw in input_blocks)
short_path = b / 'city-box-ordinary-short-comparison-4c0b-20261106.json'
short = json.loads(short_path.read_bytes())
assert short['source'] == source and short['all_three_complete_literal_equal']
for row in short['files']:
    raw = (Path(short['candidate']) / row['file']).read_bytes()
    assert raw == (Path(short['baseline']) / row['file']).read_bytes() and sha(raw) == row['candidate_sha256']
healthy_path = b / 'city-box-healthy-comparison-4c0b-20261106.json'
healthy = json.loads(healthy_path.read_bytes())
assert healthy['source'] == source and healthy['actual_exit'] == 0 and healthy['new_city_expansions_before'] == 54825
assert healthy['only_declared_city_state_before_delta'] and healthy['complete_terminal_literal_equal']
assert healthy['changed_common_fields'] == {'city_state_before': {'old': 83408, 'new': 28583}}
withdrawn = json.loads(Path(cases['withdrawal']['completion']).read_bytes())['safe_facts']
assert withdrawn['box_eligible'] == withdrawn['box_withdrawn'] == 1
assert withdrawn['boot_stop'] is True and withdrawn['completed'] is False
assert withdrawn['city_expand_before'] == withdrawn['city_expand_total'] == 1 and withdrawn['city_expand_answered'] == 0
assert withdrawn['request_faults'] == 0 and withdrawn['reply_faults'] == 1 and withdrawn['physical_after_stop'] == -1
binding = json.loads((b / 'city-box-corrected-4c0b-source-check-binding-20261106.json').read_bytes())
before = Path(binding['after_inputmap'])
m = json.loads(before.read_bytes())
assert m['head_sha'] == source and m['file_count'] == 3953
after = b / 'city-box-corrected-4c0b89263-snapshot-after-native-20261106.json'
assert not after.exists()
subprocess.run([sys.executable, '-B', '-X', 'utf8', str(b / 'sector-roster-inputmap-batched-20261106.py'), m['repo'], str(after), m['head_rev']], check=True, stdout=subprocess.DEVNULL)
assert before.read_bytes() == after.read_bytes() and all(x['raw_equals_head_blob_bytes'] for x in json.loads(after.read_bytes())['files'].values())
assert subprocess.check_output(['git', '-C', str(snapshot), 'status', '--porcelain', '--untracked-files=all']) == b''
proof = dict(source=source, actual_exit=0, snapshot=str(snapshot), cases=cases, Windows_profiles_resealed=roles,
    ordinary_short_comparison=dict(path=str(short_path), sha256=sha(short_path.read_bytes()), all_three_COMPLETE_literal_equal=True),
    healthy_comparison=dict(path=str(healthy_path), sha256=sha(healthy_path.read_bytes()), prelaunch_expansions=54825, common_fields=186, only_changed_common_field='city_state_before', old=83408, new=28583, exact_declared_delta=54825, complete_terminal_literal_equal=True, terminal_bytes=180, terminal_sha256=healthy['complete_terminal_sha256']),
    real_BOOT_withdrawal=withdrawn, six_complete1179_input_arrays_literal_equal=True,
    all_six_full_manifest_bytes_literal_equal=all(raw == manifests[0] for raw in manifests),
    complete_input_array_sha256=sha(input_blocks[0]), post_native_source_maps_literal_equal=True,
    post_native_map_sha256=sha(after.read_bytes()), source_raw_Git_files=3953,
    scope='Retained actual facts for SAME independent review. No native rerun, raw IDs/poses/reports published, normalized physical signatures, postfatal API reads or wholeD3/fullpack/Task49 acceptance. REQUIRED_CITY_BOX_LIFECYCLE_RETENTION, physical flags and allten requirements remain.',
    independent_actual_review_pending=True, verified_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
out = b / 'city-box-finite-actual-facts-4c0b-20261106.json'
assert not out.exists()
out.write_bytes((json.dumps(proof, indent=2) + '\n').encode())
print(json.dumps({'actual_exit': 0, 'all_three_case_triples': '0/0/0', 'six_full1179_arrays_literal_equal': True, 'all_six_manifest_bytes_literal_equal': proof['all_six_full_manifest_bytes_literal_equal'], 'source_files': 3953, 'profile_payloads_rehashed': [88, 88, 88], 'proof_sha256': sha(out.read_bytes())}))
