"""Complete-file byte comparison of a 900 s whole-campaign run against the retained supplied-binary side (ab-base). Reads only."""
import hashlib, json, sys, os
base, cand, out = sys.argv[1], sys.argv[2], sys.argv[3]
expected = {'recording.acmi': 11363845, 'observations.jsonl': 6629273, 'metrics.json': 11777}
res = {'baseline_dir': base, 'candidate_dir': cand, 'artifacts': [], 'note': 'complete actual files, not normalised copies; baseline side NOT re-run'}
ok = True
for f, size in expected.items():
    row = {'file': f, 'expected_bytes': size}
    for side, d in (('baseline', base), ('candidate', cand)):
        p = os.path.join(d, f)
        if not os.path.isfile(p):
            row[side + '_error'] = 'missing'; ok = False; continue
        b = open(p, 'rb').read()
        row[side + '_bytes'] = len(b); row[side + '_sha256'] = hashlib.sha256(b).hexdigest()
    row['matches'] = row.get('baseline_sha256') is not None and row.get('baseline_sha256') == row.get('candidate_sha256') and row.get('baseline_bytes') == size
    ok = ok and row['matches']
    res['artifacts'].append(row)
res['passed'] = ok
json.dump(res, open(out, 'w'), indent=2)
print(json.dumps({r['file']: r['matches'] for r in res['artifacts']}), 'passed', ok)
sys.exit(0 if ok else 1)
