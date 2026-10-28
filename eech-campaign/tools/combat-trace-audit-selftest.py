#!/usr/bin/env python3
"""Self-test of combat-trace-audit.py against a REAL complete trace (EEDCS-46).

usage: combat-trace-audit-selftest.py <complete trace.jsonl from lua/combat-characterization.lua> <its run.json> [work dir]

The base is a real trace (not a trace made by a copy of the emitter): the audit must accept it unchanged (exit 0), and it must reject each of a list of single, targeted
corruptions of that same file with the exit status that corruption deserves (1 a failure, 2 incomplete).  Each corruption is a byte-for-byte edit of one fact:
a hash written as a number, an integrity flag, a missing event, a wrong boot configuration, a missing final run event, a counter that goes down, a run that is not finished, and the typing of the cmb-kind `blast` counter (legacy decimal string, corrected number, mixed, another counter as a string, a hash-labelled `blast` as a number).
Writes the corrupted copies under the work directory (default: a temporary directory) and prints one line per case; exit 0 only if every case behaved as stated.
"""
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
AUDIT = os.path.join(HERE, 'combat-trace-audit.py')


def audit(trace, run_json, *extra):
    p = subprocess.run([sys.executable, '-B', AUDIT, trace, '--run-json', run_json, *extra], capture_output=True, text=True)
    return p.returncode, p.stdout


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    trace, run_json = sys.argv[1], sys.argv[2]
    work = sys.argv[3] if len(sys.argv) > 3 else tempfile.mkdtemp(prefix='combat-audit-selftest-')
    os.makedirs(work, exist_ok=True)
    rows = [json.loads(line) for line in open(trace, 'rb').read().split(b'\n') if line]
    done_json = json.load(open(run_json))
    results = []

    def write(name, new_rows):
        path = os.path.join(work, name + '.jsonl')
        with open(path, 'w', encoding='utf-8', newline='') as f:
            for r in new_rows:
                f.write(json.dumps(r, sort_keys=True, separators=(',', ':')) + '\n')
        return path

    def case(name, new_rows, want, run=run_json, extra=()):
        path = write(name, new_rows)
        code, out = audit(path, run, *extra)
        failing = [l for l in out.splitlines() if l.startswith('FAIL')][:2]
        results.append((name, code == want, f'exit {code} (wanted {want}) {failing}'))

    copy = lambda: [dict(r) for r in rows]

    code, out = audit(trace, run_json)
    results.append(('unchanged real trace', code == 0, f'exit {code} (wanted 0) {[l for l in out.splitlines() if l.startswith("FAIL")][:2]}'))

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-hash' and x.get('scope') == 'checkpoint')['timed'] = 1  # a hash written as a number
    case('hash_as_number', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-hash' and x.get('scope') == 'run')['order'] = 'ABCDEF0123456789'  # upper case is not what the emitter writes
    case('hash_not_lowercase_hex', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-integrity' and x.get('scope') == 'checkpoint')['trusted'] = 0
    case('untrusted_checkpoint', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-integrity' and x.get('scope') == 'run')['depth_overflow'] = 1
    case('nesting_overflow', r, 1)

    r = copy()
    logs = [i for i, x in enumerate(r) if x.get('scope') == 'log' and x.get('event') == 'cmb-ev']
    del r[logs[len(logs) // 2]]
    case('event_missing_from_the_middle', r, 1)

    r = copy()
    swap = [i for i, x in enumerate(r) if x.get('scope') == 'log' and x.get('kind') == 'blast-end']
    r[swap[0]]['kind'] = 'blast-begin'
    case('blast_pairing_broken', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'boot' and 'scope' not in x)['seconds'] = 300
    case('wrong_boot_configuration', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-damage' and x.get('scope') == 'checkpoint')['entries'] += 1
    case('damage_entries_unbalanced', r, 1)

    r = copy()
    cps = [x for x in r if x.get('event') == 'cmb-capability' and x.get('scope') == 'checkpoint']
    cps[-1]['calls'] = cps[0]['calls'] - 1  # a run-wide counter that goes down
    case('counter_decreases', r, 1)

    r = [x for x in copy() if not (x.get('event') == 'run' and 'scope' not in x)]
    case('no_final_run_event', r, 2)

    running = dict(done_json)
    running['status'] = 'running'
    running['actual_native_exit'] = None
    rj = os.path.join(work, 'run-running.json')
    json.dump(running, open(rj, 'w'))
    case('process_still_running', copy(), 2, run=rj)

    failed = dict(done_json)
    failed['actual_native_exit'] = 3
    rj2 = os.path.join(work, 'run-failed.json')
    json.dump(failed, open(rj2, 'w'))
    case('process_exited_nonzero', copy(), 1, run=rj2)

    case('partial_flag_never_exits_0', copy(), 2, extra=('--partial',))

    # the typing of the cmb-kind `blast` COUNTER: the real trace is the legacy f41 schema (decimal strings); the corrected parser writes numbers
    case('legacy_schema_requires_corrected_fails', copy(), 1, extra=('--require-corrected-schema',))

    r = copy()
    for x in r:
        if x.get('event') == 'cmb-kind' and isinstance(x.get('blast'), str):
            x['blast'] = int(x['blast'])
    case('corrected_schema_numbers_pass', r, 0)
    case('corrected_schema_numbers_pass_when_required', r, 0, extra=('--require-corrected-schema',))

    r = [dict(x) for x in r]
    kinds = [x for x in r if x.get('event') == 'cmb-kind']
    kinds[0]['blast'] = str(kinds[0]['blast'])  # one legacy string among numbers
    case('mixed_schema_fails', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-kind' and 'collisions' in x)['collisions'] = str(next(x for x in r if x.get('event') == 'cmb-kind' and 'collisions' in x)['collisions'])
    case('another_counter_as_string_fails', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-kind' and isinstance(x.get('blast'), str))['blast'] = 'abc'
    case('legacy_blast_not_decimal_fails', r, 1)

    r = copy()
    next(x for x in r if x.get('event') == 'cmb-hash' and x.get('scope') == 'run')['blast'] = 255  # the hash named like a counter must stay a string
    case('hash_blast_as_number_fails', r, 1)

    bad = 0
    for name, ok, detail in results:
        print(('ok   ' if ok else 'FAIL ') + name + ' :: ' + detail)
        bad += 0 if ok else 1
    print(f'AUDIT_SELFTEST={"ok" if not bad else "FAILED"}: {len(results)} cases, {bad} failures (work dir {work})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
