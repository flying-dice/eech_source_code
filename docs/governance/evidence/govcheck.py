"""Static checks for the governance files (EEDCS-9).

Usage: python3 docs/governance/evidence/govcheck.py <repo-root>
Exits 1 if any check fails.
"""
import re,sys,pathlib
root=pathlib.Path(sys.argv[1]); fails=0
files=['AGENT.md','AGENTS.md','CLAUDE.md','PROJECT.md','docs/governance/REVIEW.md','docs/governance/BOOTSTRAP.md','docs/governance/BASELINE_RECORD.md','.github/PULL_REQUEST_TEMPLATE.md']
for f in files:
    p=root/f; s=p.read_text()
    for m in re.finditer(r'\]\(([^)#\s]+)(#[^)]*)?\)',s):
        t=m.group(1)
        if t.startswith('http'): continue
        if not (p.parent/t).exists(): print('BROKEN',f,t); fails+=1
# CLAUDE.md import outside fence
c=(root/'CLAUDE.md').read_text(); infence=False; ok=False
for line in c.splitlines():
    if line.startswith('```'): infence=not infence
    if line.strip()=='@AGENT.md' and not infence: ok=True
print('CLAUDE.md imports @AGENT.md outside a code block:',ok); fails+= not ok
# stale gate wording
for f in files:
    s=(root/f).read_text()
    for pat in [r'#51\S*\s+must be accepted', r'mark #51 complete', r'leave #52 and the governance', r'what blocks this bootstrap']:
        if re.search(pat,s,re.S): print('STALE',f,pat); fails+=1
# required rules preserved
a=(root/'AGENT.md').read_text()
for need in ['fd-starscream-bot','APPROVED','never by the presence of these files','windows-retail v1','regression pack','Never silently re-record']:
    if need not in a: print('MISSING in AGENT.md:',need); fails+=1
# Owner's decision, 2026-10-20: no server-side protection; rules are enforced by
# process. (Supersedes the earlier check that unverified protection blocks merge.)
r=(root/'docs/governance/REVIEW.md').read_text()
for f,txt,need in [('AGENT.md',a,'enforced by process and recorded evidence only'),
                   ('AGENT.md',a,'Absent server-side protection is not a merge blocker'),
                   ('AGENT.md',a,'self-merge'),
                   ('docs/governance/REVIEW.md',r,'enforced by process and recorded evidence only'),
                   ('docs/governance/REVIEW.md',r,'current formal APPROVED review from fd-starscream-bot on the head commit')]:
    if need not in ' '.join(txt.split()): print('MISSING in',f+':',need); fails+=1
for f in files:
    s=' '.join((root/f).read_text().split())
    for pat in [r'unverified protection is a merge blocker', r'maintain the no-merge policy', r'verify effective protection', r'administrator activates and verifies']:
        if re.search(pat,s,re.I): print('STALE',f,pat); fails+=1
# file sizes
for f in ['AGENT.md','CLAUDE.md','AGENTS.md']: print(f, len((root/f).read_text().splitlines()),'lines')
print('FAILURES',fails); sys.exit(1 if fails else 0)
