#!/usr/bin/env python3
# The accepted M1-M3 regression pack (docs/m4-regression-pack.md): runs the
# existing checks against a build and the retail roots, and reports each
# accepted behaviour listed in regression/pack.json as one of
#
#   PASS - directly protected          a check asserting it passed
#   PASS - baseline-sensitive          the retained baseline or artifact it is judged by is reproduced
#   FAIL - directly protected          a check asserting it failed
#   FAIL - differs from the baseline   the evidence differs; which behaviour changed is not identified
#   EVIDENCE ONLY - not automatically checked
#   NOT COVERED
#   NOT RUN                            its check needs the runtime tier
#
# EVIDENCE ONLY and NOT COVERED never become PASS: they come from the
# inventory, not from a run. When a run fails, each failure also gets its
# disposition from the change records (tools/baseline-governance.py):
# UNCLASSIFIED, CLASSIFIED - APPROVAL REQUIRED, CLASSIFIED AND APPROVED, or
# REGRESSION - REJECTED. A disposition explains a difference; it never turns a
# failure into a pass. Two tiers:
#
#   retained  no retail data, about a minute: every retained artifact and
#             checker has its recorded hash, and each input manifest its
#             recorded identity; the checkers reproduce the retained reports
#             from the retained evidence; their self-tests still detect their
#             injected discrepancies; the accepted checks still reject the S3
#             mutant's retained metrics (#68); the baseline lineages and change
#             records are consistent, the working baselines are the current
#             accepted version, and the governance fixture's self-test passes.
#             Any failure here stops the pack:
#             it cannot judge a build against inconsistent evidence.
#   runtime   the build and the retail roots, about 20 minutes: the roots
#             against their manifests; tools/lifecycle-windows.ps1; then, in
#             parallel, tools/regress-windows.ps1 -Exact and
#             tools/reference-windows.ps1 -ObserveSupply; then
#             tools/loss-chain-check.py on the reference run, and the fresh
#             evidence against the retained.
#
# Usage:
#   tools/regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <second root> [--bin <dir>] [--out <dir>]
#   tools/regression-pack.py --retained-only [--out <dir>]
#   roots: tools/retail-map3-installs.sh (Georgia) and tools/retail-cvh.sh (Lebanon, two separately assembled)
#   --bin: the build under test with its lua/ (default target/windows); --out: default target/regression-pack
# Writes report.md, report.json and the runs' outputs to --out.
# Exit 0 when nothing fails and both tiers ran (--retained-only: when the retained tier passes).
import argparse, datetime, gzip, hashlib, importlib.util, json, os, shutil, subprocess, sys, zipfile
from concurrent.futures import ThreadPoolExecutor

sys.dont_write_bytecode = True  # campaign-expectations.py is imported: leave no __pycache__ in tools/

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PACK = os.path.join(HERE, 'regression', 'pack.json')
PY = sys.executable
PATH_KEYS = ('recording', 'observations', 'metrics')

