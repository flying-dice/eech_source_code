#!/usr/bin/env python3
# Shows that tools/baseline-governance.py enforces what it claims, on the test
# fixture regression/fixtures/governance/: NOT an EECH campaign baseline and not
# behavioural evidence; its approvals come from a local test approver, never
# from fd-starscream-bot.
#
# In a temporary git repository it walks one approved baseline transition:
#   v1 current -> a red report -> a proposal (UNCLASSIFIED) -> classified with
#   evidence (APPROVAL REQUIRED; adopt refused) -> committed -> the test approver
#   approves that commit -> APPROVED -> adopt: v2 current, v1 kept
# and then checks that each broken variant is refused or detected:
#   no-classification  the adopted record loses its classification
#   no-approval        the adopted record loses its approval reference
#   no-previous        the superseded v1 is removed
#   tampered           the record is edited after the approval
#   wrong-reviewer     the approval names another reviewer than the test approver
#   withdrawn          the approver later requested changes
#   overwritten        the working copy is overwritten outside the governance path
#   invented-class     a classification outside the programme's four
#   regression         a regression is refused and leaves v1 current
#   variation          an approved expected variation founds its own lineage and leaves v1 current
#
# Usage: tools/baseline-governance-selftest.py [--keep <dir>]   (--keep: leave the fixture repository there)
import importlib.util, json, os, shutil, subprocess, sys, tempfile

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURE = os.path.join(HERE, 'regression', 'fixtures', 'governance')
spec = importlib.util.spec_from_file_location('baseline_governance', os.path.join(HERE, 'tools', 'baseline-governance.py'))
gov = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gov)
FILE = 'fixture_metrics.json'
RECORD = 'regression/changes/fixture-change.json'


