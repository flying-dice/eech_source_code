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
#   unknown-table     an unreviewed other-table form is OUTSTANDING; the reviewed data arrays stay excluded
#   duplicate-ledger  a repeated ledger row, or a repeated embedded block, is rejected
#   wrong-items       a ledger row whose Items is not mapped + excluded + outstanding is rejected
#   invalid-utf8      an undecodable document is a setup error (exit 2), not a traceback
#   missing-items     a ledger.py run that writes no items.json is a named failure, not a traceback
#   missing-block     a document without one embedded script is a mismatch naming the script
#   script-exit       a script's nonzero exit is a mismatch carrying its stderr
#   non-object-json   a script report that is a JSON list, not an object, is a mismatch
#   missing-field     a script report without a field the check reads is a mismatch naming it
#   unknown-target    an unknown merge target is a setup error (exit 2)
#   divergent-target  a merge target that changes aphavoc/ or modules/ prints DOES NOT HOLD (exit 0)
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
cases = 0
checker = os.path.join(here, 'm5-call-map-check.py')


def expect(case, ok, detail=''):
    global cases
    cases += 1
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

    # unknown-table: an unreviewed other-table form is outstanding; the reviewed false positives stay excluded
    dispatch = json.load(open(detail('dispatch'), encoding='utf-8'))
    dispatch['other_tables']['synthetic_unreviewed_table'] = {'assignments': 1, 'first': 'synthetic.c:1', 'class': 'W'}
    mutated = os.path.join(scripts, 'dispatch-extra-table.json')
    json.dump(dispatch, open(mutated, 'w', encoding='utf-8'))
    items_path = os.path.join(scripts, 'items-extra-table.json')
    r = script('ledger.py', detail('reach'), mutated, detail('globals'), '--items', items_path)
    tables = {i['item']: i['disposition'] for i in json.load(open(items_path, encoding='utf-8')) if i['ledger'] == 'other tables'}
    outstanding = [l for l in r.stdout.splitlines() if l.startswith('OUTSTANDING')]
    expect('unknown-table: the unreviewed table is outstanding, and only it',
           len(outstanding) == 1 and outstanding[0].endswith('synthetic_unreviewed_table')
           and tables.pop('synthetic_unreviewed_table') == 'outstanding', r.stdout[-300:] + r.stderr)
    expect('unknown-table: the six reviewed other-table forms stay excluded',
           len(tables) == 6 and set(tables.values()) == {'excluded'}, json.dumps(tables))

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



def run_check(*args, doc_text=None, env=None):
    """the checker's command line, on a copy of the document when doc_text is given"""
    with tempfile.TemporaryDirectory() as d:
        argv = list(args)
        if doc_text is not None:
            path = os.path.join(d, 'doc.md')
            open(path, 'wb').write(doc_text if isinstance(doc_text, bytes) else doc_text.encode('utf-8'))
            argv += ['--doc', path]
        return subprocess.run([sys.executable, '-I', checker, *argv], capture_output=True, text=True,
                              encoding='utf-8', cwd=check.REPO, env=env)


def with_scripts(text, replacements):
    """the document with the named embedded scripts' code replaced"""
    return check.BLOCK.sub(lambda m: m.group(0).replace(m.group(2), replacements[m.group(1)])
                           if m.group(1) in replacements else m.group(0), text)


def no_traceback(r):
    return 'Traceback' not in r.stderr


def why(r):
    return f'exit {r.returncode}\n{r.stdout[-400:]}{r.stderr[-300:]}'


# duplicate-ledger / wrong-items: the full command line, on copies of the document
total = next(l for l in doc.splitlines() if l.startswith('| **Total** |'))
assert all(f'**{n}**' in total for n in ('1,126', '671', '455'))
r = run_check(doc_text=doc.replace(total, total.replace('**671**', '**999**').replace('**455**', '**127**') + '\n' + total, 1))
expect('duplicate-ledger: the check rejects a repeated Total row and names it',
       r.returncode == 1 and 'more than one row labelled **Total**' in r.stderr and no_traceback(r), why(r))
r = run_check(doc_text=doc + '\n\n<summary><code>ledger.py</code></summary>\n\n```python\nprint(1)\n```\n')
expect('duplicate-ledger: the check rejects a repeated embedded block and names it',
       r.returncode == 1 and 'more than one block for ledger.py' in r.stderr and no_traceback(r), why(r))
r = run_check(doc_text=doc.replace(total, total.replace('**1,126**', '**1,127**'), 1))
expect('wrong-items: the check rejects an Items cell that is not mapped + excluded + outstanding',
       r.returncode == 1 and 'states 1127 items' in r.stderr and 'is 1126' in r.stderr and no_traceback(r), why(r))