# every check a claim can name
CHECKS = {
    'evidence.artifacts': 'every retained artifact exists with its recorded hash (and decompressed content hash)',
    'evidence.checkers': 'every checker and runner has its recorded hash',
    'evidence.manifests': 'each input manifest has its recorded file count and combined hash',
    'evidence.reproduce.m2_check': 'observation-check.py on the retained M2 evidence reproduces reference/lebanon-3h/check.json',
    'evidence.reproduce.m3_check': 'observation-check.py on the retained M3 observations reproduces reference/m3-resupply-loop/check.json',
    'evidence.reproduce.chains': 'supply-chain-check.py on the retained M3 observations reproduces reference/m3-resupply-loop/chains.json',
    'evidence.reproduce.loss': 'loss-chain-check.py on the retained M3 observations reproduces reference/m3-supply-loss/loss-chain.json',
    'sensitivity.observation_check': 'observation-check-selftest.py on the retained M2 evidence: five injected discrepancies all fail',
    'sensitivity.supply_chain_check': 'supply-chain-check-selftest.py on the retained M3 observations: four removed links all break the chain',
    'sensitivity.loss_chain_check': 'loss-chain-check-selftest.py on the retained M3 observations: three removed links all fail',
    'sensitivity.s3_lebanon_expectation': "campaign-expectations.py rejects the S3 mutant's retained Lebanon metrics: 'airbases resupplied with ammo' fails",
    'sensitivity.s3_exact': "regress-compare.py --exact rejects the S3 mutant's retained metrics for both scenarios",
    'blindspot.georgia_resupply': "Georgia's expectations still all hold on the S3 mutant's retained metrics (the blind spot is as documented)",
    'governance.lineage': 'the baseline lineages and change records are consistent (baseline-governance.py check), and pack.json names the current version',
    'governance.fixture': 'baseline-governance-selftest.py: the approved transition works and each broken variant is refused or detected',
    'inputs.georgia': 'the Georgia root matches reference/m4-s3-mutation/inputs-georgia.json',
    'inputs.lebanon': 'the Lebanon root matches reference/lebanon-3h/inputs.json',
    'inputs.lebanon_repeat': 'the second, separately assembled Lebanon root matches it too',
    'lifecycle.boot_twice': 'tools/lifecycle-windows.ps1 case boot_twice',
    'lifecycle.bad_arguments': 'tools/lifecycle-windows.ps1 case bad_arguments',
    'lifecycle.missing_root': 'tools/lifecycle-windows.ps1 case missing_root',
    'lifecycle.missing_campaign': 'tools/lifecycle-windows.ps1 case missing_campaign',
    'lifecycle.no_reuse': 'tools/lifecycle-windows.ps1 case no_reuse',
    'lifecycle.missing_map': 'tools/lifecycle-windows.ps1 case missing_map (known defect, unchanged)',
    'regress.georgia.expectations': 'every Georgia campaign expectation holds (campaign-expectations.py)',
    'regress.georgia.exact': 'Georgia metrics IDENTICAL to regression/windows/georgia_retail.json (regress-compare.py --exact)',
    'regress.lebanon.expectations.mechanics': 'every Lebanon campaign expectation other than airbase resupply holds',
    'regress.lebanon.expectations.resupply': "Lebanon's 'airbases resupplied with ammo' and '... with fuel' hold",
    'regress.lebanon.exact': 'Lebanon metrics IDENTICAL to regression/windows/lebanon_retail.json',
    'reference.observation_check': 'observation-check.py passes on both reference runs',
    'reference.repeatable': 'the two reference runs give byte-identical recording, observations and metrics',
    'reference.baseline': "the observed run's metrics are IDENTICAL to the Lebanon baseline and every expectation holds (the runner's own check)",
    'reference.observed_equals_unobserved': "the observed run's metrics are IDENTICAL to the same build's unobserved Lebanon regression run",
    'reference.supply_chains': 'supply-chain-check.py passes on both reference runs (a complete chain; deliveries agree with the metrics)',
    'reference.loss_chain': "loss-chain-check.py passes on the reference run's first SUPPLY aircraft destroyed in flight",
    'retained.recording': "the reference run's recording is byte-identical to reference/lebanon-3h/recording.zip.acmi",
    'retained.observations': "the reference run's observations are byte-identical to reference/m3-resupply-loop/observations.jsonl.gz",
    'retained.check_report': "the reference run's observation-check report equals reference/m3-resupply-loop/check.json",
    'retained.chains_report': "the reference run's chain report equals reference/m3-resupply-loop/chains.json",
    'retained.loss_report': "the reference run's loss-chain report equals reference/m3-supply-loss/loss-chain.json",
}
RUNTIME = ('inputs.', 'lifecycle.', 'regress.', 'reference.', 'retained.')
STATUS = {
    ('direct', 'PASS'): 'PASS - directly protected', ('baseline-sensitive', 'PASS'): 'PASS - baseline-sensitive',
    ('direct', 'FAIL'): 'FAIL - directly protected', ('baseline-sensitive', 'FAIL'): 'FAIL - differs from the baseline',
    ('direct', 'NOT RUN'): 'NOT RUN', ('baseline-sensitive', 'NOT RUN'): 'NOT RUN',
    ('evidence-only', None): 'EVIDENCE ONLY - not automatically checked', ('not-covered', None): 'NOT COVERED',
}


def committed_sha256(path):
    """The SHA-256 of the committed content: text with LF line endings, .gz and .acmi as they are."""
    data = open(path, 'rb').read()
    if not path.endswith(('.gz', '.acmi')):
        data = data.replace(b'\r\n', b'\n')
    return hashlib.sha256(data).hexdigest()


def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest()


def run(args, log, cwd=HERE):
    with open(log, 'w', encoding='utf-8', newline='\n') as f:
        return subprocess.run(args, stdout=f, stderr=subprocess.STDOUT, cwd=cwd).returncode


def tail(log, n=1):
    lines = [l.rstrip() for l in open(log, encoding='utf-8', errors='replace') if l.strip()]
    return ' / '.join(lines[-n:]) if lines else '(no output)'


def shown(path):
    return os.path.relpath(path, HERE).replace(os.sep, '/')


def report_json(path):
    return {k: v for k, v in json.load(open(path)).items() if k not in PATH_KEYS}


def same_report(new, old):
    """A checker's report equal to the retained one, apart from the input paths it names."""
    if not os.path.exists(new):
        return 'FAIL', f'{shown(new)} was not written'
    a, b = report_json(new), report_json(old)
    if a == b:
        return 'PASS', f'equal to {shown(old)}'
    keys = sorted(k for k in set(a) | set(b) if a.get(k) != b.get(k))
    return 'FAIL', f"differs from {shown(old)} in: {', '.join(keys)}{summarise(a, b)}"


