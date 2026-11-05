"""Compare complete reports and raw files; process exits/input identity are separate gates."""
import argparse
import hashlib
import json
from pathlib import Path
import sys


def digest(path):
    if path.stat().st_size == 0:
        raise ValueError(f'Empty completed artifact: {path}')
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return {'bytes':path.stat().st_size, 'sha256':h.hexdigest()}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('baseline','candidate','accepted-report','out'):
        p.add_argument('--'+name,required=True,type=Path)
    a = p.parse_args()
    if a.out.exists():
        raise ValueError('Preserve existing comparison; choose a fresh --out')
    expected = json.loads(a.accepted_report.read_text(encoding='utf-8-sig'))
    checks = set(expected['checks'])
    if not checks or expected['head']['verdict'] != 'PASS' or any(v['status']!='PASS' for v in expected['checks'].values()):
        raise ValueError('Expected a complete accepted PASS report')
    for name,directory in (('baseline',a.baseline),('candidate',a.candidate)):
        report = json.loads((directory/'report.json').read_text(encoding='utf-8-sig'))
        if report['head']['verdict'] != 'PASS' or set(report['checks']) != checks or any(v['status']!='PASS' for v in report['checks'].values()):
            raise ValueError(f'{name}: complete accepted check set is not all PASS')
    paths = [Path('reference')/run/name for run in ('run1','run2')
             for name in ('recording.acmi','observations.jsonl','metrics.json')]
    paths += [Path('regress')/(name+'.json') for name in ('georgia_retail','lebanon_retail')]
    rows = []
    for rel in paths:
        old,new = digest(a.baseline/rel),digest(a.candidate/rel)
        rows.append({'file':rel.as_posix(),'baseline':old,'candidate':new,'matches':old==new})
    result = {'scope':'Complete artifact comparison only; actual exits and post-input manifests must be verified separately',
              'partial_prefix_only':False,'accepted_checks_each':len(checks),'results':rows,
              'passed':all(row['matches'] for row in rows)}
    a.out.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(f'{len(checks)} accepted checks per report; {sum(r["matches"] for r in rows)}/8 complete files equal')
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError,ValueError,KeyError) as error:
        print('REFUSED complete comparison: '+str(error),file=sys.stderr)
        sys.exit(2)
