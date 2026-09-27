#!/usr/bin/env python3
# Accepted-baseline governance (docs/governance/REVIEW.md, "Baseline changes";
# docs/m4-baseline-governance.md).
#
# A red regression pack produces a change proposal, never a new baseline. A
# change record, regression/changes/<id>.json, identifies the candidate and the
# baseline version it was judged against, cites the pack report, and classifies
# every failed claim and check with the evidence for it, as one of the
# programme's classes (AGENT.md):
#
#   regression                      rejected: never advances a baseline; the candidate stays failing
#   approved-defect-correction      may advance its lineage to a new version, only with a verified
#   intentional-semantic-change     formal APPROVED review by fd-starscream-bot of a commit holding
#                                   this exact record (bound by the record's proposal digest)
#   expected-input-world-variation  never replaces the equivalent-input baseline: with the same
#                                   approval it founds a separate lineage
#
# A difference never classifies itself: every classification carries evidence.
# Approval is not a field anyone can set: the record references a GitHub
# review, which is fetched and checked (reviewer, state, reviewed commit, and
# the record's content at that commit).
#
# Baselines (regression/baselines/lineages.json) are versioned and never
# overwritten. Adopting an approved change adds a version, keeps every earlier
# one, and materialises the new current version at the lineage's working path
# (what tools/regress-windows.ps1 reads).
#
# Usage:
#   tools/baseline-governance.py check                          lineages and change records consistent
#   tools/baseline-governance.py status --report <report.json>  a regression-pack report's disposition
#   tools/baseline-governance.py propose --report <report.json> --id <id> [--candidate <dir>]
#                                                               a record skeleton: every failure UNCLASSIFIED
#   tools/baseline-governance.py adopt --change <id>            advance a baseline by an approved record
#   --root <dir>: the eech-campaign directory (default: this checkout). Test fixtures (fixture lineages,
#   approved by a local test approver, never by fd-starscream-bot) are refused inside this checkout.
import argparse, hashlib, json, os, shutil, subprocess, sys

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LEAD = 'fd-starscream-bot'
REPOSITORY = 'flying-dice/eech_source_code'
CLASSES = ('regression', 'approved-defect-correction', 'intentional-semantic-change', 'expected-input-world-variation')
NEEDS_APPROVAL = CLASSES[1:]
ADVANCES = ('approved-defect-correction', 'intentional-semantic-change')
FIXTURE_REVIEWER = 'governance-fixture-approver (test only; not fd-starscream-bot)'
LINEAGES = 'regression/baselines/lineages.json'
CHANGES = 'regression/changes'
# set after approval; everything else in a record is what the approval covers
POST_APPROVAL = ('approval', 'resulting_baselines')
REQUIRED = ('id', 'title', 'baseline_under_test', 'candidate', 'scenarios', 'inputs', 'report_before', 'deltas',
            'classification', 'proposed', 'previous_baseline', 'claim_changes', 'residual_risks', 'approval', 'resulting_baselines')

UNCLASSIFIED = 'UNCLASSIFIED'
APPROVAL_REQUIRED = 'CLASSIFIED - APPROVAL REQUIRED'
APPROVED = 'CLASSIFIED AND APPROVED'
REJECTED = 'REGRESSION - REJECTED'


def committed_sha256(path):
    """The SHA-256 of the committed content: text with LF line endings, .gz and .acmi as they are."""
    data = open(path, 'rb').read()
    if not path.endswith(('.gz', '.acmi')):
        data = data.replace(b'\r\n', b'\n')
    return hashlib.sha256(data).hexdigest()


