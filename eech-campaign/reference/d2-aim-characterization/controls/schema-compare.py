"""Schema-expanded equivalence of two aim traces: every event of the OLD trace must appear in the NEW trace (same scope, event and identifying fields)
with every old field equal; new fields and new events are allowed and listed; a declared set of fields may differ (named, with the reason).
usage: schema-compare.py <old.jsonl> <new.jsonl> <out.json> [field=reason ...]   -- exit 0 only when no undeclared old field differs"""
import json
import sys

old_path, new_path, out_path = sys.argv[1:4]
declared = dict(a.split('=', 1) for a in sys.argv[4:])


def load(p):
    return [json.loads(l) for l in open(p, encoding='utf-8') if l.strip()]


def key(e):
    # an event's identity: scope, event, at, and its name-like fields (counters, ranges, kinds, weapons, samples)
    ident = [e.get('scope'), e.get('event'), e.get('at')]
    for f in ('name', 'source', 'function', 'sub_type', 'aiming', 'key'):
        ident.append(e.get(f))
    if e.get('event') == 'aim-sample':
        ident.append(e.get('text'))
    return tuple(ident)


old, new = load(old_path), load(new_path)
index = {}
for e in new:
    index.setdefault(key(e), []).append(e)
old_count = len(old)
missing, changed, declared_changes = [], [], []
used = {}
for e in old:
    k = key(e)
    candidates = index.get(k)
    if not candidates:
        missing.append(e)
        continue
    n = used.get(k, 0)
    used[k] = n + 1
    other = candidates[n] if n < len(candidates) else candidates[-1]
    for field, value in e.items():
        if other.get(field) != value:
            if field in declared:
                declared_changes.append({'event': e.get('event'), 'scope': e.get('scope'), 'at': e.get('at'), 'field': field, 'old': value, 'new': other.get(field)})
            else:
                changed.append({'event': e.get('event'), 'scope': e.get('scope'), 'at': e.get('at'), 'field': field, 'old': value, 'new': other.get(field)})
new_events = len(new) - sum(min(len(v), used.get(k, 0)) for k, v in index.items())
added_fields = sorted({f for e in new for f in e} - {f for e in old for f in e})
added_events = sorted({e['event'] for e in new if key(e) not in {key(o) for o in old}})
result = {
    'old': old_path, 'new': new_path, 'old_events': old_count, 'new_events': len(new), 'old_events_missing_in_new': len(missing),
    'old_fields_changed_undeclared': len(changed), 'declared_field_changes': declared, 'declared_changes_seen': len(declared_changes),
    'new_fields': added_fields, 'new_event_names': added_events, 'new_events_not_in_old': new_events,
    'first_undeclared_changes': changed[:10], 'first_missing': missing[:5], 'passed': not missing and not changed,
}
json.dump(result, open(out_path, 'w'), indent=2)
print(json.dumps({k: v for k, v in result.items() if k not in ('first_undeclared_changes', 'first_missing')}))
sys.exit(0 if result['passed'] else 1)
