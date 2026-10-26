"""Finish separate checker-only evidence after ORIGINAL pack actually exits.

Must pass --original-exit 1 only after observing exec89376 completion. All raw
scenarios and original failures remain unchanged. No native run or lifecycle
experiment is repeated. The frozen pack's unchanged post-run body and report
writer recompute results using unchanged checkers and the existing raw artifacts.
"""
import argparse
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

# Keep the frozen checker checkout immutable when importing its report code.
sys.dont_write_bytecode = True
os.environ['PYTHONDONTWRITEBYTECODE'] = '1'

p = argparse.ArgumentParser()
p.add_argument('--original-exit', required=True, type=int, choices=[1])
a_cli = p.parse_args()
root = Path('R:/d2-evidence')
frozen = root / 'shipping-source-d2-sensing-a2a041fec/eech-campaign'
original = root / 'candidate-pack-sensing-a2a041fec'
destination = root / 'candidate-pack-sensing-a2a041fec-postcheck-recovery'
source = frozen / 'tools/regression-pack.py'
old = json.loads((original / 'report.json').read_text())
assert old['head']['pack_revision'] == 'a2a041fecf52e18cbc16b33f3dc75a9c9b245c9d'
assert old['head']['verdict'].startswith('FAIL') and len(old['checks']) == 40
assert set(old['head']['tiers_run']) == {'retained','runtime'}
expected_failures = {
    'regress.georgia.exact', 'regress.lebanon.exact',
    'reference.observation_check', 'reference.baseline', 'reference.supply_chains',
    'retained.check_report', 'retained.chains_report',
}
failed = {k for k,v in old['checks'].items() if v['status'] != 'PASS'}
assert failed == expected_failures, f'Investigate any additional/different original failure: {failed}'
assert not (destination / 'report.json').exists(), 'Never overwrite a prior full recovery'
reference_record = json.loads((destination / 'reference-postcheck-recovery.json').read_text())
assert reference_record['actual_postcheck_process_exit'] == 0
assert reference_record['original_reference_files_unchanged']

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return {'bytes': path.stat().st_size, 'sha256': h.hexdigest()}

kept = [p for p in original.rglob('*') if p.is_file() and 'roots' not in p.relative_to(original).parts]
kept += [root / 'candidate-pack-sensing-a2a041fec.console.log', root / 'run-frozen-pack.py',
         source, frozen / 'tools/reference-windows.ps1', frozen / 'tools/regress-windows.ps1']
before = {str(p): digest(p) for p in kept}
# A failed native would make the unchanged regression runner exit before its
# postcheck loop, instead of printing these two alias-caused exact-check verdicts.
regress_log = (original / 'logs/regress.txt').read_text(encoding='utf-8-sig')
assert 'Python was not found' in regress_log
assert 'georgia_retail: FAIL' in regress_log and 'lebanon_retail: FAIL' in regress_log
assert ' failed (exit ' not in regress_log and 'runs took ' in regress_log

(destination / 'regress').mkdir()
(destination / 'logs').mkdir()
for name in ('georgia_retail.json', 'lebanon_retail.json'):
    shutil.copy2(original / 'regress' / name, destination / 'regress' / name)
for path in (original / 'logs').iterdir():
    if path.is_file():
        shutil.copy2(path, destination / 'logs' / path.name)
shutil.copy2(original / 'report.json', destination / 'original-pack-report-FAILED.json')
shutil.copy2(original / 'report.md', destination / 'original-pack-report-FAILED.md')
shutil.copy2(destination / 'reference-postchecks.log', destination / 'logs/reference.txt')

# Run only the original regression runner's post-native block, with a real Python.
regress_source = frozen / 'tools/regress-windows.ps1'
regress_text = regress_source.read_text(encoding='utf-8')
marker = '$status = 0\nforeach ($name in'
assert regress_text.count(marker) == 1
post = '$status = 0\nforeach ($name in' + regress_text.split(marker,1)[1]
assert 'Start-Process' not in post and '$p.WaitForExit()' not in post
def quote(value):
    return "'" + str(value).replace("'", "''") + "'"
