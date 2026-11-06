import json, hashlib, subprocess, sys
repo, code, mp = sys.argv[1:4]
post = json.load(open(mp, encoding='utf-8'))
p = "eech-campaign/crates/eech-world/tests/entity_authority.rs"
blob = subprocess.check_output(['git', '-C', repo, 'show', code + ':' + p])
e = post['files'][p]
print(p.rsplit('/', 1)[1], hashlib.sha256(blob).hexdigest() == e['raw_sha256'], subprocess.check_output(['git', '-C', repo, 'rev-parse', code + ':' + p], text=True).strip()[:12], e['size'], len(blob))
