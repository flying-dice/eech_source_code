"""Schema4 zero-step candidate: input seal, PRE/POST source-row census, freeze, stage and literal Git publication join.
Modes: seal | PRE | POST | freeze | stage | join. No build, native or Windows action."""
from pathlib import Path
import datetime, hashlib, json, re, subprocess, sys
E = Path(__file__).resolve().parent
R = E.parent
W = R / 'collision-characterization-20261029'
P = W / 'eech-campaign/reference/d3-entity-collision/zero-step-contract'
BASE = '23ec563cc48b16bf90c6de061ee13efc109475dd'
RUNTIME = '2fcf69f9b8972b4353687dc2ca1eaaac780836f3'
READERS = ['run-normal-fire-zs4-direct-20261111.py', 'check-normal-fire-zs4-linux-20261111.sh', 'run-zs4-final-20261111.py', 'zs4-proof-20261111.py', 'sector-roster-inputmap-batched-20261106.py']
# External records copied literally into the packet (path relative to R).
ROOTS = ['zero-step-lead216-live-services-20261111.json', 'zero-step-23ec-source-proposal-20261111.json', 'fable-opus-20261110/ledger216-contract.html',
         'independent-persistent-motion-23ec-source-20261111.json', 'independent-persistent-motion-23ec-terminal-20261111.json',
         'persistent-motion-23ec-terminal-Root-binding-20261111.json', 'independent-public-domain-profile-d2bde-source-20261108.json',
         'D3-REQUIREMENTS.json', 'normal-fire-observer-2fcf69-build-Root-binding-20261108.json']
LOCAL = ['zs4-cache-lease-20261111.json', 'dockerd-restore-process.json', 'dockerd-restore-attempt1-FAILED-unquoted-process.json',
         'dockerd-restore-attempt1-FAILED-unquoted.stderr.log', 'dockerd-restore-attempt1-FAILED-unquoted.stdout.log']
GUARDS = ['g1-fix-blocker', 'g2-characterize', 'g3-check']
DEVELOPMENT = ['d1-real-positive', 'd2-old-gate-RED', 'd3-target-GREEN', 'd4-facade-default', 'd5-facade-observer', 'd6-format', 'd7-clippy']
OTHER = ['zs4-expected-changes-20261111.json', 'zs4-readers-sealed-20261111.json', 'zs4-source-audit-20261111.json']
ARTIFACTS = ['eech-campaign/crates/eech-world/lua/' + n for n in ['normal-fire-session.lua', 'normal-fire-correlation.lua', 'physical-flags-read-transaction.lua', 'normal-fire-session-contract.lua']] + \
            ['eech-campaign/tools/' + n for n in ['normal_fire_capture.py', 'normal_fire_capture_controller.py', 'test_normal_fire_capture.py', 'native_flag_read.py', 'native_flag_read_controller.py', 'migration-progress.py']] + \
            ['eech-campaign/docs/d3-public-session-finalization.md', 'eech-campaign/crates/eech-world/tests/normal_fire_session.rs',
             'eech-campaign/crates/eech-world/tests/lua/normal-fire-persistent-control.lua', 'eech-campaign/crates/eech-world/tests/lua/normal-fire-session-control.lua',
             'eech-campaign/crates/eech-world/tests/data/synthetic-normal-fire-positive.txt', 'eech-campaign/crates/eech-world/tests/data/synthetic-normal-fire-same-frame.txt']
FINAL = ['s1-python', 's2-fmt', 's3-clippy', 's4-session-lua', 's5-facade-default', 's6-facade-observer']
TEST_ONLY = 'eech-campaign/crates/eech-dc/src/native_flag_read_refusal_check.rs'
sha = lambda b: hashlib.sha256(b).hexdigest()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*a): return subprocess.check_output(['git', '-C', str(W), *a])
def j(p): return json.loads(p.read_bytes())
def put(p, b):
    with p.open('xb') as f: f.write(b)
def dump(p, o): put(p, (json.dumps(o, indent=2) + '\n').encode())
def readers(): return {n: sha((E / n).read_bytes()) for n in READERS}
def extras(): return {**{n: sha((R / n).read_bytes()) for n in ROOTS}, **{n: sha((E / n).read_bytes()) for n in READERS + LOCAL + OTHER}, **{n: sha((W / n).read_bytes()) for n in ARTIFACTS}}
def dirty(): return sorted(set(git('diff', '--name-only', 'HEAD').decode().splitlines() + git('ls-files', '--others', '--exclude-standard').decode().splitlines()))