# setup and format errors at the command line
r = run_check(doc_text=b'\xff\xfe not UTF-8')
expect('invalid-utf8: the check exits 2 and names the document, without a traceback',
       r.returncode == 2 and 'as UTF-8' in r.stderr and no_traceback(r), why(r))
r = run_check(doc_text=doc.replace('<summary><code>ledger.py</code></summary>', '<summary>ledger.py</summary>', 1))
expect('missing-block: the check exits 1 and names the script with no block',
       r.returncode == 1 and 'no block for ledger.py' in r.stderr and no_traceback(r), why(r))
r = run_check('no-such-ref-for-selftest')
expect('unknown-target: the check exits 2 and names the target',
       r.returncode == 2 and 'no-such-ref-for-selftest' in r.stderr and no_traceback(r), why(r))

# missing-items / missing-field: every script replaced by a stub, so only the checker's own handling is exercised
stubbed = with_scripts(doc, {n: 'print("{}")' for n in ('inventory.py', 'reach.py', 'dispatch.py', 'globals.py')})
r = run_check(doc_text=with_scripts(stubbed, {'ledger.py': 'print("{}")'}))
expect('missing-items: the check exits 1 naming ledger.py and items.json, without a traceback',
       r.returncode == 1 and 'ledger.py' in r.stderr and 'items.json' in r.stderr and no_traceback(r), why(r))
writes_items = 'import sys; open(sys.argv[sys.argv.index("--items") + 1], "w").write("[]"); print("{}")'
r = run_check(doc_text=with_scripts(stubbed, {'ledger.py': writes_items}))
expect('missing-field: the check exits 1 naming the missing report field, without a traceback',
       r.returncode == 1 and 'missing or invalid field' in r.stderr and 'items by rule kind' in r.stderr
       and no_traceback(r), why(r))

# script-exit / non-object-json: run() is the checker's boundary to every script
with tempfile.TemporaryDirectory() as d:
    for name, code in (('exits.py', 'import sys; sys.stderr.write("boom"); sys.exit(3)'), ('list.py', "print('[{\"a\": 1}]')")):
        check.Path(d, name).write_text(code + '\n', encoding='utf-8')
        try:
            check.run(d, name, cwd=d)
            message = ''
        except check.Failure as e:
            message = str(e)
        if name == 'exits.py':
            expect('script-exit: a nonzero exit is a mismatch naming the script, its status and stderr',
                   'exits.py exited 3' in message and 'boom' in message, message)
        else:
            expect('non-object-json: a JSON list is a mismatch, not an accepted report',
                   'list.py' in message and 'no valid JSON report' in message, message)


# divergent-target: commits made in a throw-away object directory that reads the clone's objects, so the
# clone gains nothing. The target adds one file under aphavoc/ (or only under modules/) to the snapshot.
def git_in(env, *args, stdin=None):
    out = subprocess.run(['git', *args], capture_output=True, text=True, cwd=check.REPO, env=env, input=stdin)
    if out.returncode:
        sys.exit(f'git {" ".join(args)} failed: {out.stderr}')
    return out.stdout.strip()


with tempfile.TemporaryDirectory() as objects:
    clone_objects = os.path.abspath(os.path.join(check.REPO, git_in(os.environ, 'rev-parse', '--git-path', 'objects')))
    env = dict(os.environ, GIT_OBJECT_DIRECTORY=objects, GIT_INDEX_FILE=os.path.join(objects, 'index'),
               GIT_ALTERNATE_OBJECT_DIRECTORIES=clone_objects, GIT_AUTHOR_NAME='selftest',
               GIT_AUTHOR_EMAIL='selftest@invalid', GIT_COMMITTER_NAME='selftest', GIT_COMMITTER_EMAIL='selftest@invalid')
    blob = git_in(env, 'hash-object', '-w', '--stdin', stdin='int divergence;\n')
    for case, path in (('aphavoc/', 'aphavoc/zz_selftest_divergence.c'),
                       ('modules/ only', 'modules/zz_selftest_divergence.c')):
        git_in(env, 'read-tree', check.SNAPSHOT)
        git_in(env, 'update-index', '--add', '--cacheinfo', f'100644,{blob},{path}')
        commit = git_in(env, 'commit-tree', git_in(env, 'write-tree'), '-p', check.SNAPSHOT, '-m', 'selftest divergence')
        r = run_check(commit, env=env)
        expect(f'divergent-target ({case}): the check prints DOES NOT HOLD, naming the file, and keeps exit 0',
               r.returncode == 0 and 'merge target: DOES NOT HOLD' in r.stdout and f'first {path}' in r.stdout
               and '1 files differ' in r.stdout and no_traceback(r), why(r))

print(f"FAIL: {len(failures)} of {cases} cases failed" if failures else f'PASS: all {cases} cases hold')
sys.exit(1 if failures else 0)
