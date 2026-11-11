"""Schema4 candidate-parity correction: unchanged-input reuse proof and small sibling packet. Modes: reuse | freeze | stage.
No build, Docker, Cargo, native or Windows action; the zero-step-contract packet is never touched."""
from pathlib import Path
import datetime, hashlib, json, re, subprocess, sys
E = Path(__file__).resolve().parent
R = E.parent
W = R / 'collision-characterization-20261029'
Z = R / 'zs4-20261111'
P = W / 'eech-campaign/reference/d3-entity-collision/zero-step-parity-correction'
OLD = W / 'eech-campaign/reference/d3-entity-collision/zero-step-contract'
BASE = '0e174c1a09a9d1f74c159ffe59378b77ce43e93a'; CODE = '594ef61368e2c5a9cbe6ed9f27cfdbad51ac7a94'; RUNTIME = '2fcf69f9b8972b4353687dc2ca1eaaac780836f3'
REVIEW = ('independent-zero-step-0e174-source-20261111.json', '0e61b0dabc819362aaeeefcc599590d4c4a6fc4a29abb66835e20d9d90ce4dd5')
BINDING = ('zero-step-0e174-Root-binding-20261111.json', '2cdc0dc34ea4bc8b2c0ad6c17f6443ba90e0e949092d20c94a7bc12c59235f39')
CHANGED = ['STATUS.md', 'docs/handover/ACTIVE-EXTRACTION.json', 'eech-campaign/docs/d3-public-session-finalization.md', 'eech-campaign/tools/normal_fire_capture.py', 'eech-campaign/tools/test_normal_fire_capture.py']
READERS = ['run-zs4p-direct-20261111.py', 'zs4p-parity-20261111.py', 'zs4p-proof-20261111.py']
RUNS = ['r1-old-validator-RED', 'g1-python-GREEN', 'g2-py-compile', 'g3-parity']
GUARDS = ['p-g1-fix-blocker', 'p-g2-characterize']
REUSED = ['s2-fmt', 's3-clippy', 's4-session-lua', 's5-facade-default', 's6-facade-observer']
sha = lambda b: hashlib.sha256(b).hexdigest()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*a): return subprocess.check_output(['git', '-C', str(W), *a])
def j(p): return json.loads(p.read_bytes())
def put(p, b):
    with p.open('xb') as f: f.write(b)
def dump(p, o): put(p, (json.dumps(o, indent=2) + '\n').encode())


def reuse():
    assert git('rev-parse', 'HEAD').decode().strip() == BASE
    for name, digest in (REVIEW, BINDING): assert sha((R / name).read_bytes()) == digest, name
    binding = j(R / BINDING[0]); assert binding['META'] == BASE and binding['CODE'] == CODE and binding['compiled_runtime'] == RUNTIME
    names = sorted(set(git('diff', '--name-only', BASE).decode().splitlines()) | set(git('ls-files', '--others', '--exclude-standard').decode().splitlines()))
    outside = [n for n in names if not n.startswith(P.relative_to(W).as_posix() + '/')]
    assert outside == sorted(CHANGED), outside
    post = j(Z / 'zs4-inputmap-POST-20261111.json'); assert post['head_sha'] == '23ec563cc48b16bf90c6de061ee13efc109475dd' and post['fingerprint_of_raw_sha256_map'] == binding['source_fingerprint']
    current = {n: sha((W / n).read_bytes()) for n in post['files']}
    assert all(current[n] == r['raw_sha256'] for n, r in post['files'].items()), 'a census input changed'
    # Every census row's Git blob at BASE equals its blob at CODE (the receipts' commit): BASE added only packet metadata.
    tree = lambda rev: {k: v for k, v in ((l.split('\t', 1)[1], l.split()[2]) for l in git('ls-tree', '-r', rev).decode().splitlines()) if k in post['files']}
    assert tree(BASE) == tree(CODE) and len(tree(BASE)) == len(post['files']), 'census blobs differ between receipt commit and base'
    extras_post = j(Z / 'zs4-extra-POST-20261111.json'); diff = []
    for n, h in extras_post.items():
        for root in (W, Z, R):
            if (root / n).exists(): cur = sha((root / n).read_bytes()); break
        if cur != h: diff.append(n)
    assert sorted(diff) == sorted(n for n in CHANGED if n in extras_post), diff  # only the listed changed files among the 38 extras
    receipts = {}
    for label in REUSED:
        r = j(OLD / 'runs' / (label + '.json')); assert r['state'] == 'terminal' and r['actual_exit'] == 0
        receipts[label] = dict(actual_exit=0, pid=r['pid'], started_utc=r['started_utc'], completed_utc=r['completed_utc'], record_sha256=sha((OLD / 'runs' / (label + '.json')).read_bytes()), command=r['command'])
    return dict(base=BASE, receipt_code=CODE, compiled_runtime=RUNTIME, census_rows=len(current), census_rows_raw_equal_0e174_POST=len(current), census_fingerprint=post['fingerprint_of_raw_sha256_map'],
                census_blobs_equal_CODE_and_BASE=True, extras_total=len(extras_post), extras_changed=sorted(diff), changed_paths=sorted(CHANGED),
                reused_not_rerun=receipts, reused_scope='Lua/Rust tests/fixtures/compiled product rows/cfg(test) seam/producer configs byte-equal to the 0e174 inputs those receipts consumed; Python s1 is superseded by the new g1 run.')


