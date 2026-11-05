"""Finish the unchanged report writer after its cross-drive display failure.

No scenarios, retained checks, inputs, binaries or original outputs are rerun or
modified. Original runner exit was 1 after writing both PASS reports. This is a
separate report-only invocation, not a claim that the original runner exited 0.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

root = Path('R:/d2-evidence')
source = root / 'shipping-source-a7863a73f/eech-campaign/tools/regression-pack.py'
original = root / 'candidate-pack-los-61e856645'
destination = original / 'report-completion'
assert not destination.exists(), 'Preserve any prior report-completion attempt'
report = json.loads((original / 'report.json').read_text())
assert report['head']['pack_revision'] == '61e85664540770ab10e3ac7e9f5dfa916291c52f'
assert report['head']['verdict'] == 'PASS'
assert set(report['head']['tiers_run']) == {'retained', 'runtime'}
assert len(report['checks']) == 40
assert all(c['status'] == 'PASS' for c in report['checks'].values())
assert json.loads((root / 'los-final-complete-pack-comparison.json').read_text())['passed']

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

kept = [original / 'report.json', original / 'report.md',
        root / 'candidate-pack-los-61e856645.console.log']
for item in json.loads((root / 'los-final-complete-pack-comparison.json').read_text())['results']:
    kept.append(original / item['file'])
before = {str(p): digest(p) for p in kept}

os.chdir(root)  # explicit native cwd, not PowerShell's provider location
assert os.path.splitdrive(os.getcwd())[0].lower() == 'r:'
spec = importlib.util.spec_from_file_location('completed_los_pack', source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
pack = json.loads(Path(module.PACK).read_text())
assert not module.validate(pack)
assert set(report['checks']) == set(module.CHECKS)
arguments = argparse.Namespace(
    georgia=report['head']['roots']['georgia'],
    lebanon=report['head']['roots']['lebanon'],
    lebanon_repeat=report['head']['roots']['lebanon_repeat'],
    bin=report['head']['build']['path'], out=str(destination), retained_only=False)
results = {k: (v['status'], v['detail']) for k, v in report['checks'].items()}
destination.mkdir()
result = module.write_report(pack, arguments, str(destination), results,
                             report['head']['build'], {'retained': True, 'runtime': True})
completed = json.loads((destination / 'report.json').read_text())
for key in ('checks', 'claims', 'counts', 'blind_spots', 'disposition'):
    assert completed[key] == report[key], key
for key in ('pack_revision', 'build', 'roots', 'baseline', 'tiers_run', 'verdict'):
    assert completed['head'][key] == report['head'][key], key
after = {str(p): digest(p) for p in kept}
assert before == after, 'Original evidence must remain untouched'
record = {
    'scope': 'Report-only recovery of completed LOS pack; NO scenario/check rerun',
    'original_runner_exec_session': 15414,
    'original_runner_observed_exit': 1,
    'original_failure': "write_report final display: ntpath.relpath ValueError R: vs C:",
    'original_pack_report_verdict': 'PASS', 'original_checks_passed': 40,
    'report_writer_source': str(source), 'report_writer_sha256': digest(source),
    'report_only_return': result, 'original_evidence_unchanged': before == after,
    'preserved_original_sha256': before,
    'regenerated_content_matches_except_command_and_time': True,
    'report_only_output': str(destination),
}
(root / 'los-report-only-completion.json').write_text(json.dumps(record, indent=2) + '\n')
print('REPORT_ONLY_RETURN=' + str(result))
sys.exit(result)
