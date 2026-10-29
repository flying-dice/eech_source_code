"""Bounded in-memory acceptance/refusal controls for the v2 evidence helper; no native runs."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

base = Path('R:/d2-evidence/combat-characterization-20261028')
path = base / 'audit-r3-64cf-six-v2.py'
spec = importlib.util.spec_from_file_location('r3_audit_v2', path)
audit = importlib.util.module_from_spec(spec); spec.loader.exec_module(audit)
proof = audit.read(base / 'root-r3-64cf-six-report-completion-v2.json')
items = audit.read(base / 'root-r3-64cf-six-original-driver-exits.json')
results = []
hashes = {name: audit.digest(Path(case['trace'])) for name, case in proof['scenarios'].items()}
for name, case in proof['scenarios'].items():
    audit.validate_case(name, case['driver'], case['run'], case['rows'], proof['build_info_sha256'])
    results.append(dict(control='valid:' + name, accepted=True))
audit.driver_set(items)

def reject(label, call):
    try:
        call()
    except AssertionError as error:
        results.append(dict(control=label, rejected=True, reason=str(error)))
    else:
        raise AssertionError('Mutated evidence accepted: ' + label)

def corrupt(label, case_name, change):
    name = 'object-report-' + case_name + '-02'
    case = copy.deepcopy(proof['scenarios'][name])
    change(case)
    reject(label, lambda: audit.validate_case(name, case['driver'], case['run'], case['rows'], proof['build_info_sha256']))

def row(case, event):
    return next(x for x in case['rows'] if x['event'] == event)

corrupt('control-before-boot', 'site-damage-raise', lambda c: c['rows'].insert(0, c['rows'].pop()))
def move_after(c):
    receipt = next(i for i, x in enumerate(c['rows']) if x['event'] == 'object_receipt')
    c['rows'][receipt], c['rows'][receipt + 1] = c['rows'][receipt + 1], c['rows'][receipt]
corrupt('after-before-receipt', 'site-damage-raise', move_after)
reject('missing-own-case', lambda: audit.driver_set([x for x in items if 'own' not in x['case']]))
duplicate = copy.deepcopy(items); duplicate[-1] = copy.deepcopy(duplicate[0])
reject('duplicate-case-hides-one-case', lambda: audit.driver_set(duplicate))
corrupt('receipt-wrong-identity', 'site-damage-raise', lambda c: row(c, 'object_receipt').update(host_id='other-object'))
corrupt('receipt-wrong-before', 'site-damage-raise', lambda c: row(c, 'object_receipt').update(damage_level_before=999))
corrupt('receipt-wrong-after', 'site-damage-raise', lambda c: row(c, 'object_receipt').update(damage_level_after=999))
corrupt('summary-wrong-state', 'site-damage-raise', lambda c: row(c, 'object_summary').update(damage_after=999))
corrupt('boot-wrong-mode', 'site-damage-raise', lambda c: row(c, 'boot').update(report='none'))
corrupt('run-wrong-mode', 'site-damage-raise', lambda c: c['run'].update(report='none'))
assert all(audit.digest(Path(proof['scenarios'][name]['trace'])) == value for name, value in hashes.items())
out = base / 'root-r3-six-auditor-v2-controls.json'; assert not out.exists()
record = dict(passed=True, positives=6, negative_controls=10, results=results,
              helper_sha256=hashlib.sha256(path.read_bytes()).hexdigest(), actual_traces_unchanged=True,
              limits='Synthetic in-memory evidence mutations only; no campaign run, new native coverage or acceptance baseline change.')
out.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(dict(passed=True, positives=6, negatives=10, proof=str(out))))