def audit():
    """Every product/build/Cargo/static row of the prior raw map equals compiled 2fc; only eech-world Lua/test rows and the
    inherited cfg(test)-excluded seam may differ, and that seam is byte-unchanged from BASE."""
    assert git('rev-parse', 'HEAD').decode().strip() == BASE
    old = j(R / 'nq56-inputmap-POST-20261108.json'); product = {}; unchanged = {}; changed = []
    for n, r in old['files'].items():
        h = sha((W / n).read_bytes())
        if h == r['raw_sha256']: unchanged[n] = h
        else:
            assert n == TEST_ONLY or (n.startswith('eech-campaign/crates/eech-world/') and ('/lua/' in n or '/tests/' in n)), n; changed.append(n)
        if '/tests/' not in n and '/lua/' not in n and n != TEST_ONLY:
            assert h == r['raw_sha256'], n; product[n] = h
    extra = {}
    for n in git('ls-tree', '-r', '--name-only', BASE, '--', 'eech-campaign/.cargo', 'tools', 'eech-campaign/tools', 'eech-campaign/rust-toolchain.toml').decode().splitlines():
        if n.startswith('eech-campaign/tools/'): continue  # diagnostic tooling is bound separately as artifacts
        b = git('cat-file', 'blob', BASE + ':' + n); assert (W / n).read_bytes() == b, n; extra[n] = sha(b)
    module = 'eech-campaign/crates/eech-dc/src/module.rs'; importer = (W / module).read_bytes(); assert sha(importer) == old['files'][module]['raw_sha256']
    assert b'#[cfg(test)]\n#[path = "native_flag_read_refusal_check.rs"]\nmod native_flag_read_refusal_check;' in importer
    current = (W / TEST_ONLY).read_bytes(); assert current == git('cat-file', 'blob', BASE + ':' + TEST_ONLY), 'cfg(test) seam changed in this batch'
    before = git('cat-file', 'blob', RUNTIME + ':' + TEST_ONLY); start = current.index(b'// Actual mlua registration'); end = current.index(b'#[test]\nfn nr602_', start); assert current[:start] + current[end:] == before
    exception = dict(path=TEST_ONLY, before_sha256=sha(before), current_sha256=sha(current), unchanged_from_base=True, cfg_test_importer=module, importer_raw_sha256=sha(importer), production_excluded=True, old_test_body_literal_unchanged=True)
    return dict(compiled_runtime=RUNTIME, base=BASE, prior_raw_map_sha256=sha((R / 'nq56-inputmap-POST-20261108.json').read_bytes()), whole_relevant_product_build_Cargo_static_rows=len(product),
                cfg_test_raw_exception=exception, product_map_sha256=sha(json.dumps(product, sort_keys=True).encode()), extra_producer_files=extra, unchanged_prior_raw_rows=len(unchanged),
                changed_prior_test_or_Lua_rows=sorted(changed), artifacts={n: sha((W / n).read_bytes()) for n in ARTIFACTS},
                scope='All product/build/Cargo/static rows of the prior raw map equal compiled 2fc; the inherited cfg(test)-excluded seam is byte-unchanged from 23ec. Diagnostic Lua/validators/tests/docs bound separately.')


mode = sys.argv[1]
if mode == 'seal':
    assert not P.exists()
    for n in ['zs4-source-audit-20261111.json', 'zs4-expected-changes-20261111.json', 'zs4-readers-sealed-20261111.json']: assert not (E / n).exists()
    dump(E / 'zs4-source-audit-20261111.json', audit()); dump(E / 'zs4-expected-changes-20261111.json', dirty()); dump(E / 'zs4-readers-sealed-20261111.json', readers()); print('sealed', len(READERS)); sys.exit()
if mode in ['PRE', 'POST']:
    assert audit() == j(E / 'zs4-source-audit-20261111.json'); assert readers() == j(E / 'zs4-readers-sealed-20261111.json'); assert dirty() == j(E / 'zs4-expected-changes-20261111.json')
    x = extras(); dump(E / ('zs4-extra-' + mode + '-20261111.json'), x); out = E / ('zs4-inputmap-' + mode + '-20261111.json'); assert not out.exists(); start = now()
    r = subprocess.run([sys.executable, '-I', '-B', '-X', 'utf8', str(E / 'sector-roster-inputmap-batched-20261106.py'), str(W), str(out), BASE], check=False)
    dump(E / ('zs4-' + mode + '-utc-20261111.json'), dict(started_utc=start, completed_utc=now(), actual_exit=r.returncode, map_sha256=sha(out.read_bytes())))
    assert r.returncode == 0 and x == extras()
    if mode == 'POST': assert j(E / 'zs4-inputmap-PRE-20261111.json')['files'] == j(out)['files'] and x == j(E / 'zs4-extra-PRE-20261111.json')
    print(mode, j(out)['file_count'], j(out)['fingerprint_of_raw_sha256_map'], 'extras', len(x)); sys.exit()