script = destination / 'regress-postchecks.ps1'
script.write_text(
    "$ErrorActionPreference='Stop'\n"
    f'$out={quote(destination / "regress")}\n$baselines={quote(frozen / "regression/windows")}\n'
    f'$here={quote(frozen)}\n$python={quote(sys.executable)}\n$Exact=$true\n' + post)
shell = shutil.which('pwsh') or shutil.which('powershell')
assert shell
with (destination / 'logs/regress.txt').open('wb') as log:
    result = subprocess.run([shell,'-NoProfile','-File',str(script)],stdout=log,stderr=subprocess.STDOUT)
assert result.returncode == 0, 'Regression checker failure retained; do not accept'

spec = importlib.util.spec_from_file_location('frozen_sensing_pack',source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
pack = json.loads(Path(module.PACK).read_text())
assert not module.validate(pack) and set(old['checks']) == set(module.CHECKS)
text = source.read_text(encoding='utf-8')
tree = ast.parse(text)
function = next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='runtime_tier')
lines = text.splitlines(keepends=True)
start = next(i for i in range(function.lineno-1,function.end_lineno) if lines[i].startswith('    # the regression: expectations'))
tail = ''.join(lines[start:function.end_lineno])
assert 'pool.submit' not in tail and 'regress-windows.ps1' not in tail and 'reference-windows.ps1' not in tail
prefix = ('def recovered_postchecks(pack,a,out,results,identity):\n'
          '    logs=os.path.join(out,"logs")\n'
          '    regress=os.path.join(out,"regress")\n'
          '    reference=os.path.join(out,"reference")\n'
          '    tool=lambda name: os.path.join(HERE,"tools",name)\n')
exec(compile(prefix+tail,str(source)+'::unchanged-post-run-body','exec'),module.__dict__)
results = {k:(v['status'],v['detail']) for k,v in old['checks'].items()}
arguments = argparse.Namespace(**old['head']['roots'],bin=old['head']['build']['path'],out=str(destination),retained_only=False)
os.chdir(root)
runtime = module.recovered_postchecks(pack,arguments,str(destination),results,old['head']['build'])
assert runtime and len(results)==40 and all(v[0]=='PASS' for v in results.values()), results
return_code = module.write_report(pack,arguments,str(destination),results,old['head']['build'],{'retained':True,'runtime':True})
after = {str(p):digest(p) for p in kept}
record = {
    'scope':'Separate unchanged offline checker recovery; NO native/lifecycle/retained-experiment rerun',
    'original_runner_exec':89376,'original_runner_observed_exit':a_cli.original_exit,
    'original_verdict':old['head']['verdict'],'original_failed_checks':sorted(failed),
    'reused_passing_results':'Original retained tier, input preflights and lifecycle results; no override of any failed check',
    'new_results':'Unchanged PowerShell post-native checks and frozen pack post-run body/report writer',
    'source':str(source),'source_identity':digest(source),
    'recovery_helper_identity':digest(Path(__file__)),
    'reference_recovery_helper_identity':digest(root / 'recover-sensing-reference-postchecks.py'),
    'post_run_body_sha256':hashlib.sha256(tail.encode()).hexdigest(),
    'regression_postcheck_actual_exit':result.returncode,'pack_report_writer_return':return_code,
    'original_evidence_unchanged':before==after,'preserved_original_files':before,
    'complete_40_checks_passed':all(v[0]=='PASS' for v in results.values()),
}
(destination / 'full-postcheck-recovery.json').write_text(json.dumps(record,indent=2)+'\n')
assert before==after,'Original evidence changed'
print('OFFLINE_PACK_REPORT_RETURN='+str(return_code))
sys.exit(return_code)