def git(root, *args):
    r = subprocess.run(['git', '-C', root, '-c', 'user.name=governance fixture', '-c', 'user.email=fixture@invalid', *args], capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(r.stderr)
    return r.stdout.strip()


def load(root, rel):
    return json.load(open(os.path.join(root, rel)))


def save(root, rel, data):
    gov.write_json(os.path.join(root, rel), data)


def refused(fn):
    try:
        fn()
    except SystemExit as e:
        return str(e)
    return None


def build(root):
    """v1 of a fixture lineage, committed."""
    os.makedirs(os.path.join(root, 'regression/baselines/fixture/v1'))
    os.makedirs(os.path.join(root, 'regression/fixture-working'))
    os.makedirs(os.path.join(root, 'regression/fixture-reports'))
    for d in ('regression/baselines/fixture/v1', 'regression/fixture-working'):
        shutil.copyfile(os.path.join(FIXTURE, 'v1', FILE), os.path.join(root, d, FILE))
    shutil.copyfile(os.path.join(FIXTURE, 'report-before.json'), os.path.join(root, 'regression/fixture-reports/report-before.json'))
    save(root, gov.LINEAGES, {'about': 'governance test fixture: not an EECH campaign baseline', 'lineages': {'fixture': {
        'fixture': True, 'configuration': 'test fixture', 'inputs': {}, 'working_path': 'regression/fixture-working', 'current': 'v1',
        'versions': [{'id': 'v1', 'path': 'regression/baselines/fixture/v1',
                      'files': {FILE: gov.committed_sha256(os.path.join(FIXTURE, 'v1', FILE))}, 'established': {'at': 'the fixture'}}]}}})
    git(root, 'init', '-q')
    git(root, 'add', '-A')
    git(root, 'commit', '-q', '-m', 'fixture v1')


def classify(root, cls='intentional-semantic-change', **extra):
    rec = load(root, RECORD)
    c = json.load(open(os.path.join(FIXTURE, 'classification.json')))
    for k in ('title', 'semantic_change', 'claim_changes', 'residual_risks'):
        rec[k] = c[k]
    for d in rec['deltas']:
        d.update(c['delta'], classification=cls)
    rec['classification'] = cls
    rec.update(extra)
    save(root, RECORD, rec)


def approve(root, reviewer=gov.FIXTURE_REVIEWER, later=None):
    """The test approver approves the current commit, and the record references that review."""
    git(root, 'add', '-A')
    git(root, 'commit', '-q', '-m', 'fixture proposal')
    commit = git(root, 'rev-parse', 'HEAD')
    reviews = [{'id': 1, 'reviewer': reviewer, 'state': 'APPROVED', 'commit': commit, 'submitted_at': '2026-01-01T00:00:00Z'}]
    if later:
        reviews.append({'id': 2, 'reviewer': reviewer, 'state': later, 'commit': commit, 'submitted_at': '2026-01-02T00:00:00Z'})
    save(root, 'fixture-approvals.json', reviews)
    rec = load(root, RECORD)
    rec['approval'] = {'pr': 1, 'review_id': 1, 'commit': commit}
    save(root, RECORD, rec)


def state(root):
    g = gov.Governance(root)
    return g.state(g.record('fixture-change'))[0]


def main():
    keep = sys.argv[sys.argv.index('--keep') + 1] if '--keep' in sys.argv else None
    base = tempfile.mkdtemp(prefix='governance-fixture-')
    results = []

    def expect(name, ok, detail):
        results.append(ok)
        print(f"{name:18} {'as expected' if ok else 'NOT AS EXPECTED'}: {detail}")

    try:
        root = os.path.join(base, 'approved')
        os.makedirs(root)
        build(root)
        report = os.path.join(root, 'regression/fixture-reports/report-before.json')
        gov.Governance(root).propose(report, 'fixture-change', os.path.join(FIXTURE, 'candidate'), 'fixture')
        s0 = state(root)
        classify(root)
        s1 = state(root)
        no_adopt = refused(lambda: gov.Governance(root).adopt('fixture-change'))
        before = gov.Governance(root).disposition(json.load(open(report)))['text']
        snapshot = os.path.join(base, 'classified')
        shutil.copytree(root, snapshot)
        approve(root)
        s2 = state(root)
        approved = os.path.join(base, 'approved-not-adopted')
        shutil.copytree(root, approved)
        made = gov.Governance(root).adopt('fixture-change')
        git(root, 'add', '-A')
        git(root, 'commit', '-q', '-m', 'fixture adoption')
        g = gov.Governance(root)
        lin = g.lineages['fixture']
        v1_kept = os.path.exists(os.path.join(root, 'regression/baselines/fixture/v1', FILE))
        working = load(root, f'regression/fixture-working/{FILE}')['value']
        after = g.disposition(json.load(open(report)))['text']
        ok = (s0 == gov.UNCLASSIFIED and s1 == gov.APPROVAL_REQUIRED and no_adopt and s2 == gov.APPROVED and made['version'] == 'v2'
              and lin['current'] == 'v2' and [v['id'] for v in lin['versions']] == ['v1', 'v2'] and v1_kept and working == 2 and not g.check())
        expect('approved path', ok, f'proposal {s0}; classified {s1} (adopt refused: {bool(no_adopt)}); approved {s2}; adopted: '
               f"{lin['current']} current, versions {[v['id'] for v in lin['versions']]}, v1 kept {v1_kept}, working value {working}")
        print(f'  before approval: {before}')
        print(f'  after adoption:  {after}')
        print('  the lineage: ' + json.dumps(lin, indent=None))

        def variant(name, mutate, want):
            d = os.path.join(base, name)
            shutil.copytree(root, d)
            mutate(d)
            problems = gov.Governance(d).check()
            hit = [p for p in problems if want in p]
            expect(name, bool(hit), hit[0] if hit else f'not detected; problems: {problems}')

        def edit(d, fn):
            rec = load(d, RECORD)
            fn(rec)
            save(d, RECORD, rec)

        variant('no-classification', lambda d: edit(d, lambda r: [r.update(classification=None)] + [x.update(classification=None) for x in r['deltas']]),
                'it produced a baseline, but it is UNCLASSIFIED')
        variant('no-approval', lambda d: edit(d, lambda r: r.update(approval=None)), 'it produced a baseline, but it is CLASSIFIED - APPROVAL REQUIRED')
        variant('no-previous', lambda d: shutil.rmtree(os.path.join(d, 'regression/baselines/fixture/v1')), 'v1: regression/baselines/fixture/v1')
        variant('tampered', lambda d: edit(d, lambda r: r['deltas'][0].update(interpretation='edited after the approval')), 'differs from the version approved')
        variant('wrong-reviewer', lambda d: save(d, 'fixture-approvals.json', [dict(load(d, 'fixture-approvals.json')[0], reviewer=gov.LEAD)]),
                f'is by {gov.LEAD}, not {gov.FIXTURE_REVIEWER}')
        variant('withdrawn', lambda d: save(d, 'fixture-approvals.json', load(d, 'fixture-approvals.json') + [
            {'id': 2, 'reviewer': gov.FIXTURE_REVIEWER, 'state': 'CHANGES_REQUESTED', 'commit': 'x', 'submitted_at': '2026-01-02T00:00:00Z'}]),
                'followed by a request for changes')
        variant('overwritten', lambda d: shutil.copyfile(os.path.join(FIXTURE, 'v1', FILE), os.path.join(d, f'regression/fixture-working/{FILE}')),
                'is not the current version v2')
        variant('invented-class', lambda d: edit(d, lambda r: r.update(classification='benign-drift')), 'benign-drift is not a programme classification')

        # a regression is refused, and v1 stays current
        d = os.path.join(base, 'regression')
        shutil.copytree(snapshot, d)
        classify(d, 'regression')
        why = refused(lambda: gov.Governance(d).adopt('fixture-change'))
        g = gov.Governance(d)
        expect('regression', bool(why) and g.lineages['fixture']['current'] == 'v1' and load(d, f'regression/fixture-working/{FILE}')['value'] == 1 and not g.check(),
               f"{why}; current {g.lineages['fixture']['current']}; {g.disposition(json.load(open(os.path.join(d, 'regression/fixture-reports/report-before.json'))))['text']}")

        # an approved expected variation founds its own lineage and never replaces v1
        d = os.path.join(base, 'variation')
        shutil.copytree(snapshot, d)
        classify(d, 'expected-input-world-variation', variation={'changed': 'the fixture input set (a stand-in for a different installation)',
                                                                 'new_lineage': 'fixture-other-inputs', 'configuration': 'test fixture, other inputs'})
        approve(d)
        made = gov.Governance(d).adopt('fixture-change')
        g = gov.Governance(d)
        expect('variation', made['lineage'] == 'fixture-other-inputs' and g.lineages['fixture']['current'] == 'v1'
               and load(d, f'regression/fixture-working/{FILE}')['value'] == 1 and not g.check(),
               f"founded {made['lineage']} {made['version']}; fixture still {g.lineages['fixture']['current']} (value "
               f"{load(d, f'regression/fixture-working/{FILE}')['value']})")
    finally:
        if keep:
            shutil.copytree(base, keep, dirs_exist_ok=True)
        shutil.rmtree(base, ignore_errors=True)
    print(f"governance fixture: {sum(results)} of {len(results)} as expected")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
