"""Rerun only unchanged reference postchecks, never native scenarios.

The original pack/failed logs remain untouched. The original PowerShell runner
reached its postchecks only after its native-exit guard accepted both processes.
Its subsequent Python commands resolved to the Windows Store alias and failed.
This is separate checker evidence, not an original-pack exit-zero claim.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

root = Path('R:/d2-evidence')
frozen = root / 'shipping-source-d2-sensing-a2a041fec/eech-campaign'
original = root / 'candidate-pack-sensing-a2a041fec'
destination = root / 'candidate-pack-sensing-a2a041fec-postcheck-recovery'
source = frozen / 'tools/reference-windows.ps1'
text = source.read_text(encoding='utf-8')
summary = (original / 'reference/summary.txt').read_text(encoding='utf-8-sig')
assert 'REFERENCE: FAIL' in summary
assert 'Python was not found' in summary
assert ' failed (exit ' not in summary
assert 'runs took ' in summary and 'run 2: observation check FAIL' in summary
guard = 'if ($runs[$i].ExitCode -ne 0)'
assert guard in text and 'exit 1 } }' in text
assert not destination.exists(), 'Never overwrite a prior recovery attempt'

def digest(p):
    h = hashlib.sha256()
    with p.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return {'bytes': p.stat().st_size, 'sha256': h.hexdigest()}

before = {str(p): digest(p) for p in (original / 'reference').rglob('*') if p.is_file()}
rows = []
for run in ('run1', 'run2'):
    for name in ('recording.acmi', 'observations.jsonl', 'metrics.json'):
        rel = Path('reference') / run / name
        old, actual = digest(root / 'baseline-host-20261026-shortpath' / rel), digest(original / rel)
        assert old == actual, f'Complete reference differs: {rel}'
        rows.append({'file': rel.as_posix(), 'baseline': old, 'candidate': actual, 'matches': True})

destination.mkdir()
for row in rows:
    rel = Path(row['file'])
    (destination / rel).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(original / rel, destination / rel)
    assert digest(destination / rel) == row['candidate']
shutil.copy2(original / 'reference/BUILD-INFO.txt', destination / 'reference/BUILD-INFO.txt')
(destination / 'original-reference-summary-FAILED.txt').write_text(summary, encoding='utf-8')

# This entire trailing block is the original script's post-native implementation.
marker = '# 3. each run\'s recording against its observations and metrics'
assert text.count(marker) == 1
postchecks = marker + text.split(marker, 1)[1]
assert 'Start-Process' not in postchecks and 'Start-Run' not in postchecks
assert postchecks.endswith('exit $status\n')
def psquote(value):
    return "'" + str(value).replace("'", "''") + "'"
prefix = (
    "$ErrorActionPreference = 'Stop'\n"
    f'$here = {psquote(frozen)}\n$out = {psquote(destination / "reference")}\n'
    f'$python = {psquote(sys.executable)}\n'
    '$Hours = 3\n$ObserveSupply = $true\n$status = 0\n'
    "function Say ($text) { Write-Host $text; Add-Content -Path (Join-Path $out 'summary.txt') -Value $text }\n"
    "Set-Content -Path (Join-Path $out 'summary.txt') -Value 'OFFLINE POSTCHECK RECOVERY ONLY: original native runs reused; original alias failures preserved separately'\n"
)
script = destination / 'reference-postchecks.ps1'
script.write_text(prefix + postchecks, encoding='utf-8')
shell = shutil.which('pwsh') or shutil.which('powershell')
assert shell
os.chdir(root)
with (destination / 'reference-postchecks.log').open('wb') as output:
    result = subprocess.run([shell, '-NoProfile', '-File', str(script)], stdout=output, stderr=subprocess.STDOUT)
after = {str(p): digest(p) for p in (original / 'reference').rglob('*') if p.is_file()}
record = {
    'scope': 'Separate unchanged reference postchecks only; NO native scenario rerun; NOT a complete40-check pack yet',
    'original_pack_exec': 89376, 'original_pack_exit': 'still running Georgia; not asserted',
    'original_reference_native_exits': 'Both zero established by unchanged runner exit guard reaching postchecks; no fabricated direct exit capture',
    'original_reference_failure': 'PowerShell selected Windows Store Python alias; original summary FAIL retained untouched',
    'source': str(source), 'source_identity': digest(source),
    'unchanged_postcheck_block_sha256': hashlib.sha256(postchecks.encode()).hexdigest(),
    'recovery_script': str(script), 'recovery_script_identity': digest(script),
    'explicit_python': sys.executable, 'python_version': sys.version,
    'actual_postcheck_process_exit': result.returncode,
    'all_six_complete_reference_files_equal_baseline': rows,
    'original_reference_files_unchanged': before == after,
    'preserved_original_reference_files': before,
}
(destination / 'reference-postcheck-recovery.json').write_text(json.dumps(record, indent=2) + '\n')
assert before == after, 'Original failed evidence changed'
print('REFERENCE_POSTCHECK_PROCESS_EXIT=' + str(result.returncode))
sys.exit(result.returncode)