def digest(record):
    """What an approval covers: the record without the fields set after it."""
    core = {k: v for k, v in record.items() if k not in POST_APPROVAL and not k.startswith('_')}
    return hashlib.sha256(json.dumps(core, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def failures(report):
    """The failed claims and checks of a regression-pack report."""
    claims = {c['id']: c for c in report.get('claims', []) if str(c.get('status', '')).startswith('FAIL')}
    checks = sorted(k for k, v in report.get('checks', {}).items() if v.get('status') == 'FAIL')
    return claims, checks


class GitHubApprover:
    """A formal review on the programme's repository, fetched from GitHub."""
    reviewer = LEAD

    def review(self, pr, review_id):
        # one JSON line per review, over every page; review bodies are UTF-8 whatever the console's code page
        r = subprocess.run(['gh', 'api', '--paginate', f'repos/{REPOSITORY}/pulls/{pr}/reviews',
                            '--jq', '.[] | {id, reviewer: .user.login, state, commit: .commit_id, submitted_at} | @json'],
                           capture_output=True, text=True, encoding='utf-8')
        if r.returncode:
            raise LookupError(f'cannot read the reviews of PR #{pr}: {r.stderr.strip()[:200]}')
        reviews = [json.loads(line) for line in r.stdout.splitlines() if line.strip()]
        return pick(reviews, review_id, lambda v: (v['reviewer'], v['state'], v['commit'], v.get('submitted_at') or ''))


class FixtureApprover:
    """The governance fixture's test approver: a local file, never a GitHub review."""
    reviewer = FIXTURE_REVIEWER

    def __init__(self, root):
        self.path = os.path.join(root, 'fixture-approvals.json')

    def review(self, pr, review_id):
        if not os.path.exists(self.path):
            raise LookupError('the fixture has no fixture-approvals.json')
        return pick(json.load(open(self.path)), review_id, lambda v: (v['reviewer'], v['state'], v['commit'], v['submitted_at']))


def pick(reviews, review_id, fields):
    found = [v for v in reviews if v.get('id') == review_id]
    if not found:
        raise LookupError(f'no review {review_id}')
    who, state, commit, when = fields(found[0])
    # a later request for changes by the same reviewer withdraws the approval
    later = [fields(v) for v in reviews if v is not found[0] and fields(v)[0] == who and fields(v)[3] > when]
    return {'reviewer': who, 'state': state, 'commit': commit, 'withdrawn': any(v[1] == 'CHANGES_REQUESTED' for v in later)}


class Governance:
    def __init__(self, root=HERE):
        self.root = os.path.abspath(root)
        self.data = json.load(open(self.path(LINEAGES)))
        self.lineages = self.data['lineages']
        self.fixture = any(l.get('fixture') for l in self.lineages.values())
        self.approver = FixtureApprover(self.root) if self.fixture else GitHubApprover()
        self._verified = {}

    def path(self, rel):
        return os.path.join(self.root, rel)

    def records(self):
        folder = self.path(CHANGES)
        names = sorted(n for n in os.listdir(folder) if n.endswith('.json')) if os.path.isdir(folder) else []
        return [json.load(open(os.path.join(folder, n))) | {'_file': n} for n in names]

    def record(self, change_id):
        path = self.path(f'{CHANGES}/{change_id}.json')
        if not os.path.exists(path):
            sys.exit(f'no change record {CHANGES}/{change_id}.json')
        return json.load(open(path)) | {'_file': change_id + '.json'}

    def git(self, *args):
        return subprocess.run(['git', '-C', self.root, *args], capture_output=True)

    # -- a record ----------------------------------------------------------

    def verify_approval(self, rec):
        """(ok, text): the referenced review approved a commit holding this exact record."""
        key = (rec['id'], digest(rec), json.dumps(rec.get('approval'), sort_keys=True))
        if key in self._verified:
            return self._verified[key]
        a = rec.get('approval')
        if not isinstance(a, dict) or not all(a.get(k) for k in ('pr', 'review_id', 'commit')):
            result = (False, 'no approval reference (pr, review_id, commit)')
        else:
            try:
                review = self.approver.review(a['pr'], a['review_id'])
            except LookupError as e:
                review, result = None, (False, f'approval cannot be verified: {e}')
            if review:
                prefix = self.git('rev-parse', '--show-prefix').stdout.decode().strip()
                shown = self.git('show', f"{a['commit']}:{prefix}{CHANGES}/{rec['id']}.json")
                if review['reviewer'] != self.approver.reviewer:
                    result = (False, f"review {a['review_id']} is by {review['reviewer']}, not {self.approver.reviewer}")
                elif review['state'] != 'APPROVED':
                    result = (False, f"review {a['review_id']} is {review['state']}, not APPROVED")
                elif review['withdrawn']:
                    result = (False, f"review {a['review_id']} was followed by a request for changes")
                elif review['commit'] != a['commit']:
                    result = (False, f"review {a['review_id']} approved {review['commit'][:8]}, not {a['commit'][:8]}")
                elif shown.returncode:
                    result = (False, f"the record is not in the approved commit {a['commit'][:8]}")
                elif digest(json.loads(shown.stdout)) != digest(rec):
                    result = (False, f"the record differs from the version approved at {a['commit'][:8]}")
                else:
                    result = (True, f"approved by {review['reviewer']}: PR #{a['pr']}, review {a['review_id']}, commit {a['commit'][:8]}")
        self._verified[key] = result
        return result

    def state(self, rec, report=None):
        """(state, problems): the record's disposition, and what is wrong with it."""
        problems = []
        missing = [k for k in REQUIRED if k not in rec]
        if missing:
            return UNCLASSIFIED, [f"{rec.get('id', rec['_file'])}: missing {', '.join(missing)}"]
        rid = rec['id']
        if rec['_file'] != rid + '.json':
            problems.append(f'{rid}: the file is {rec["_file"]}')
        rb = rec['report_before'] or {}
        path = self.path(rb.get('path', ''))
        if not rb.get('path') or not os.path.exists(path):
            problems.append(f"{rid}: the report before the change ({rb.get('path')}) is missing")
        elif committed_sha256(path) != rb.get('sha256'):
            problems.append(f"{rid}: {rb['path']} differs from its recorded hash")
        elif report is None:
            report = json.load(open(path))
        for d in rec['deltas']:
            for e in d.get('evidence') or []:
                if e.get('path') and not os.path.exists(self.path(e['path'])):
                    problems.append(f"{rid}: evidence {e['path']} is missing")
                elif e.get('path') and e.get('sha256') and committed_sha256(self.path(e['path'])) != e['sha256']:
                    problems.append(f"{rid}: evidence {e['path']} differs from its recorded hash")
        cls = rec['classification']
        invented = sorted({c for c in [cls] + [d.get('classification') for d in rec['deltas']] if c is not None and c not in CLASSES})
        if invented:
            problems.append(f"{rid}: {', '.join(invented)} is not a programme classification ({', '.join(CLASSES)})")

        # classified: every failure covered by a delta, every delta classified with evidence
        complete = cls in CLASSES and not invented
        if report is not None:
            claims, checks = failures(report)
            covered_claims = {c for d in rec['deltas'] for c in d.get('claims', [])}
            covered_checks = {c for d in rec['deltas'] for c in d.get('checks', [])}
            if set(claims) - covered_claims or set(checks) - covered_checks:
                complete = False
        for d in rec['deltas']:
            if d.get('classification') not in CLASSES or not d.get('evidence') or not d.get('interpretation'):
                complete = False
            elif cls in CLASSES and d['classification'] != cls:
                problems.append(f"{rid}: a delta classified {d['classification']} under a record classified {cls}")
        defect = rec.get('defect') or {}
        if complete and cls == 'approved-defect-correction' and not (defect.get('corrected') and defect.get('retired_behaviour')):
            problems.append(f'{rid}: an approved defect correction names the corrected defect and the retired behaviour (defect)')
        if complete and cls == 'intentional-semantic-change' and not ((rec.get('semantic_change') or {}).get('rule') and rec['claim_changes']):
            problems.append(f'{rid}: an intentional semantic change states the changed rule (semantic_change.rule) and the claims it changes (claim_changes)')
        if complete and cls == 'expected-input-world-variation' and not (rec.get('variation') or {}).get('changed'):
            problems.append(f'{rid}: an expected variation names what input, world or configuration changed (variation.changed)')
        if complete and cls in ADVANCES and not (rec['proposed'] or {}).get('files'):
            problems.append(f'{rid}: a change that advances a baseline carries the proposed baseline (proposed)')

        if not complete:
            state = UNCLASSIFIED
        elif cls == 'regression':
            state = REJECTED
            if rec['approval'] or rec['resulting_baselines']:
                problems.append(f'{rid}: a regression is rejected: it has no approval to advance and no resulting baseline')
        else:
            ok, why = self.verify_approval(rec)
            state = APPROVED if ok else APPROVAL_REQUIRED
            if rec['approval'] and not ok:
                problems.append(f'{rid}: {why}')
        if rec['resulting_baselines'] and state != APPROVED:
            problems.append(f'{rid}: it produced a baseline, but it is {state}')
        return state, problems

    # -- the lineages ------------------------------------------------------

    def check(self):
        """Problems with the lineages and every change record; [] when consistent."""
        problems = []
        if self.fixture and os.path.realpath(self.root) == os.path.realpath(HERE):
            problems.append('a fixture lineage is in the programme\'s lineages: fixtures are approved by a test approver only')
        records = {r['id']: r for r in self.records() if 'id' in r}
        states = {}
        for r in self.records():
            state, found = self.state(r)
            states[r.get('id')] = state
            problems += found
        for name, lin in self.lineages.items():
            ids = [v['id'] for v in lin['versions']]
            if len(set(ids)) != len(ids) or lin.get('current') not in ids:
                problems.append(f'{name}: current {lin.get("current")} is not one of its versions {ids}')
                continue
            for i, v in enumerate(lin['versions']):
                for f, sha in v['files'].items():
                    p = self.path(f"{v['path']}/{f}")
                    if not os.path.exists(p):
                        problems.append(f"{name} {v['id']}: {v['path']}/{f} is missing (versions are never removed)")
                    elif committed_sha256(p) != sha:
                        problems.append(f"{name} {v['id']}: {v['path']}/{f} differs from its recorded hash (versions are never changed)")
                change = v.get('change') or v.get('founded_by')
                if i == 0 and not (v.get('established') or v.get('founded_by')):
                    problems.append(f"{name} {v['id']}: the first version says how it was established")
                if i > 0 and (v.get('supersedes') != lin['versions'][i - 1]['id'] or not v.get('change')):
                    problems.append(f"{name} {v['id']}: a later version supersedes the one before it, by a change record")
                if change:
                    r = records.get(change)
                    if not r:
                        problems.append(f"{name} {v['id']}: its change record {change} is missing")
                        continue
                    wanted = {'lineage': name, 'version': v['id'], 'files': v['files']}
                    if states.get(change) != APPROVED:
                        problems.append(f"{name} {v['id']}: its change {change} is {states.get(change)}, not {APPROVED}")
                    if wanted not in r['resulting_baselines']:
                        problems.append(f"{name} {v['id']}: change {change} does not record producing it")
                    if v.get('change') and r['previous_baseline'] != {'lineage': name, 'version': v['supersedes'], 'path': lin['versions'][i - 1]['path']}:
                        problems.append(f"{name} {v['id']}: change {change} was judged against another baseline than {v['supersedes']}")
                    if v.get('change') and r['classification'] not in ADVANCES:
                        problems.append(f"{name} {v['id']}: a {r['classification']} cannot advance a lineage")
            current = next(v for v in lin['versions'] if v['id'] == lin['current'])
            if lin.get('working_path'):
                for f, sha in current['files'].items():
                    p = self.path(f"{lin['working_path']}/{f}")
                    if not os.path.exists(p) or committed_sha256(p) != sha:
                        problems.append(f"{name}: {lin['working_path']}/{f} is not the current version {current['id']}: "
                                        'a baseline changes only through an approved change record (tools/baseline-governance.py adopt)')
        for r in records.values():
            for b in r.get('resulting_baselines') or []:
                lin = self.lineages.get(b.get('lineage'), {})
                if b not in [{'lineage': b.get('lineage'), 'version': v['id'], 'files': v['files']} for v in lin.get('versions', [])]:
                    problems.append(f"{r['id']}: its resulting baseline {b.get('lineage')} {b.get('version')} is not in the lineages")
        return problems

    def describe(self, name):
        lin = self.lineages[name]
        cur = lin['current']
        history = [v for v in lin['versions'] if v.get('change')]
        return f"{name} {cur}" + (f" (since change {history[-1]['change']}, superseding {history[-1]['supersedes']})" if history and history[-1]['id'] == cur
                                  else f" (established {lin['versions'][0].get('established', {}).get('at', '')}; unchanged)")

    # -- a pack report's disposition ----------------------------------------

    def disposition(self, report):
        """The disposition of a regression-pack report: {'state', 'record', 'text', 'items': {claim or check: text}}."""
        claims, checks = failures(report)
        if not claims and not checks:
            return {'state': 'PASS', 'record': None, 'text': '', 'items': {}}
        build = (report.get('head', {}).get('build') or {}).get('files')
        matching = [r for r in self.records() if build and (r.get('candidate') or {}).get('build') == build]
        if not matching:
            items = {k: UNCLASSIFIED for k in list(claims) + checks}
            return {'state': UNCLASSIFIED, 'record': None, 'items': items,
                    'text': f'{UNCLASSIFIED}: no change record for this build (tools/baseline-governance.py propose)'}
        rec = matching[-1]
        state, problems = self.state(rec, report)
        lineage = (rec.get('baseline_under_test') or {}).get('lineage')
        judged = ((report.get('head') or {}).get('baseline') or {}).get('version') or (rec.get('baseline_under_test') or {}).get('version')
        items = {}
        for d in rec['deltas']:
            text = self.item_text(d.get('classification'), state, rec, lineage, judged)
            for k in d.get('claims', []) + d.get('checks', []):
                items[k] = text
        for k in list(claims) + checks:
            items.setdefault(k, UNCLASSIFIED)
        overall = UNCLASSIFIED if UNCLASSIFIED in items.values() or state == UNCLASSIFIED else state
        text = self.item_text(rec['classification'], overall, rec, lineage, judged)
        return {'state': overall, 'record': rec['id'], 'items': items, 'problems': problems,
                'text': f"{UNCLASSIFIED if overall == UNCLASSIFIED else text} (change record {CHANGES}/{rec['id']}.json)"}

    def item_text(self, cls, state, rec, lineage, judged):
        if cls not in CLASSES or state == UNCLASSIFIED:
            return UNCLASSIFIED
        if state == REJECTED:
            return 'CLASSIFIED AS REGRESSION - REJECTED'
        if state == APPROVAL_REQUIRED:
            return f'CLASSIFIED AS {cls} - APPROVAL REQUIRED'
        made = [b for b in rec.get('resulting_baselines') or []]
        if made and cls in ADVANCES:
            cur = self.lineages.get(lineage, {}).get('current')
            return f"against {judged}: approved {cls}; {made[0]['version']} is the accepted baseline" + ('' if cur == made[0]['version'] else f' (now {cur})')
        if made:
            return f"approved {cls}; recorded as lineage {made[0]['lineage']}, {lineage} unchanged"
        return f'CLASSIFIED AS {cls} - APPROVED, not yet adopted'

    # -- changes -------------------------------------------------------------

    def propose(self, report_path, change_id, candidate=None, lineage='windows-retail'):
        report = json.load(open(report_path))
        claims, checks = failures(report)
        if not claims and not checks:
            sys.exit('the report has no failure: there is nothing to propose')
        if os.path.exists(self.path(f'{CHANGES}/{change_id}.json')):
            sys.exit(f'{CHANGES}/{change_id}.json exists')
        folder = self.path(f'{CHANGES}/{change_id}')
        rel = os.path.relpath(os.path.abspath(report_path), self.root).replace(os.sep, '/')
        if rel.startswith('..') or rel.startswith('target/'):
            # keep the report with the record
            os.makedirs(folder, exist_ok=True)
            shutil.copyfile(report_path, os.path.join(folder, 'report-before.json'))
            rel = f'{CHANGES}/{change_id}/report-before.json'
        lin = self.lineages[lineage]
        head = report.get('head', {})
        judged = (head.get('baseline') or {}).get('version') or lin['current']
        version = next(v for v in lin['versions'] if v['id'] == judged)
        proposed = None
        if candidate:
            os.makedirs(os.path.join(folder, 'candidate'), exist_ok=True)
            files = {}
            for f in version['files']:
                shutil.copyfile(os.path.join(candidate, f), os.path.join(folder, 'candidate', f))
                files[f] = committed_sha256(os.path.join(folder, 'candidate', f))
            proposed = {'path': f'{CHANGES}/{change_id}/candidate', 'files': files}
        deltas, placed = [], set()
        for cid, c in claims.items():
            failed = [k for k in c.get('checks', []) if k in checks]
            placed |= set(failed)
            deltas.append({'claims': [cid], 'checks': failed, 'result': c['status'], 'interpretation': None, 'evidence': [], 'classification': None})
        for k in checks:
            if k not in placed:
                deltas.append({'claims': [], 'checks': [k], 'result': report['checks'][k].get('detail', ''), 'interpretation': None, 'evidence': [], 'classification': None})
        build = head.get('build') or {}
        rec = {
            'id': change_id, 'title': None,
            'baseline_under_test': {'lineage': lineage, 'version': judged, 'integration': head.get('pack_revision')},
            'candidate': {'revision': ' / '.join(build.get('build_info', [])), 'path': build.get('path'), 'build': build.get('files')},
            'scenarios': sorted(json.load(open(self.path('regression/pack.json')))['scenarios']) if os.path.exists(self.path('regression/pack.json')) else [],
            'inputs': {k: v.get('detail') for k, v in report.get('checks', {}).items() if k.startswith('inputs.')},
            'report_before': {'path': rel, 'sha256': committed_sha256(self.path(rel)), 'verdict': head.get('verdict')},
            'deltas': deltas, 'classification': None, 'proposed': proposed,
            'previous_baseline': {'lineage': lineage, 'version': judged, 'path': version['path']},
            'claim_changes': [], 'residual_risks': [], 'approval': None, 'resulting_baselines': [],
        }
        os.makedirs(self.path(CHANGES), exist_ok=True)
        write_json(self.path(f'{CHANGES}/{change_id}.json'), rec)
        return rec

    def adopt(self, change_id):
        """Advance a baseline by an approved record; returns the resulting baseline or exits with the reason."""
        if self.fixture and os.path.realpath(self.root) == os.path.realpath(HERE):
            sys.exit("refused: a fixture lineage is in the programme's lineages")
        rec = self.record(change_id)
        state, problems = self.state(rec)
        cls = rec.get('classification')
        if cls == 'regression':
            sys.exit(f'refused: {change_id} is classified as a regression, which never advances a baseline; the candidate stays failing')
        if state != APPROVED:
            why = self.verify_approval(rec)[1] if state == APPROVAL_REQUIRED else 'not every failure is classified with evidence'
            sys.exit(f'refused: {change_id} is {state}: {why}')
        if problems:
            sys.exit(f'refused: ' + '; '.join(problems))
        if rec['resulting_baselines']:
            sys.exit(f'refused: {change_id} has already produced {rec["resulting_baselines"]}')
        prev = rec['previous_baseline']
        lin = self.lineages[prev['lineage']]
        if cls in ADVANCES:
            if lin['current'] != prev['version']:
                sys.exit(f"refused: {change_id} was judged against {prev['version']}, but {lin['current']} is current")
            name, new = prev['lineage'], f"v{len(lin['versions']) + 1}"
            entry = {'id': new, 'path': f'regression/baselines/{name}/{new}', 'files': rec['proposed']['files'],
                     'supersedes': prev['version'], 'change': change_id}
        else:
            name = (rec.get('variation') or {}).get('new_lineage')
            if not name or name in self.lineages:
                sys.exit(f'refused: an expected variation founds a new lineage (variation.new_lineage), and never replaces {prev["lineage"]}')
            new = 'v1'
            entry = {'id': new, 'path': f'regression/baselines/{name}/{new}', 'files': rec['proposed']['files'], 'founded_by': change_id}
        source = self.path(rec['proposed']['path'])
        os.makedirs(self.path(entry['path']), exist_ok=True)
        for f, sha in entry['files'].items():
            data = open(os.path.join(source, f), 'rb').read().replace(b'\r\n', b'\n')
            if hashlib.sha256(data).hexdigest() != sha:
                sys.exit(f"refused: the proposed {f} differs from the approved hash")
            open(self.path(f"{entry['path']}/{f}"), 'wb').write(data)
        if cls in ADVANCES:
            lin['versions'].append(entry)
            lin['current'] = new
            if lin.get('working_path'):
                for f in entry['files']:
                    shutil.copyfile(self.path(f"{entry['path']}/{f}"), self.path(f"{lin['working_path']}/{f}"))
        else:
            self.lineages[name] = {'configuration': rec['variation'].get('configuration', ''), 'inputs': rec['variation'].get('inputs', {}),
                                   'working_path': None, 'current': new, 'versions': [entry]}
        rec['resulting_baselines'] = [{'lineage': name, 'version': new, 'files': entry['files']}]
        write_json(self.path(LINEAGES), self.data)
        write_json(self.path(f'{CHANGES}/{change_id}.json'), {k: v for k, v in rec.items() if k != '_file'})
        return rec['resulting_baselines'][0]


def write_json(path, data):
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        json.dump(data, f, indent=1, ensure_ascii=False)
        f.write('\n')


def main():
    p = argparse.ArgumentParser(description='accepted-baseline governance')
    p.add_argument('--root', default=HERE)
    sub = p.add_subparsers(dest='command', required=True)
    sub.add_parser('check')
    s = sub.add_parser('status')
    s.add_argument('--report', required=True)
    s = sub.add_parser('propose')
    s.add_argument('--report', required=True)
    s.add_argument('--id', required=True)
    s.add_argument('--candidate')
    s.add_argument('--lineage', default='windows-retail')
    s = sub.add_parser('adopt')
    s.add_argument('--change', required=True)
    a = p.parse_args()
    g = Governance(a.root)
    if a.command == 'check':
        problems = g.check()
        for name in g.lineages:
            print(f'lineage {g.describe(name)}: versions ' + ', '.join(v['id'] for v in g.lineages[name]['versions']))
        for r in g.records():
            print(f"change {r.get('id')}: {r.get('classification')}: {g.state(r)[0]}")
        for problem in problems:
            print(f'PROBLEM: {problem}')
        print('baseline governance: ' + ('consistent' if not problems else f'{len(problems)} problems'))
        return 1 if problems else 0
    if a.command == 'status':
        d = g.disposition(json.load(open(a.report)))
        for k, text in d['items'].items():
            print(f'{k}: {text}')
        for problem in d.get('problems', []):
            print(f'PROBLEM: {problem}')
        print(f"disposition: {d['text'] or d['state']}")
        return 0
    if a.command == 'propose':
        rec = g.propose(a.report, a.id, a.candidate, a.lineage)
        print(f"{CHANGES}/{a.id}.json: {len(rec['deltas'])} deltas, all {UNCLASSIFIED}: classify each with its evidence")
        return 0
    if a.command == 'adopt':
        made = g.adopt(a.change)
        print(f"adopted {a.change}: {made['lineage']} {made['version']} is current; every earlier version is kept")
        return 0


if __name__ == '__main__':
    sys.exit(main())
