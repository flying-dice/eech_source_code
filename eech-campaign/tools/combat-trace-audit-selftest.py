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

    # a final report whose run event is gone is inconsistent (the event is written before the report): a failure, not an incomplete run
    r = [x for x in copy() if not (x.get('event') == 'run' and 'scope' not in x)]
    case('final_report_without_its_run_event_fails', r, 1)
    # a genuinely unfinished run: neither the run event nor the final report
    r = [x for x in copy() if not (x.get('event') == 'run' and 'scope' not in x) and x.get('scope') != 'run']
    case('no_final_run_event_and_no_final_report_is_incomplete', r, 2)

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

    # ---- the REQUIRED report structure: rows removed or repeated in the real trace must fail, never pass on the rows that remain
    def report_rows(rr, scope, at=None, event=None):
        return [i for i, x in enumerate(rr) if x.get('scope') == scope and str(x.get('event', '')).startswith('cmb-') and (at is None or x.get('at') == at) and (event is None or x.get('event') == event)]

    def drop(rr, indexes):
        gone = set(indexes)
        return [x for i, x in enumerate(rr) if i not in gone]

    r = copy()
    case('structure_all_damage_rows_removed', drop(r, [i for i, x in enumerate(r) if x.get('event') == 'cmb-damage']), 1)
    r = copy()
    case('structure_all_checkpoint_integrity_rows_removed', drop(r, report_rows(r, 'checkpoint', event='cmb-integrity')), 1)
    r = copy()
    case('structure_one_singleton_removed_from_a_middle_checkpoint', drop(r, report_rows(r, 'checkpoint', 300, 'cmb-capability')), 1)
    r = copy()
    case('structure_one_timed_kind_removed', drop(r, [i for i in report_rows(r, 'checkpoint', 180, 'cmb-timed') if r[i].get('kind') == 3]), 1)
    r = copy()
    i = report_rows(r, 'checkpoint', 120, 'cmb-explosion')[0]
    r.insert(i + 1, dict(r[i]))
    case('structure_singleton_duplicated', r, 1)
    r = copy()
    i = [j for j in report_rows(r, 'checkpoint', 240, 'cmb-timed') if r[j].get('kind') == 5][0]
    r.insert(i + 1, dict(r[i]))
    case('structure_timed_kind_duplicated', r, 1)
    r = copy()
    case('structure_a_whole_checkpoint_report_removed_marker_kept', drop(r, report_rows(r, 'checkpoint', 300)), 1)
    r = copy()
    g = report_rows(r, 'checkpoint', 360)
    after_group = g[-1] + 1
    for k, j in enumerate(g):
        r.insert(after_group + k, dict(r[j]))
    case('structure_a_whole_report_repeated_contiguously', r, 1)
    r = copy()
    g = report_rows(r, 'checkpoint', 60)
    nxt = report_rows(r, 'checkpoint', 120)[-1] + 1
    for k, j in enumerate(g):
        r.insert(nxt + k, dict(r[j]))
    case('structure_a_whole_report_repeated_later', r, 1)
    r = copy()
    r.insert(report_rows(r, 'checkpoint', 60)[-1] + 1, {'event': 'cmb-bogus', 'scope': 'checkpoint', 'at': 60})
    case('structure_unknown_report_row', r, 1)
    r = copy()
    j = next(i for i, x in enumerate(r) if x.get('event') == 'checkpoint' and x.get('at') == 420 and 'scope' not in x)
    del r[j]
    case('structure_a_checkpoint_marker_removed_report_kept', r, 1)
    r = copy()
    case('structure_boot_report_removed', drop(r, report_rows(r, 'boot')), 1)
    r = copy()
    case('structure_final_report_missing_a_timed_kind', drop(r, [i for i in report_rows(r, 'run', event='cmb-timed') if r[i].get('kind') == 6]), 1)
    r = copy()
    case('structure_final_report_missing_integrity', drop(r, report_rows(r, 'run', event='cmb-integrity')), 1)
    r = copy()
    j = next(i for i, x in enumerate(r) if x.get('scope') == 'checkpoint' and x.get('at') == 540 and x.get('event') == 'cmb-hash')
    k = next(i for i, x in enumerate(r) if x.get('event') == 'checkpoint' and x.get('at') == 540 and 'scope' not in x)
    r[j], r[k] = r[k], r[j]
    case('structure_a_report_before_its_marker', r, 1)

    # a run still in progress: the last report may be cut short (INCOMPLETE, never PASS, never FAIL for that alone); a hole in any earlier report is still a failure
    full = copy()
    cut = report_rows(full, 'checkpoint', 300, 'cmb-capability')[0]
    truncated = full[:cut + 1]  # ends inside the 300 s report; no run event
    case('partial_truncated_inside_the_last_report_is_incomplete', truncated, 2, extra=('--partial',))
    case('partial_without_the_flag_is_still_incomplete_not_a_pass', truncated, 2)
    hole = [x for i, x in enumerate(truncated) if not (x.get('scope') == 'checkpoint' and x.get('at') == 180 and x.get('event') == 'cmb-damage')]
    case('partial_a_hole_in_an_earlier_report_fails', hole, 1, extra=('--partial',))
    no_final_rows = copy()
    case('complete_run_with_a_cut_final_report_fails', drop(no_final_rows, report_rows(no_final_rows, 'run', event='cmb-blast')), 1)

    # ---- the bounded reader: the REAL read path with an instrumented file; every request is at most min(line limit, bytes left) + 1 and nothing is ever read whole
    import importlib.util
    import io
    spec = importlib.util.spec_from_file_location('combat_trace_audit', AUDIT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    class Spy(io.BytesIO):
        def __init__(self, data):
            super().__init__(data)
            self.requests, self.returned = [], []

        def readline(self, size=-1):
            self.requests.append(size)
            got = super().readline(size)
            self.returned.append(len(got))
            return got

        def read(self, size=-1):
            self.requests.append(('read', size))
            return super().read(size)

    def bounded(name, data, max_bytes, line_max, want_problem, want_rows):
        spy = Spy(data)
        rows_out, problems_out, size_out = module.read_rows(spy, max_bytes, line_max)
        asked = [x for x in spy.requests if not isinstance(x, tuple)]
        other = [x for x in spy.requests if isinstance(x, tuple)]
        cap = line_max + 1
        ok = (not other) and all(isinstance(x, int) and 0 < x <= cap for x in asked) and max(spy.returned or [0]) <= cap
        ok = ok and (bool([p for p in problems_out if want_problem in p]) if want_problem else not problems_out) and len(rows_out) == want_rows
        results.append((name, ok, f'requests<={max(asked or [0])} returned<={max(spy.returned or [0])} cap {cap} problems {problems_out} rows {len(rows_out)}'))

    good = b'{"a":1}\n' * 5
    bounded('reader_reads_small_valid_lines_fully', good, 10_000, 1_000, None, 5)
    bounded('reader_rejects_an_oversized_unterminated_line_without_reading_it_whole', b'x' * 2_000_000, 100_000_000, 1_000, 'longer than 1000', 0)
    bounded('reader_rejects_an_oversized_terminated_line_after_good_lines', good + b'y' * 5_000 + b'\n' + good, 100_000_000, 1_000, 'longer than 1000', 5)
    bounded('reader_stops_at_the_total_limit', good * 100, 100, 1_000, 'larger than --max-bytes', 5 - 5 + 12)
    bounded('reader_total_limit_inside_a_line', b'{"a":' + b'1' * 400 + b'}\n', 100, 1_000, 'larger than --max-bytes', 0)

    # the same through the command line on a file on disk
    big = os.path.join(work, 'long-line.jsonl')
    with open(big, 'wb') as f:
        f.write(b'{"event":"boot"}\n')
        f.write(b'z' * 3_000_000)
    code, out = audit(big, run_json, '--line-max', '1000')
    results.append(('cli_line_limit_fails_a_long_unterminated_line', code == 1 and 'longer than 1000' in out, f'exit {code}'))
    code, out = audit(big, run_json, '--max-bytes', '200')
    results.append(('cli_total_limit_fails_a_long_line_under_the_default_line_limit', code == 1 and 'larger than --max-bytes' in out, f'exit {code}'))

    bad = 0
    for name, ok, detail in results:
        print(('ok   ' if ok else 'FAIL ') + name + ' :: ' + detail)
        bad += 0 if ok else 1
    print(f'AUDIT_SELFTEST={"ok" if not bad else "FAILED"}: {len(results)} cases, {bad} failures (work dir {work})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
