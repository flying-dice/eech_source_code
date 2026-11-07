"""Standard exact-source profiles, only after corrected review and raw/check clearance."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
import traceback

b = Path('R:/d3-evidence')
review, expected, gate_path, gate_expected = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3]), sys.argv[4]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(review) == expected and sha(gate_path) == gate_expected
r, gate = json.loads(review.read_bytes()), json.loads(gate_path.read_bytes())
assert r['decision']['native_launch_clearance'] is True and r['decision']['findings'] == []
assert gate['actual_exit'] == 0 and gate['native_clearance'] is True
assert gate['source_review_sha256'] == expected
assert gate['all_snapshot_raw_bytes_equal_Git'] and gate['final_pre_g_to_Root_post_terminal_per_path_hashes_and_sizes_identical']
chronology_path=Path(gate['chronology_proof']);assert sha(chronology_path)==gate['chronology_proof_sha256']
chronology=json.loads(chronology_path.read_bytes());assert chronology['source']==gate['source'] and chronology['final_pre_g_to_Root_post_terminal_per_path_hashes_and_sizes_identical'] is True
source, snap = gate['source'], Path(gate['snapshot'])
assert re.fullmatch('[0-9a-f]{40}', source) and source in json.dumps(r['source'])
short = source[:9]
assert snap.as_posix() == 'D:/EECH-Migration-Evidence/city-lifecycle-witness-' + short + '-20261106-source'
assert subprocess.check_output(['git', '-C', str(snap), 'rev-parse', 'HEAD'], text=True).strip() == source
assert subprocess.check_output(['git', '-C', str(snap), 'status', '--porcelain', '--untracked-files=all'], text=True) == ''
assert not (snap / '.git/objects/info/alternates').exists()
repo = snap / 'eech-campaign'
original = (repo / 'tools/build-windows.sh').read_bytes()
old = b'-p eech-dc -p eech-world > /target/build.log'
new = b'-p eech-dc -p eech-world --features eech-dc/weapon-fixture,eech-dc/collision-fixture > /target/build.log'
banner = b'echo "target: x86_64-pc-windows-gnu, release"'
new_banner = b'echo "target: x86_64-pc-windows-gnu, release, FIXTURE BUILD (eech-dc/weapon-fixture,eech-dc/collision-fixture): explicit Original control, not shipping"'
assert original.count(old) == original.count(banner) == 1
adapted = original.replace(old, new).replace(banner, new_banner)
assert sum(x != y for x, y in zip(original.splitlines(), adapted.splitlines())) == 2
(repo / 'target').mkdir(exist_ok=True)
adapter = repo / ('target/.build-city-lifecycle-witness-original-' + short + '.sh')
assert not adapter.exists()
adapter.write_bytes(adapted)
stem = 'city-lifecycle-witness-' + short + '-windows-20261106'
volume = 'eech-win-city-lifecycle-witness-' + short + '-20261106'
receipt = b / (stem + '-process.json')
assert not receipt.exists()
helper = b / 'build-city-lifecycle-witness-windows-20261106.sh'
state = dict(controller_pid=os.getpid(), source=source, snapshot=str(snap), state='running', started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), cache=volume, source_clearance=str(review), source_clearance_sha256=expected, raw_check_gate=str(gate_path), raw_check_gate_sha256=gate_expected, controller_sha256=sha(Path(__file__)), helper_sha256=sha(helper), original_fixture_adapter_sha256=sha(adapter), profiles={}, scope='Exact corrected-source ordinary/hosted/Original standard profiles. Actual header-derived bound COFF/features/exports/payload audit and native comparisons remain separate gates. Source clearance permits these builds only; native authority, complete recordings, Task49 and whole-D3 acceptance remain open.')
save = lambda: receipt.write_bytes((json.dumps(state, indent=2) + '\n').encode())
save()
try:
    for profile in ['ordinary', 'hosted', 'original']:
        args = ['wsl', '-d', 'EECH-D2-Build', '--', 'sh', '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/build-city-lifecycle-witness-windows-20261106.sh', '/mnt/d/' + snap.as_posix()[3:], source, volume, profile]
        with (b / (stem + '-' + profile + '.log')).open('xb') as log, (b / (stem + '-' + profile + '.stderr.log')).open('xb') as err:
            child = subprocess.Popen(args, cwd='C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS', stdout=log, stderr=err, creationflags=subprocess.CREATE_NO_WINDOW)
            row = dict(pid=child.pid, args=args, state='running', started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
            state['profiles'][profile] = row
            save()
            code = child.wait()
        row.update(state='terminal', actual_exit=code, completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
        save()
        assert code == 0, profile + ' build failed'
        target = dict(ordinary='windows-city-lifecycle-witness-' + short, hosted='weapon-fixture/city-lifecycle-witness-' + short, original='weapon-fixture/city-lifecycle-witness-original-' + short)[profile]
        binary = repo / 'target' / target
        info = (binary / 'BUILD-INFO.txt').read_text()
        assert 'commit: ' + source in info and 'uncommitted changes in the build inputs: 0 files' in info
        assert ('FIXTURE BUILD' in info) == (profile != 'ordinary')
        entries = re.findall(r'^  ([0-9a-f]{64})  (.+)$', info, re.M)
        wanted = {'eech_dc.dll', 'eech-world.exe', 'lua.dll'} | {'lua/' + p.name for p in (repo / 'crates/eech-world/lua').glob('*.lua')}
        assert len(entries) == len(wanted) and {name for _, name in entries} == wanted
        assert all(sha(binary / name) == expected_hash for expected_hash, name in entries)
        row.update(binary=str(binary), build_info_sha256=sha(binary / 'BUILD-INFO.txt'), verified_payloads=len(entries), build_info_header=info.split('\nsha256:')[0], payloads=[dict(path=name, sha256=expected_hash) for expected_hash, name in entries])
        save()
    assert subprocess.check_output(['git', '-C', str(snap), 'status', '--porcelain', '--untracked-files=all'], text=True) == ''
    state.update(state='terminal', actual_exit=0, completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
except Exception:
    state.update(state='terminal', actual_exit=1, error=traceback.format_exc(), completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
    raise
finally:
    if state['state'] == 'terminal':
        (b / (stem + '-completion.json')).write_bytes(receipt.read_bytes())
