# Compares two pack logs/ directories line by line, after normalising CRLF, worktree paths and the
# candidate output-directory name. Usage: python -I compare-logs.py <a logs dir> <b logs dir>
import difflib, re, sys, os
a_dir, b_dir = sys.argv[1:3]
def norm(p):
    t = open(p, encoding='utf-8', errors='replace').read().replace('\r', '')
    t = re.sub(r'agent-a[0-9a-f]+', 'AGENT', t)
    t = re.sub(r'pr(59|76)-candidate', 'CAND', t)
    return t.splitlines()
for f in sorted(os.listdir(a_dir)):
    pa, pb = os.path.join(a_dir, f), os.path.join(b_dir, f)
    if not os.path.isfile(pa):
        continue
    if not os.path.exists(pb):
        print(f'== {f}: missing in b'); continue
    d = list(difflib.unified_diff(norm(pa), norm(pb), lineterm='', n=0))
    print(f'== {f}: {"same after normalising" if not d else "DIFFERS"}')
    for line in d[:14]:
        print('   ' + line)
