"""Parent verification of the completed candidate; never writes a baseline."""
from pathlib import Path
import datetime
import gzip
import hashlib
import json
import sys
import zipfile

out = Path('E:/eech-runs/m5-d1/aircraft-control-20261023/native')
repo = Path('E:/eech_source_code/.claude/worktrees/m5-aircraft-control/eech-campaign')
if not (out / 'pack.exit').exists():
    raise SystemExit('Pack is still running; no complete-result claim.')
exit_code = int((out / 'pack.exit').read_text(encoding='utf-8-sig').strip())
report = json.loads((out / 'pack/report.json').read_text())
failed = {key: value for key, value in report['checks'].items() if value['status'] != 'PASS'}
if exit_code or failed or len(report['checks']) != 40:
    print(json.dumps({'pack_exit': exit_code, 'check_count': len(report['checks']), 'nonpassing': failed}))
    raise SystemExit(1)

def digest(stream):
    hashed = hashlib.sha256()
    size = 0
    while chunk := stream.read(1024 * 1024):
        size += len(chunk)
        hashed.update(chunk)
    return size, hashed.hexdigest()

reference = repo / 'reference/lebanon-3h'
with zipfile.ZipFile(reference / 'recording.zip.acmi') as archive:
    with archive.open('recording.acmi') as stream:
        tacview = digest(stream)
with gzip.open(repo / 'reference/m3-resupply-loop/observations.jsonl.gz', 'rb') as stream:
    observations = digest(stream)
if tacview != (117690861, '3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b'):
    raise SystemExit('Unexpected retained Tacview identity.')
if observations != (85716447, 'da26bab01163aa4ab725f9f2ecb516d37a4558777fe3684222b8d885822e77cf'):
    raise SystemExit('Unexpected retained supply-detail observation identity.')
baseline = (repo / 'regression/baselines/windows-retail/v1/lebanon_retail.json').read_bytes()
results = []
for run in ('run1', 'run2'):
    folder = out / 'pack/reference' / run
    for filename, retained in (('recording.acmi', tacview), ('observations.jsonl', observations)):
        with (folder / filename).open('rb') as stream:
            size, checksum = digest(stream)
        results.append({'run': run, 'file': filename, 'bytes': size, 'sha256': checksum,
                        'retained_bytes': retained[0], 'retained_sha256': retained[1],
                        'matches_retained_bytes': (size, checksum) == retained})
    metrics = (folder / 'metrics.json').read_bytes()
    results.append({'run': run, 'file': 'metrics.json', 'bytes': len(metrics),
                    'sha256': hashlib.sha256(metrics).hexdigest(),
                    'matches_retained_json_values': json.loads(metrics) == json.loads(baseline),
                    'matches_after_crlf_normalization': metrics.replace(b'\r\n', b'\n') == baseline.replace(b'\r\n', b'\n')})

passed = all(item.get('matches_retained_bytes', item.get('matches_retained_json_values')) for item in results)
record = {'checked_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'complete_files': True, 'pack_exit': exit_code, 'pack_checks_passed': 40,
          'native_head': (out / 'build.head').read_text().strip(),
          'passed': passed,
          'note': 'Complete Tacview compared with reference/lebanon-3h/recording.zip.acmi; observations with the accepted supply-detail reference/m3-resupply-loop/observations.jsonl.gz. Metrics compared as JSON values; normalized-byte equality is reported separately. No baseline was modified.',
          'results': results}
(out / 'independent-reference-hashes.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
sys.exit(0 if passed else 1)
