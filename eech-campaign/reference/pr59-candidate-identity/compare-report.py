# Compares two regression-pack report.json files leaf by leaf and prints every differing value.
# Usage: python -I compare-report.py <a report.json> <b report.json>
import json, sys
a, b = (json.load(open(p)) for p in sys.argv[1:3])
print('top-level keys:', sorted(a.keys()))
def flat(x, pre=''):
    if isinstance(x, dict):
        for k, v in x.items():
            yield from flat(v, f'{pre}.{k}')
    elif isinstance(x, list):
        for i, v in enumerate(x):
            yield from flat(v, f'{pre}[{i}]')
    else:
        yield pre, x
fa, fb = dict(flat(a)), dict(flat(b))
diff = sorted(k for k in set(fa) | set(fb) if fa.get(k) != fb.get(k))
print(f'{len(diff)} differing leaves')
for k in diff:
    print(f'  {k}: {str(fa.get(k))[:110]!r} -> {str(fb.get(k))[:110]!r}')
