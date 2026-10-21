#!/usr/bin/env python3
# Shows that the scripts embedded in the M5 DCS call map, and the tools/m5-call-map-check.py
# that runs them, fail where they should. Each case runs the real scripts extracted from the
# document against the snapshot the document describes:
#   unreviewed-send   a dynamic message send not on the reviewed list is OUTSTANDING in ledger.py
#   comment-line      a '#' line in sources.txt is skipped, as eech-engine-sys/build/main.rs:73 does
#   missing-entry     reach.py names a missing entry point instead of raising a traceback
#   tampered-figure   a changed figure in a copy of the document makes the check exit 1
#   missing-snapshot  an absent snapshot commit is a setup error (exit 2), not a mismatch
#   malformed-report  a successful script with no JSON is a mismatch, not a traceback
#   blocked-extract   an unwritable extraction path is a setup error, not a traceback
#
# Usage: tools/m5-call-map-check-selftest.py        (takes a few minutes: it runs the scripts)
import importlib.util, json, os, subprocess, sys, tempfile

sys.dont_write_bytecode = True

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location('m5_call_map_check', os.path.join(here, 'm5-call-map-check.py'))
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)
check.REPO = check.Path(check.git('rev-parse', '--show-toplevel').strip())
doc_path = check.REPO / 'eech-campaign' / 'docs' / 'm5-dcs-call-map.md'
doc = doc_path.read_text(encoding='utf-8')
failures = []


def expect(case, ok, detail=''):
    print(f"{'ok  ' if ok else 'FAIL'} {case}{': ' + detail if detail and not ok else ''}")
    if not ok:
        failures.append(case)


with tempfile.TemporaryDirectory() as scripts, tempfile.TemporaryDirectory() as tree:
    check.extract_scripts(doc, scripts)
    check.materialise_snapshot(check.SNAPSHOT, tree)

    check.Path(scripts, 'malformed.py').write_text("print('not JSON')\n", encoding='utf-8')
    try:
        check.run(scripts, 'malformed.py', cwd=tree)
        malformed_failure = False
    except check.Failure as e:
        malformed_failure = 'malformed.py' in str(e) and 'no valid JSON' in str(e)
    expect('malformed-report: the check names the script and invalid report', malformed_failure)

    def script(name, *args):
        return subprocess.run([sys.executable, '-I', os.path.join(scripts, name), *args], cwd=tree,
                              capture_output=True, text=True, encoding='utf-8')

    def detail(name):
        return os.path.join(scripts, name + '.json')

    for name in ('reach', 'dispatch', 'globals'):
        r = script(name + '.py', tree, '--detail', detail(name))
        if r.returncode:
            sys.exit(f'the unmodified snapshot does not run {name}.py:\n{r.stderr}')
    baseline_globals = open(detail('globals'), encoding='utf-8').read()
    ledger_args = [detail('reach'), detail('dispatch'), detail('globals')]
    base = script('ledger.py', *ledger_args)
    expect('baseline ledger has no outstanding item', base.returncode == 0 and 'OUTSTANDING' not in base.stdout,
           base.stdout[-300:] + base.stderr)

    # unreviewed-send: one extra non-literal send site, on a copy of dispatch.py's output
    fake = 'aphavoc/source/entity/special/group/gp_msgs.c:99999'
    dispatch = json.load(open(detail('dispatch'), encoding='utf-8'))
    dispatch['dynamic'].append([fake, 'C', 'message'])
    mutated = os.path.join(scripts, 'dispatch-extra-send.json')
    json.dump(dispatch, open(mutated, 'w', encoding='utf-8'))
    r = script('ledger.py', detail('reach'), mutated, detail('globals'))
    outstanding = [l for l in r.stdout.splitlines() if l.startswith('OUTSTANDING')]
    expect('unreviewed-send: the extra site is outstanding, and only it',
           len(outstanding) == 1 and 'dynamic' in outstanding[0] and fake in outstanding[0], r.stdout[-300:] + r.stderr)

    # comment-line: globals.py must read the same files with a comment line added to sources.txt
    sources = os.path.join(tree, 'eech-campaign', 'crates', 'eech-engine-sys', 'sources.txt')
    original = open(sources, encoding='utf-8', newline='').read()
    open(sources, 'w', encoding='utf-8', newline='').write('# modules/not_a_compiled_file.c\n' + original)
    r = script('globals.py', tree, '--detail', os.path.join(scripts, 'globals-comment.json'))
    same = r.returncode == 0 and open(os.path.join(scripts, 'globals-comment.json'), encoding='utf-8').read() == baseline_globals
    expect('comment-line: globals.py output is unchanged', same, r.stderr[-300:])
    open(sources, 'w', encoding='utf-8', newline='').write(original)

    # missing-entry: one ENTRIES function renamed in csrc/eech_engine.c
    engine = os.path.join(tree, 'eech-campaign', 'crates', 'eech-engine-sys', 'csrc', 'eech_engine.c')
    text = open(engine, encoding='utf-8', newline='').read()
    open(engine, 'w', encoding='utf-8', newline='').write(text.replace('eech_engine_clock', 'eech_engine_clock_renamed'))
    r = script('reach.py', tree)
    expect('missing-entry: reach.py exits non-zero naming the entry point, with no traceback',
           r.returncode != 0 and 'entry points not found' in r.stderr and 'eech_engine_clock' in r.stderr
           and 'Traceback' not in r.stderr, r.stderr[-300:])

# tampered-figure: the check, run on a copy of the document that claims one more mapped item
with tempfile.TemporaryDirectory() as d:
    tampered = os.path.join(d, 'tampered.md')
    assert '671 are mapped' in doc
    open(tampered, 'w', encoding='utf-8', newline='').write(doc.replace('671 are mapped', '672 are mapped', 1))
    r = subprocess.run([sys.executable, '-I', os.path.join(here, 'm5-call-map-check.py'), '--doc', tampered],
                       capture_output=True, text=True, encoding='utf-8', cwd=check.REPO)
    expect('tampered-figure: the check exits 1 and reports expected against actual',
           r.returncode == 1 and 'FAIL ledger items' in r.stdout and '672' in r.stdout and '671' in r.stdout,
           f'exit {r.returncode}\n{r.stdout[-400:]}{r.stderr[-300:]}')

# missing-snapshot: a commit that is not in the clone
r = subprocess.run([sys.executable, '-I', os.path.join(here, 'm5-call-map-check.py'), '--snapshot', '0' * 40],
                   capture_output=True, text=True, encoding='utf-8', cwd=check.REPO)
expect('missing-snapshot: the check exits 2 and says to fetch the commit',
       r.returncode == 2 and 'git fetch' in r.stderr, f'exit {r.returncode}\n{r.stderr[-300:]}')

with tempfile.TemporaryDirectory() as d:
    blocked = os.path.join(d, 'file-not-directory')
    open(blocked, 'w', encoding='utf-8').close()
    r = subprocess.run([sys.executable, '-I', os.path.join(here, 'm5-call-map-check.py'), '--extract', blocked],
                       capture_output=True, text=True, encoding='utf-8', cwd=check.REPO)
    expect('blocked-extract: the check exits 2 and names the path without a traceback',
           r.returncode == 2 and blocked in r.stderr and 'Traceback' not in r.stderr,
           f'exit {r.returncode}\n{r.stderr[-300:]}')

print(f"FAIL: {len(failures)} of 8 cases failed" if failures else 'PASS: all 8 cases hold')
sys.exit(1 if failures else 0)