if mode == 'freeze':
    pre = j(E / 'zs4-inputmap-PRE-20261111.json'); post = j(E / 'zs4-inputmap-POST-20261111.json'); assert pre['files'] == post['files']
    assert extras() == j(E / 'zs4-extra-PRE-20261111.json') == j(E / 'zs4-extra-POST-20261111.json')
    c = j(E / 'zs4-final-completion-20261111.json'); assert list(c['actual_exits']) == FINAL and set(c['actual_exits'].values()) == {0} and c['actual_exit'] == 0
    lua = (E / 'zs4-s4-session-lua.stdout.log').read_text()
    assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;', lua) == [('ok', '2', '0')]
    for marker in ['normal_session_proof_gates=PASS', 'schema4_receipt_faults=PASS consumed_input_and_receipt_single_field_faults=11', 'pre_effect_reference_faults=46', 'actual_TEST_domain=PASS', 'existing_move_causality=PASS', 'ZERO_STEP_PAIR=PASS healthy_position=10,100,21.999998092651367 zero_step_position=10,100,20 delegated=0']:
        assert marker in lua, marker
    cases = re.findall(r'persistent_real_collaborators=PASS case=([^\s]+)', lua); assert len(cases) == 25 and len(set(cases)) == 25
    assert 'Ran 15 tests' in (E / 'zs4-s1-python.stderr.log').read_text() and (E / 'zs4-s1-python.stderr.log').read_text().rstrip().endswith('OK')
    for label in ['s5-facade-default', 's6-facade-observer']:
        assert 'public_normal_session_driver_consumes_only_registered_userdata_methods ... ok' in (E / ('zs4-' + label + '.stdout.log')).read_text()
    red = (E / 'zs4-d2-old-gate-RED.stdout.log').read_text(); assert j(E / 'zs4-d2-old-gate-RED.json')['actual_exit'] == 101
    assert 'REAL_NEED_READ_REASK_TEST=PASS move_frame302 query_frame303 case=healthy' in red and 'UNREACHED first_code=11 first_receipt=1 outcomes=0,0,0,0,0,0,0,0,0,0,1,' in red
    P.mkdir(); (P / 'readers').mkdir(); (P / 'runs').mkdir(); (P / 'external').mkdir()
    for n in READERS: put(P / 'readers' / n, (E / n).read_bytes())
    for n in ROOTS: put(P / 'external' / Path(n).name, (R / n).read_bytes())
    for n in LOCAL + OTHER + ['zs4-extra-PRE-20261111.json', 'zs4-extra-POST-20261111.json', 'zs4-PRE-utc-20261111.json', 'zs4-POST-utc-20261111.json', 'zs4-final-process-20261111.json', 'zs4-final-completion-20261111.json']: put(P / n, (E / n).read_bytes())
    for g in GUARDS:
        for ext in ['stdout.log', 'stderr.log', 'exit']: put(P / 'runs' / (g + '.' + ext), (E / (g + '.' + ext)).read_bytes())
    for label in DEVELOPMENT + FINAL:
        for ext in ['json', 'stdout.log', 'stderr.log']: put(P / 'runs' / (label + '.' + ext), (E / ('zs4-' + label + '.' + ext)).read_bytes())
    frozen = dict(utc=now(), source_before=BASE, compiled_runtime=RUNTIME, source_rows=post['file_count'], source_fingerprint=post['fingerprint_of_raw_sha256_map'], readers=readers(), extras_count=len(extras()),
                  final_direct_exits=c['actual_exits'], development_direct_exits={l: j(E / ('zs4-' + l + '.json'))['actual_exit'] for l in DEVELOPMENT},
                  guard_direct_exits={g: (E / (g + '.exit')).read_text().strip() for g in GUARDS}, Rust_target_tests=2, Python_tests=15, real_reference_host_cases=cases,
                  real_collaborator_scope='Actual reference-host bind/movement/SharedMovers/model/FlagStore with the genuine convoy solver; synthetic Engine/C-event transport only, fresh process per case; not retail boot/fire authority',
                  report_schema=4, outcomes=27, retired_codes=[11, 12, 13], schema4_codes={'22': 'terrain present', '23': 'incomplete/nonfinite consumed input', '24': 'step time not >0', '25': 'negative pre-speed', '26': 'pre-position outside map volume', '27': 'no actual displacement / zero-step away from pre-position'},
                  RED='d2 exit101: real host/provider accept the terrain-absent MOVE (REAL_NEED_READ_REASK_TEST=PASS, receipt1) and the old schema3 gate (reviewed 5210f432/e62fb66c) rejects it code11 only',
                  GREEN='d1/d3/s4: healthy 10,100,21.999998092651367, zero-step 10,100,20 = actual pre-position, delegated 0 once, all other consumed fields the engine\'s',
                  refusals='B_dt/B_input/B_terrain/B_bounds refuse at the callback with zero real-host delegates; B_writer loses credit after the effect; terrain/zero_dt/reverse/outside/waypoint UNREACHED codes 22/24/25/26/27; 11 Lua and 13 Python healthy single-field consumed-input/receipt faults refuse before output/child; 5 Python zero-step pair faults',
                  reuse=audit(), maps={n: dict(path=str(E / ('zs4-inputmap-' + n + '-20261111.json')), sha256=sha((E / ('zs4-inputmap-' + n + '-20261111.json')).read_bytes())) for n in ['PRE', 'POST']},
                  scope='Schema4 zero-step diagnostic contract under lead216, UNREVIEWED. No runtime/C/default/Windows/native replay, no root, no cutover/all10/GenericNative1 closure. Native reachability of a displacing terrain-absent candidate UNKNOWN.')
    dump(E / 'D3-ZERO-STEP-CONTRACT-FROZEN-20261111.json', frozen); dump(P / 'D3-ZERO-STEP-CONTRACT-FROZEN-20261111.json', frozen); put(P / '.gitattributes', b'* -text\n')
    put(P / 'README.md', (E / 'zs4-README.md').read_bytes()); print(json.dumps(dict(source_rows=frozen['source_rows'], fingerprint=frozen['source_fingerprint'], cases=len(cases)))); sys.exit()