def summarise(a, b):
    out = []
    for k in ('deliveries', 'complete_chains', 'failures', 'frames'):
        if k in a and a.get(k) != b.get(k):
            out.append(f'{k} {b.get(k)} -> {a.get(k)}')
    if 'loss' in a and a.get('loss') != b.get('loss'):
        la, lb = a.get('loss') or {}, b.get('loss') or {}
        out.append(f"loss of {lb.get('aircraft')} at {lb.get('t')} s -> {la.get('aircraft')} at {la.get('t')} s")
    if 'time' in a and a['time'] != b.get('time'):
        out.append(f"session clock drift {b['time'].get('session_clock_drift_s')} -> {a['time'].get('session_clock_drift_s')} s")
    return ' (' + '; '.join(out) + ')' if out else ''


def governance(root=HERE):
    spec = importlib.util.spec_from_file_location('baseline_governance', os.path.join(HERE, 'tools', 'baseline-governance.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.Governance(root)


def expectations_module():
    spec = importlib.util.spec_from_file_location('campaign_expectations', os.path.join(HERE, 'tools', 'campaign-expectations.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expectations(metrics_path, scenario):
    """campaign-expectations.py's own check: [(ok, name, detail)]."""
    return expectations_module().check(json.load(open(metrics_path)), scenario)


def verdict_of(results, names=None, exclude=None):
    chosen = [r for r in results if (names is None or r[1] in names) and not (exclude and r[1].startswith(exclude))]
    failed = [f'{n} ({d})' for ok, n, d in chosen if not ok]
    return ('FAIL', 'failed: ' + '; '.join(failed)) if failed else ('PASS', f'{len(chosen)} of {len(chosen)} hold')


def compare(baseline, metrics, log, exact=True):
    args = [PY, os.path.join(HERE, 'tools', 'regress-compare.py'), baseline, metrics] + (['--exact'] if exact else [])
    code = run(args, log)
    lines = [l.strip() for l in open(log, encoding='utf-8', errors='replace')]
    verdict = next((l for l in lines if 'IDENTICAL' in l or 'tolerance' in l), tail(log))
    outside = sum(1 for l in lines if l.startswith('FAIL '))
    return code, verdict + (f' ({outside} metrics outside tolerance)' if outside else '')


# -- the retained tier ------------------------------------------------------

def retained_tier(pack, out, results):
    tmp = os.path.join(out, 'retained')
    os.makedirs(tmp, exist_ok=True)
    art = {k: os.path.join(HERE, v['path']) for k, v in pack['artifacts'].items()}

    problems = []
    for key, a in pack['artifacts'].items():
        path = art[key]
        if not os.path.exists(path):
            problems.append(f"{a['path']} is missing")
        elif committed_sha256(path) != a['sha256']:
            problems.append(f"{a['path']} has SHA-256 {committed_sha256(path)[:12]}, recorded {a['sha256'][:12]}")
    # the decompressed evidence the checks below use, and its recorded content hash
    content = {'m2.recording': os.path.join(tmp, 'recording.acmi'), 'm2.observations': os.path.join(tmp, 'observations-m2.jsonl'),
               'm3.observations': os.path.join(tmp, 'observations-m3.jsonl')}
    if not problems:
        with open(content['m2.recording'], 'wb') as f:
            f.write(zipfile.ZipFile(art['m2.recording']).read('recording.acmi'))
        for key in ('m2.observations', 'm3.observations'):
            with gzip.open(art[key]) as src, open(content[key], 'wb') as dst:
                shutil.copyfileobj(src, dst)
        for key, path in content.items():
            if sha256(path) != pack['artifacts'][key]['content_sha256']:
                problems.append(f"{pack['artifacts'][key]['path']}'s content has SHA-256 {sha256(path)[:12]}, recorded {pack['artifacts'][key]['content_sha256'][:12]}")
    results['evidence.artifacts'] = ('FAIL', '; '.join(problems)) if problems else ('PASS', f"{len(pack['artifacts'])} artifacts as recorded")

    changed = []
    for tool, c in pack['checkers'].items():
        path = os.path.join(HERE, tool)
        if not os.path.exists(path):
            changed.append(f'{tool} is missing')
        elif committed_sha256(path) != c['sha256']:
            changed.append(f"{tool} differs from its recorded version ({c['revision']})")
    results['evidence.checkers'] = ('FAIL', '; '.join(changed)) if changed else ('PASS', f"{len(pack['checkers'])} checkers and runners as recorded")

    wrong = []
    for name, m in pack['inputs'].items():
        path = os.path.join(HERE, m['manifest'])
        if not os.path.exists(path):
            wrong.append(f"{m['manifest']} is missing")
            continue
        d = json.load(open(path))
        if d.get('files') != m['files'] or d.get('combined_sha256') != m['combined_sha256']:
            wrong.append(f"{m['manifest']}: {d.get('files')} files, {str(d.get('combined_sha256'))[:12]}; recorded {m['files']}, {m['combined_sha256'][:12]}")
    results['evidence.manifests'] = ('FAIL', '; '.join(wrong)) if wrong else ('PASS', ', '.join(f"{n} {m['files']} files {m['combined_sha256'][:12]}" for n, m in pack['inputs'].items()))
    if problems or changed or wrong:
        return False

    tool = lambda name: os.path.join(HERE, 'tools', name)
    jobs = {
        'evidence.reproduce.m2_check': ([PY, tool('observation-check.py'), art['m2.recording'], art['m2.observations'], art['m2.metrics'], '--report', os.path.join(tmp, 'm2-check.json')], ('m2-check.json', 'm2.check')),
        'evidence.reproduce.m3_check': ([PY, tool('observation-check.py'), art['m2.recording'], art['m3.observations'], art['m2.metrics'], '--report', os.path.join(tmp, 'm3-check.json')], ('m3-check.json', 'm3.check')),
        'evidence.reproduce.chains': ([PY, tool('supply-chain-check.py'), art['m3.observations'], art['m2.metrics'], '--report', os.path.join(tmp, 'chains.json')], ('chains.json', 'm3.chains')),
        'evidence.reproduce.loss': ([PY, tool('loss-chain-check.py'), art['m3.observations'], '--report', os.path.join(tmp, 'loss-chain.json')], ('loss-chain.json', 'm3.loss')),
        'sensitivity.observation_check': ([PY, tool('observation-check-selftest.py'), content['m2.recording'], content['m2.observations'], art['m2.metrics']], None),
        'sensitivity.supply_chain_check': ([PY, tool('supply-chain-check-selftest.py'), content['m3.observations'], art['m2.metrics']], None),
        'sensitivity.loss_chain_check': ([PY, tool('loss-chain-check-selftest.py'), art['m3.observations'], '--unit', '87568'], None),
        'governance.fixture': ([PY, tool('baseline-governance-selftest.py')], None),
    }
    with ThreadPoolExecutor(len(jobs)) as pool:
        codes = dict(zip(jobs, pool.map(lambda k: run(jobs[k][0], os.path.join(tmp, k + '.txt')), jobs)))
    for key, (args, reproduce) in jobs.items():
        log = os.path.join(tmp, key + '.txt')
        if reproduce:
            status, detail = same_report(os.path.join(tmp, reproduce[0]), art[reproduce[1]])
            results[key] = (status if codes[key] == 0 else 'FAIL', (f'exit {codes[key]}; ' if codes[key] else '') + detail)
        else:
            lines = [l.strip() for l in open(log, encoding='utf-8', errors='replace') if l.strip()]
            results[key] = ('PASS' if codes[key] == 0 else 'FAIL', tail(log) if key == 'governance.fixture' else f"{len(lines)} cases: " + '; '.join(lines))

    lebanon = expectations(art['m4.mutant_lebanon'], 'lebanon_retail')
    ammo = [ok for ok, name, _ in lebanon if name == 'airbases resupplied with ammo']
    results['sensitivity.s3_lebanon_expectation'] = (('PASS', "'airbases resupplied with ammo' fails on the mutant's metrics, as in #68") if ammo == [False]
                                                    else ('FAIL', "'airbases resupplied with ammo' does not fail on the S3 mutant's metrics"))
    verdicts = []
    for scenario, key in (('georgia_retail', 'm4.mutant_georgia'), ('lebanon_retail', 'm4.mutant_lebanon')):
        code, verdict = compare(art['baseline.' + scenario.split('_')[0]], art[key], os.path.join(tmp, f's3-exact-{scenario}.txt'))
        verdicts.append((scenario, code, verdict))
    results['sensitivity.s3_exact'] = ('PASS' if all(c != 0 for _, c, _ in verdicts) else 'FAIL', '; '.join(f'{s}: {v}' for s, _, v in verdicts))
    g = governance()
    problems = g.check()
    current = g.lineages['windows-retail']
    files = next(v for v in current['versions'] if v['id'] == current['current'])['files']
    for scenario in ('georgia', 'lebanon'):
        if pack['artifacts']['baseline.' + scenario]['sha256'] != files[f'{scenario}_retail.json']:
            problems.append(f"regression/pack.json's baseline.{scenario} is not windows-retail {current['current']}")
    results['governance.lineage'] = (('FAIL', '; '.join(problems)) if problems else
                                     ('PASS', f"{g.describe('windows-retail')}; {len(g.records())} change records consistent"))
    georgia = expectations(art['m4.mutant_georgia'], 'georgia_retail')
    status, detail = verdict_of(georgia)
    results['blindspot.georgia_resupply'] = (('PASS', f'confirmed: {detail} on the S3 mutant') if status == 'PASS'
                                             else ('FAIL', f'CHANGED: Georgia now detects the S3 mutant ({detail}); update regression/pack.json'))
    return all(results[k][0] == 'PASS' for k in results)


# -- the runtime tier -------------------------------------------------------

def shell():
    for name in ('pwsh', 'powershell'):
        path = shutil.which(name)
        if path:
            return path
    sys.exit('PowerShell is needed for the Windows runners')


def build_identity(pack, bin_dir):
    info = {'path': bin_dir, 'files': {}, 'differs_from_accepted': []}
    accepted = dict(pack['accepted']['candidate']['files'], **pack['accepted']['scripts']['files'])
    for name, want in accepted.items():
        path = os.path.join(bin_dir, name)
        got = sha256(path) if os.path.exists(path) else None
        info['files'][name] = got
        if got != want:
            info['differs_from_accepted'].append(name)
    build_info = os.path.join(bin_dir, 'BUILD-INFO.txt')
    info['build_info'] = [l.rstrip() for l in open(build_info)][:3] if os.path.exists(build_info) else ['(no BUILD-INFO.txt)']
    return info


def runtime_tier(pack, a, out, results, identity):
    ps = shell()
    tool = lambda name: os.path.join(HERE, 'tools', name)
    logs = os.path.join(out, 'logs')
    os.makedirs(logs, exist_ok=True)

    # the inputs: each root against its accepted manifest
    roots = {'inputs.georgia': (a.georgia, 'georgia'), 'inputs.lebanon': (a.lebanon, 'lebanon'), 'inputs.lebanon_repeat': (a.lebanon_repeat, 'lebanon')}
    for key, (root, name) in roots.items():
        log = os.path.join(logs, key + '.txt')
        code = run([PY, tool('input-manifest.py'), root, '--compare', os.path.join(HERE, pack['inputs'][name]['manifest'])], log)
        results[key] = ('PASS' if code == 0 else 'FAIL', f'{root}: ' + tail(log, 2 if code == 0 else 3))
    if os.path.realpath(a.lebanon) == os.path.realpath(a.lebanon_repeat):
        results['inputs.lebanon_repeat'] = ('FAIL', 'the second Lebanon root must be a separately assembled root, not the same directory')
    if any(results[k][0] != 'PASS' for k in roots):
        return False

    # lifecycle: seconds, and on the Lebanon root before the long runs use it
    log = os.path.join(logs, 'lifecycle.txt')
    run([ps, '-NoProfile', '-File', tool('lifecycle-windows.ps1'), '-Root', a.lebanon, '-Bin', a.bin, '-Out', os.path.join(out, 'lifecycle')], log)
    lines = [l.strip() for l in open(log, encoding='utf-8', errors='replace')]
    for case in ('boot_twice', 'bad_arguments', 'missing_root', 'missing_campaign', 'no_reuse', 'missing_map'):
        line = next((l for l in lines if l.startswith(case + ': ')), None)
        ok = line in (f'{case}: PASS', f'{case}: KNOWN DEFECT (unchanged)')
        results['lifecycle.' + case] = ('PASS' if ok else 'FAIL', line or 'no result: see logs/lifecycle.txt')

    # the regression gets its own copy of the Lebanon root: no two engines share a root
    copy = os.path.join(out, 'roots', 'lebanon-regress')
    shutil.rmtree(copy, ignore_errors=True)
    shutil.copytree(a.lebanon, copy)
    if run([PY, tool('input-manifest.py'), copy, '--compare', os.path.join(HERE, pack['inputs']['lebanon']['manifest'])], os.path.join(logs, 'inputs.lebanon-copy.txt')) != 0:
        sys.exit('the copy of the Lebanon root differs from its manifest')

    regress, reference = os.path.join(out, 'regress'), os.path.join(out, 'reference')
    print('running the regression (Georgia, Lebanon) and the reference (Lebanon twice) for 3 simulated hours each ...', flush=True)
    with ThreadPoolExecutor(2) as pool:
        r = pool.submit(run, [ps, '-NoProfile', '-File', tool('regress-windows.ps1'), '-Georgia', a.georgia, '-Lebanon', copy,
                              '-Bin', a.bin, '-Out', regress, '-Exact'], os.path.join(logs, 'regress.txt'))
        f = pool.submit(run, [ps, '-NoProfile', '-File', tool('reference-windows.ps1'), '-Root', a.lebanon, '-RepeatRoot', a.lebanon_repeat,
                              '-Bin', a.bin, '-Out', reference, '-ObserveSupply'], os.path.join(logs, 'reference.txt'))
        r.result(), f.result()

    # the regression: expectations by campaign-expectations.py's own check, the exact comparison by the runner
    text = [l.strip() for l in open(os.path.join(logs, 'regress.txt'), encoding='utf-8', errors='replace')]
    for scenario in ('georgia_retail', 'lebanon_retail'):
        s = scenario.split('_')[0]
        metrics = os.path.join(regress, scenario + '.json')
        if not os.path.exists(metrics) or any(l.startswith(f'{scenario} failed') for l in text):
            for key in [k for k in CHECKS if k.startswith(f'regress.{s}.')]:
                results[key] = ('FAIL', 'the run failed or wrote no metrics: see logs/regress.txt')
            continue
        found = expectations(metrics, scenario)
        if s == 'georgia':
            results['regress.georgia.expectations'] = verdict_of(found)
        else:
            results['regress.lebanon.expectations.mechanics'] = verdict_of(found, exclude='airbases resupplied with')
            results['regress.lebanon.expectations.resupply'] = verdict_of(found, names={'airbases resupplied with ammo', 'airbases resupplied with fuel'})
        exact = f'{scenario}: PASS' in text
        code, verdict = compare(os.path.join(HERE, pack['artifacts']['baseline.' + s]['path']), metrics, os.path.join(logs, f'tolerance-{scenario}.txt'), exact=False)
        results[f'regress.{s}.exact'] = ('PASS' if exact else 'FAIL', 'IDENTICAL to the baseline' if exact
                                         else f'not identical; tolerance mode: {verdict}')

    # the reference: the runner's own verdicts (its summary), then the loss chain and the retained evidence
    summary_path = os.path.join(reference, 'summary.txt')
    summary = [l.strip() for l in open(summary_path, encoding='utf-8', errors='replace')] if os.path.exists(summary_path) else []
    failed_run = next((l for l in summary if ' failed (exit ' in l), None) if summary else 'no summary: see logs/reference.txt'
    has = lambda text: any(text in l for l in summary)
    if failed_run:
        for key in [k for k in CHECKS if k.startswith(('reference.', 'retained.'))]:
            results[key] = ('FAIL', failed_run)
        return True
    obs = [l for l in summary if 'observation check' in l]
    results['reference.observation_check'] = ('PASS' if len(obs) == 2 and all(l.endswith('PASS') for l in obs) else 'FAIL', '; '.join(obs) or 'no result')
    same = [l for l in summary if 'byte-identical in both runs' in l or 'the runs DIFFER' in l]
    results['reference.repeatable'] = ('PASS' if len(same) == 3 and all('byte-identical' in l for l in same) else 'FAIL', '; '.join(l.split(' (')[0] for l in same) or 'no result')
    baseline = has('metrics against the regression baseline: IDENTICAL') and has('lebanon_retail: 39 of 39 campaign expectations hold')
    results['reference.baseline'] = ('PASS' if baseline else 'FAIL', '; '.join(l for l in summary if 'regression baseline' in l or 'campaign expectations' in l))
    # observing does not perturb: judged against this build's own unobserved run, so a changed build is not mistaken for perturbation
    unobserved = os.path.join(regress, 'lebanon_retail.json')
    if os.path.exists(unobserved):
        code, verdict = compare(unobserved, os.path.join(reference, 'run1', 'metrics.json'), os.path.join(logs, 'observed-vs-unobserved.txt'))
        results['reference.observed_equals_unobserved'] = ('PASS' if code == 0 else 'FAIL', verdict)
    else:
        results['reference.observed_equals_unobserved'] = ('FAIL', 'the regression wrote no Lebanon metrics to compare with')
    chains = [l for l in summary if 'supply chains:' in l]
    results['reference.supply_chains'] = ('PASS' if len(chains) == 2 and all(l.endswith('PASS') for l in chains) else 'FAIL', '; '.join(chains) or 'no result')

    run1 = os.path.join(reference, 'run1')
    log = os.path.join(logs, 'loss-chain.txt')
    with open(log, 'w', encoding='utf-8', newline='\n') as err:
        code = subprocess.run([PY, tool('loss-chain-check.py'), os.path.join(run1, 'observations.jsonl'), '--report', os.path.join(out, 'loss-chain.json')],
                              stdout=subprocess.DEVNULL, stderr=err, cwd=HERE).returncode
    loss = json.load(open(os.path.join(out, 'loss-chain.json'))) if os.path.exists(os.path.join(out, 'loss-chain.json')) else {}
    first = loss.get('loss') or {}
    results['reference.loss_chain'] = ('PASS' if code == 0 else 'FAIL', (f"{first.get('type')} {first.get('aircraft')} of '{first.get('callsign')}' at {first.get('t')} s; " if first else '') + tail(log))

    art = lambda key: os.path.join(HERE, pack['artifacts'][key]['path'])
    for key, name, want in (('retained.recording', 'recording.acmi', 'm2.recording'), ('retained.observations', 'observations.jsonl', 'm3.observations')):
        path = os.path.join(run1, name)
        got = sha256(path) if os.path.exists(path) else None
        target = pack['artifacts'][want]['content_sha256']
        results[key] = ('PASS' if got == target else 'FAIL', f"{name} {'byte-identical to' if got == target else 'differs from'} {pack['artifacts'][want]['path']}"
                        + ('' if got == target else f' ({str(got)[:12]} vs {target[:12]})'))
    results['retained.check_report'] = same_report(os.path.join(run1, 'check.json'), art('m3.check'))
    results['retained.chains_report'] = same_report(os.path.join(run1, 'chains.json'), art('m3.chains'))
    results['retained.loss_report'] = same_report(os.path.join(out, 'loss-chain.json'), art('m3.loss'))
    return True


# -- the report -------------------------------------------------------------

def claim_status(claim, results):
    if claim['class'] in ('evidence-only', 'not-covered'):
        return STATUS[(claim['class'], None)]
    states = [results.get(c, ('NOT RUN', ''))[0] for c in claim['checks']]
    worst = 'FAIL' if 'FAIL' in states else 'NOT RUN' if 'NOT RUN' in states else 'PASS'
    return STATUS[(claim['class'], worst)]


def validate(pack):
    problems = []
    spots = {b['id'] for b in pack['blind_spots']}
    for c in pack['claims']:
        if c['class'] not in pack['classes']:
            problems.append(f"{c['id']}: unknown class {c['class']}")
        if c['class'] in ('direct', 'baseline-sensitive') and not c['checks']:
            problems.append(f"{c['id']}: a {c['class']} claim needs a check")
        if c['class'] in ('evidence-only', 'not-covered') and c['checks']:
            problems.append(f"{c['id']}: an {c['class']} claim runs no check")
        problems += [f"{c['id']}: unknown check {k}" for k in c['checks'] if k not in CHECKS]
        problems += [f"{c['id']}: unknown blind spot {b}" for b in c.get('blind_spots', []) if b not in spots]
    problems += [f"blind spot {b['id']}: unknown check {b['check']}" for b in pack['blind_spots'] if b.get('check') and b['check'] not in CHECKS]
    if len({c['id'] for c in pack['claims']}) != len(pack['claims']):
        problems.append('claim ids are not unique')
    return problems


def git(*args):
    try:
        return subprocess.run(['git', *args], capture_output=True, text=True, cwd=HERE).stdout.strip()
    except OSError:
        return ''


def write_report(pack, a, out, results, identity, tiers):
    claims = [dict(c, status=claim_status(c, results)) for c in pack['claims']]
    counts = {}
    for c in claims:
        counts[c['status']] = counts.get(c['status'], 0) + 1
    failed = [k for k, (s, _) in results.items() if s == 'FAIL']
    complete = tiers['runtime'] and not failed
    verdict = 'PASS' if complete else 'FAIL' if failed else 'INCOMPLETE (retained tier only)' if tiers['retained'] else 'FAIL'
    dirty = git('status', '--porcelain', '--untracked-files=no', '--', '.')
    lineage = baseline = None
    if tiers['retained']:
        g = governance()
        lin = g.lineages['windows-retail']
        lineage = g.describe('windows-retail')
        baseline = {'lineage': 'windows-retail', 'version': lin['current'],
                    'files': next(v for v in lin['versions'] if v['id'] == lin['current'])['files']}
    head = {
        'pack_revision': git('rev-parse', 'HEAD') + (' (with uncommitted changes)' if dirty else ''),
        'when': datetime.datetime.now().isoformat(timespec='seconds'),
        'command': ' '.join(sys.argv),
        'build': identity, 'roots': {'georgia': a.georgia, 'lebanon': a.lebanon, 'lebanon_repeat': a.lebanon_repeat},
        'baseline': baseline, 'tiers_run': [t for t, ran in tiers.items() if ran], 'verdict': verdict,
    }
    # a candidate's differences: their disposition from the change records (never a pass)
    disposition = None
    if tiers['runtime'] and failed:
        disposition = governance().disposition({'head': head, 'claims': claims, 'checks': {k: {'status': s} for k, (s, _) in results.items()}})
        for c in claims:
            if c['status'].startswith('FAIL'):
                c['disposition'] = disposition['items'].get(c['id'], 'UNCLASSIFIED')
        verdict = head['verdict'] = f"FAIL - {disposition['text']}"
    json.dump({'head': head, 'checks': {k: dict({'status': s, 'detail': d, 'what': CHECKS[k]}, **({'disposition': disposition['items'][k]} if disposition and k in disposition['items'] else {}))
                                        for k, (s, d) in results.items()},
               'claims': claims, 'counts': counts, 'disposition': disposition, 'blind_spots': pack['blind_spots']},
              open(os.path.join(out, 'report.json'), 'w', encoding='utf-8', newline='\n'), indent=1)

    md = [f'# Regression pack: {verdict}', '',
          f"- pack revision: `{head['pack_revision']}`; run {head['when']}",
          f"- command: `{head['command']}`"]
    if identity:
        same = not identity['differs_from_accepted']
        md += [f"- build under test: `{identity['path']}`: " + ('the accepted candidate binaries (0288ece2) and M3 scripts (0ea49a4a), byte for byte' if same
               else 'a different build from the accepted one in ' + ', '.join(f'`{n}`' for n in identity['differs_from_accepted'])),
               '  - ' + ' / '.join(identity['build_info'])]
    if lineage:
        md.append(f'- accepted baseline: {lineage} (`regression/baselines/lineages.json`)')
    if disposition:
        md.append(f"- disposition of the differences: {disposition['text']}")
    md += [f"- roots: Georgia `{a.georgia}`, Lebanon `{a.lebanon}` and `{a.lebanon_repeat}`" if a.georgia else '- roots: none (the runtime tier did not run)',
           '- seed 1, 3 simulated hours, 100 ms frames; the scenarios, inputs, artifacts and checkers are listed in `regression/pack.json`', '',
           '## Claims', '', '| Status | Claim | Checks | Accepted | Sensitivity |', '|---|---|---|---|---|']
    for c in claims:
        checks = '<br>'.join(f"`{k}`: {results.get(k, ('NOT RUN', ''))[0]}" for k in c['checks']) or '-'
        shown = f"**{c['status']}**" + (f"; {c['disposition']}" if c.get('disposition') else '')
        md.append(f"| {shown} | `{c['id']}`: {c['claim']} | {checks} | {c['accepted']} | {c.get('sensitivity', '')} |")
    md += ['', '## Checks', '', '| Check | Result | Detail |', '|---|---|---|']
    for k in CHECKS:
        s, d = results.get(k, ('NOT RUN', ''))
        md.append(f"| `{k}` | {s} | {d.replace('|', '/')} |")
    md += ['', '## Blind spots', '', '| Blind spot | What | Claims it limits | Check |', '|---|---|---|---|']
    for b in pack['blind_spots']:
        limits = ', '.join(f"`{c['id']}`" for c in claims if b['id'] in c.get('blind_spots', [])) or '-'
        check = f"`{b['check']}`: {results.get(b['check'], ('NOT RUN', ''))[0]}" if b.get('check') else 'documented'
        md.append(f"| `{b['id']}` | {b['what']} | {limits} | {check} |")
    md += ['', '## Summary', ''] + [f'- {s}: {n}' for s, n in sorted(counts.items())] + ['', f'**REGRESSION PACK: {verdict}**', '']
    open(os.path.join(out, 'report.md'), 'w', encoding='utf-8', newline='\n').write('\n'.join(md))

    for c in claims:
        print(f"{c['status']:45} {c['id']}" + (f"  [{c['disposition']}]" if c.get('disposition') else ''))
    for k, (s, d) in results.items():
        if s != 'PASS':
            print(f'{s}: {k}: {d}')
    print('; '.join(f'{s}: {n}' for s, n in sorted(counts.items())))
    print(f'REGRESSION PACK: {verdict} (report: {os.path.relpath(os.path.join(out, "report.md"), os.getcwd())})')
    return 0 if complete or (not a.georgia and tiers['retained'] and not failed) else 1


def main():
    p = argparse.ArgumentParser(description='the accepted M1-M3 regression pack')
    p.add_argument('--georgia')
    p.add_argument('--lebanon')
    p.add_argument('--lebanon-repeat')
    p.add_argument('--bin', default=os.path.join(HERE, 'target', 'windows'))
    p.add_argument('--out', default=os.path.join(HERE, 'target', 'regression-pack'))
    p.add_argument('--retained-only', action='store_true')
    a = p.parse_args()
    if not a.retained_only and not (a.georgia and a.lebanon and a.lebanon_repeat):
        p.error('--georgia, --lebanon and --lebanon-repeat are needed (or --retained-only)')
    if a.retained_only:
        a.georgia = a.lebanon = a.lebanon_repeat = None
    a.bin, out = os.path.abspath(a.bin), os.path.abspath(a.out)
    os.makedirs(out, exist_ok=True)

    pack = json.load(open(PACK))
    problems = validate(pack)
    if problems:
        sys.exit('regression/pack.json is inconsistent:\n  ' + '\n  '.join(problems))
    results, identity, tiers = {}, None, {'retained': False, 'runtime': False}
    print('retained tier: artifacts, checkers, reproduction and sensitivity (no retail data) ...', flush=True)
    tiers['retained'] = retained_tier(pack, out, results)
    if not tiers['retained']:
        print('RETAINED EVIDENCE INCONSISTENT: the pack cannot judge a build against it', file=sys.stderr)
    elif not a.retained_only:
        if not os.path.exists(os.path.join(a.bin, 'eech-world.exe')):
            sys.exit(f'{a.bin} has no eech-world.exe: pass --bin <build> (tools/build-windows.sh)')
        identity = build_identity(pack, a.bin)
        tiers['runtime'] = runtime_tier(pack, a, out, results, identity)
    return write_report(pack, a, out, results, identity, tiers)


if __name__ == '__main__':
    sys.exit(main())
