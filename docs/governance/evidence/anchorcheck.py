"""Check relative links and #anchors in Markdown files (GitHub slug rules).
Usage: python docs/governance/evidence/anchorcheck.py <repo-root> <file>...
External links are listed, not fetched. Exits 1 if any check fails.
"""
import re, sys, pathlib

root = pathlib.Path(sys.argv[1])

def slugs(path):
    out, seen, fence = set(), {}, False
    for line in path.read_text(encoding='utf-8').splitlines():
        if line.startswith('```'):
            fence = not fence
        if fence:
            continue
        m = re.match(r'#{1,6}\s+(.*)', line)
        if not m:
            continue
        t = m.group(1).strip().lower()
        t = re.sub(r'[^\w\- ]', '', t).replace(' ', '-')
        n = seen.get(t, 0); seen[t] = n + 1
        out.add(t if n == 0 else f'{t}-{n}')
    return out

fails = 0; ext = set()
for f in sys.argv[2:]:
    p = root / f
    for m in re.finditer(r'\]\(([^)\s]+)\)', p.read_text(encoding='utf-8')):
        link = m.group(1)
        if link.startswith('http'):
            ext.add(link); continue
        target, _, anchor = link.partition('#')
        tp = (p.parent / target) if target else p
        if not tp.exists():
            print('BROKEN', f, link); fails += 1; continue
        if anchor and tp.suffix == '.md' and anchor not in slugs(tp):
            print('BAD ANCHOR', f, link); fails += 1
print('external links:'); [print(' ', e) for e in sorted(ext)]
print('FAILURES', fails); sys.exit(1 if fails else 0)