if mode == 'stage':
    for n in j(E / 'zs4-expected-changes-20261111.json'): git('add', n)
    git('add', '-A', P.relative_to(W).as_posix()); files = [dict(path=p.relative_to(P).as_posix(), sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()]
    dump(P / 'manifest.json', dict(files=files, self_excluded=True)); git('add', '-A', P.relative_to(W).as_posix()); print('manifest_entries', len(files)); sys.exit()
assert mode == 'join'
code = git('rev-parse', 'HEAD').decode().strip(); post = j(E / 'zs4-inputmap-POST-20261111.json'); paths = sorted(post['files'])
raw = subprocess.run(['git', '-C', str(W), 'cat-file', '--batch'], input=''.join(code + ':' + n + '\n' for n in paths).encode(), capture_output=True, check=True).stdout; offset = 0; qualified = []
for n in paths:
    end = raw.index(b'\n', offset); h = raw[offset:end].decode().split(); assert h[1] == 'blob'; size = int(h[2]); offset = end + 1; blob = raw[offset:offset + size]; offset += size + 1
    current = (W / n).read_bytes(); assert sha(current) == post['files'][n]['raw_sha256']
    if current != blob: qualified.append(n)
assert offset == len(raw) and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
    if p.is_file(): assert git('cat-file', 'blob', code + ':' + p.relative_to(W).as_posix()) == p.read_bytes()
dump(P / 'code-publication.json', dict(CODE=code, compiled_runtime=RUNTIME, source_rows=len(paths), raw_equals_Git=len(paths) - len(qualified), inherited_raw_filter_qualifications=qualified,
                                       source_fingerprint=post['fingerprint_of_raw_sha256_map'], post_POST_metadata=['STATUS.md', 'docs/handover/ACTIVE-EXTRACTION.json', 'manifest.json', 'code-publication.json', 'publication-manifest.json'],
                                       scope='UNREVIEWED schema4 zero-step contract; no binary relabel/native/cutover/all10 closure.'))
files = [dict(path=p.relative_to(P).as_posix(), sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()]
dump(P / 'publication-manifest.json', dict(files=files, self_excluded=True, original_manifest_retained=True)); git('add', '-A', P.relative_to(W).as_posix())
print(json.dumps(dict(CODE=code, publication_manifest_entries=len(files), packet_files=len(files) + 1, raw_qualified=len(qualified))))