mode = sys.argv[1]
if mode == 'reuse':
    out = E / 'zs4p-reuse-20261111.json'; assert not out.exists(); dump(out, reuse()); print(out.read_text()); sys.exit()
if mode == 'freeze':
    assert not P.exists() and reuse() == j(E / 'zs4p-reuse-20261111.json')
    red = j(E / 'zs4p-r1-old-validator-RED.json'); assert red['actual_exit'] == 1
    redlog = (E / 'zs4p-r1-old-validator-RED.stderr.log').read_text()
    fails = re.findall(r"^FAIL: test_schema4_candidate_receipt_only_mismatch_refused_before_child .*\(field='(\w+)', report='(\w+)'\)$", redlog.replace('\r', ''), re.M)
    assert len(fails) == 9 and len(set(fails)) == 9 and redlog.count('AssertionError: ValueError not raised') == 9, fails
    assert 'Ran 16 tests' in redlog and 'FAILED (failures=9)' in redlog
    green = (E / 'zs4p-g1-python-GREEN.stderr.log').read_text(); assert j(E / 'zs4p-g1-python-GREEN.json')['actual_exit'] == 0 and 'Ran 16 tests' in green and green.rstrip().endswith('OK') and 'candidate_receipt_only_refusals=9' in green
    assert j(E / 'zs4p-g2-py-compile.json')['actual_exit'] == 0 and j(E / 'zs4p-g3-parity.json')['actual_exit'] == 0
    parity = json.loads((E / 'zs4p-g3-parity.stdout.log').read_text()); assert parity['verdict'] == 'PARITY'
    P.mkdir(); (P / 'readers').mkdir(); (P / 'runs').mkdir(); (P / 'external').mkdir()
    for n in READERS: put(P / 'readers' / n, (E / n).read_bytes())
    for label in RUNS:
        for ext in ['json', 'stdout.log', 'stderr.log']: put(P / 'runs' / (label + '.' + ext), (E / ('zs4p-' + label + '.' + ext)).read_bytes())
    for g in GUARDS:
        for ext in ['stdout.log', 'stderr.log', 'exit']: put(P / 'runs' / (g + '.' + ext), (E / (g + '.' + ext)).read_bytes())
    for n in ['zs4p-reuse-attempt1-FAILED-cmdline-too-long.console.log', 'zs4p-reuse-attempt2-FAILED-unpatched-cmdline-too-long.console.log', 'zs4p-reuse-attempt3-FAILED-extras-expectation.console.log','zs4p-reuse.console.log', 'zs4p-parity.stdout.log', 'zs4p-parity.stderr.log', 'zs4p-parity.exit', 'zs4p-g2-pycache-removed.txt', 'zs4p-reuse-20261111.json']: put(P / 'runs' / n, (E / n).read_bytes())
    for n in [REVIEW[0], BINDING[0], 'D3-REQUIREMENTS.json']: put(P / 'external' / n, (R / n).read_bytes())
    frozen = dict(utc=now(), finding='ENV-PUBLIC-SESSION-SCHEMA4-CANDIDATE-PARITY-1', source_base=BASE, review=dict(path=REVIEW[0], sha256=REVIEW[1]), Root_binding=dict(path=BINDING[0], sha256=BINDING[1]),
                  changed_files={n: sha((W / n).read_bytes()) for n in CHANGED}, readers={n: sha((E / n).read_bytes()) for n in READERS},
                  RED=dict(run='r1-old-validator-RED', actual_exit=1, validator_sha256_equal_0e174='308e6bd525f189fbdf3099288a1d05f14f724175f32beb0afffdd0ece3152a20', accepted=sorted('%s/%s' % f for f in fails)),
                  GREEN=dict(python_tests=16, actual_exit=0, candidate_only_refusals=9, py_compile_exit=0, parity_exit=0),
                  parity=parity, unjoined_reason=dict(mode="literal 'move' checked in both validators; selection has no mode", test_request='checked through the actual TEST request domain in both validators; no selection counterpart'),
                  reuse=j(E / 'zs4p-reuse-20261111.json'), all10=dict(path='external/D3-REQUIREMENTS.json', sha256=sha((R / 'D3-REQUIREMENTS.json').read_bytes()), status='all ten D3 requirements unchanged and OPEN'),
                  cache='No cache leased; no Docker/Cargo/native/default/Windows/retail run.',
                  scope='Python validator/test/doc/card/STATUS correction only, UNREVIEWED until the SAME reviewer closes the finding. zero-step-contract packet unchanged.')
    dump(P / 'D3-ZERO-STEP-PARITY-CORRECTION-FROZEN-20261111.json', frozen); dump(E / 'D3-ZERO-STEP-PARITY-CORRECTION-FROZEN-20261111.json', frozen)
    put(P / '.gitattributes', b'* -text\n'); put(P / 'README.md', (E / 'zs4p-README.md').read_bytes()); print(json.dumps(dict(packet=str(P), changed=len(CHANGED)))); sys.exit()
assert mode == 'stage'
for n in CHANGED: git('add', n)
git('add', '-A', P.relative_to(W).as_posix()); files = [dict(path=p.relative_to(P).as_posix(), sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()]
dump(P / 'manifest.json', dict(files=files, self_excluded=True)); git('add', '-A', P.relative_to(W).as_posix())
assert git('diff', '--cached', '--name-only', '--', OLD.relative_to(W).as_posix()) == b''
print('manifest_entries', len(files))
