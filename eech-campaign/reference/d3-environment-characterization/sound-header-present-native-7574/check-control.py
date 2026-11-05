"""Independent check of the header-present boot control against its DECLARED fixture (reads files only; launches nothing).

usage: check-original-sound-header-present-20261101-0400.py <run output dir> [--out <new verdict json>]
       check-original-sound-header-present-20261101-0400.py --self-test        (exercises the checker itself; no evidence is produced)

It reads <out>/environment.jsonl.case15.raw (the complete native report sidecar) and fixture-declaration.json and compares row by row. Three classes, never merged:
  header_derived     consequences of the declared sound.hdr read through the original source (header count, matches, sizes, digest, attempts)
  carried_over       expectations carried from the closed header-ABSENT measurement (backend, DirectSound, flags); a difference is a NEW FINDING, not something to tune away
  consistency        the report agrees with itself and with the unchanged read-only fingerprint
The verdict lists every check with expected and observed values. A failed check is reported, never repaired. Exit 0 = all pass, 3 = at least one check failed, 1 = the input is unusable.
"""
import json
import re
import sys
from pathlib import Path

B = Path('R:/d3-evidence')
STEM = 'original-sound-header-present-20261101-0400'
DECLARATION = B / (STEM + '-fixture') / 'fixture-declaration.json'
BASELINE_DIGEST = '6605372942e1ea2c'


def parse(raw_text):
    rows = []
    for line in raw_text.split('\n'):
        if not line:
            continue
        parts = line.split(' ')
        fields = {}
        for token in parts[1:]:
            if '=' in token:
                k, v = token.split('=', 1)
                fields[k] = v
        rows.append((parts[0], fields, line))
    return rows


def check(raw_text, declaration):
    rows = parse(raw_text)
    by = {}
    for name, fields, line in rows:
        by.setdefault(name, []).append(fields)
    expected = declaration['expected']
    results = []

    def one(cls, label, want, got):
        results.append(dict(cls=cls, check=label, expected=want, observed=got, ok=(want == got)))

    def first(name, **where):
        for f in by.get(name, []):
            if all(f.get(k) == str(v) for k, v in where.items()):
                return f
        return {}

    store = first('env-sndinit-store')
    for k, want in (('boots', '1'), ('stored', '1'), ('dropped', '0'), ('orphans', '0'), ('opens_outside_effects', '0'), ('saturated', '0'), ('catalogue_entries', str(declaration['enum_slots']))):
        one('consistency', 'store.' + k, want, store.get(k))
    boot = first('env-sndinit-boot', n=0)
    for k in ('effects_begin', 'filled', 'effects_end', 'exited'):
        one('consistency', 'boot.' + k, '1', boot.get(k))
    one('header_derived', 'boot.open_true', '1', boot.get('open_true'))
    one('header_derived', 'boot.open_false', '0', boot.get('open_false'))
    one('header_derived', 'boot.header_count', str(expected['header_count']), boot.get('header_count'))
    one('header_derived', 'boot.matched', str(expected['matched']), boot.get('matched'))
    one('header_derived', 'boot.missed', str(expected['missed']), boot.get('missed'))
    carried = declaration['carried_over_from_the_closed_header_absent_report']
    one('carried_over', 'boot.no_sound', str(carried['no_sound']), boot.get('no_sound'))
    one('carried_over', 'boot.dedicated_at_init', str(carried['dedicated_at_init']), boot.get('dedicated'))
    one('carried_over', 'boot.backend', str(carried['backend_attempted']), boot.get('backend'))
    one('carried_over', 'boot.skipped', '0', boot.get('skipped'))
    one('carried_over', 'boot.load', str(carried['load']), boot.get('load'))
    one('carried_over', 'boot.enabled_exit', str(carried['enabled_exit']), boot.get('enabled_exit'))
    filled = first('env-sndinit-catalogue', n=0, when='filled')
    end = first('env-sndinit-catalogue', n=0, when='end')
    report = first('env-sndinit-catalogue', n='now', when='report')
    one('header_derived', 'catalogue.filled.nonzero_size', '0', filled.get('nonzero_size'))
    one('header_derived', 'catalogue.filled.digest_w', expected['digest_filled'], filled.get('digest_w'))
    for label, row in (('end', end), ('report', report)):
        one('header_derived', 'catalogue.%s.nonzero_size' % label, str(expected['nonzero_size']), row.get('nonzero_size'))
        one('header_derived', 'catalogue.%s.digest_w' % label, expected['digest_end_and_report'], row.get('digest_w'))
    for label, row in (('filled', filled), ('end', end), ('report', report)):
        one('consistency', 'catalogue.%s.named' % label, str(expected['catalogue_named']), row.get('named'))
        one('consistency', 'catalogue.%s.nonzero_rate' % label, str(expected['catalogue_named']), row.get('nonzero_rate'))
        one('consistency', 'catalogue.%s.nonzero_orate' % label, str(expected['catalogue_named']), row.get('nonzero_orate'))
    now = first('env-sndinit-now')
    one('carried_over', 'now.dedicated', str(carried['dedicated_at_report']), now.get('dedicated'))
    one('carried_over', 'now.enabled', '0', now.get('enabled'))
    one('carried_over', 'now.initialised', '0', now.get('initialised'))
    one('header_derived', 'now.blocked_count (number_of_blocked_sound_samples is not reset by close_application_sound_file_system)', str(expected['header_count']), now.get('blocked_count'))
    one('consistency', 'now.application_samples', str(expected['catalogue_named']), now.get('application_samples'))
    samples = first('env-sndinit-samples')
    one('header_derived', 'samples.lightning_size (FIRST of two records for the key)', str(expected['lightning_size']), samples.get('lightning_size'))
    one('header_derived', 'samples.thunder_size (matched, zero length)', str(expected['thunder_size']), samples.get('thunder_size'))
    for k in ('lightning_rate', 'lightning_orate', 'thunder_rate', 'thunder_orate'):
        one('consistency', 'samples.' + k, '22050', samples.get(k))
    sides = first('env-sndinit-load-side')
    one('carried_over', 'load_side.calls', str(carried['load_side_calls']), sides.get('calls'))
    header, data = first('env-sndinit-file', kind='header'), first('env-sndinit-file', kind='data')
    one('header_derived', 'file.header.attempts', str(expected['header_attempts']), header.get('attempts'))
    one('header_derived', 'file.header.ok', str(expected['header_ok']), header.get('ok'))
    one('header_derived', 'file.data.attempts', str(expected['data_attempts']), data.get('attempts'))
    one('header_derived', 'file.data.ok', str(expected['data_ok']), data.get('ok'))
    file_log = first('env-sndinit-file-log')
    one('header_derived', 'file_log.attempts_and_recorded', [str(expected['header_attempts'] + expected['data_attempts'])] * 2, [file_log.get('attempts'), file_log.get('recorded')])
    attempts = [f for f in by.get('env-sndinit-file-attempt', [])]
    one('header_derived', 'file_log.kinds_in_order', expected['file_log_kinds'], [f.get('kind') for f in attempts])
    one('header_derived', 'file_log.all_ok', ['1'] * len(attempts), [f.get('ok') for f in attempts])
    one('header_derived', 'file_log.no_installation_root_retry', ['0'] * len(attempts), [f.get('installation_root_retry') for f in attempts])
    com = {f.get('function'): f for f in by.get('env-sndinit-com', [])}
    one('carried_over', 'com.DirectSoundCreate', [str(carried['directsoundcreate_calls']), carried['directsoundcreate_return']], [com.get('DirectSoundCreate', {}).get('calls'), com.get('DirectSoundCreate', {}).get('last_return_w')])
    one('carried_over', 'com.other_calls', [str(carried['other_com_calls'])] * 3, [com.get(n, {}).get('calls') for n in ('IDirectSound_SetCooperativeLevel', 'IDirectSound_CreateSoundBuffer', 'IDirectSound_Release')])
    # every kept constructor sample row must carry the size the declared header gives its slot (zero for every slot not matched), and the descriptor's 22050 rates
    sizes = {int(k): v for k, v in expected['slot_sizes'].items()}
    sample_rows = by.get('env-sndinit-sample', [])
    bad = [f for f in sample_rows if int(f['size']) != sizes.get(int(f['index']), 0) or f['rate'] != '22050' or f['orate'] != '22050']
    one('header_derived', 'ctor sample rows: every kept key carries its declared size (rows kept: %d)' % len(sample_rows), [], bad)
    ctor = first('env-sndinit-ctor')
    one('consistency', 'ctor.distinct_equals_kept_rows', ctor.get('distinct'), str(len(sample_rows)))
    one('consistency', 'ctor.out_of_domain', '0', ctor.get('out_of_domain'))
    fp = first('env-sndinit-fingerprint')
    one('consistency', 'fingerprint.unchanged', '1', fp.get('unchanged'))
    one('consistency', 'end.read_only_unchanged', ['0', '1'], [first('env-sndinit-end').get('changed'), first('env-sndinit-end').get('read_only')])
    observed_sizes = {f['index']: f['size'] for f in sample_rows if f['size'] != '0'}
    return dict(results=results, ok=all(r['ok'] for r in results), failed=[r['check'] for r in results if not r['ok']], kept_sample_rows=len(sample_rows), kept_nonzero_size_keys=observed_sizes,
                declared_matched_keys_read_by_constructor=sorted(set(str(k) for k in sizes) & {f['index'] for f in sample_rows}), classes={c: sum(1 for r in results if r['cls'] == c) for c in ('header_derived', 'carried_over', 'consistency')})


def self_test(declaration):
    """A synthetic raw built from the retained header-ABSENT sidecar must fail where the header matters; the same rows rewritten to the declared values must pass. Evidence is never produced."""
    old = (B / 'environment-original-sound-init-recovery-7574caa2c-20261101/probe/environment.jsonl.case15.raw').read_text(encoding='utf-8')
    absent = check(old, declaration)
    assert not absent['ok'] and any(x.startswith('boot.open_true') for x in absent['failed']), absent['failed']
    e = declaration['expected']
    text = old
    text = text.replace('open_true=0 open_false=1 header_count=0 matched=0 missed=0', 'open_true=1 open_false=0 header_count=%d matched=%d missed=%d' % (e['header_count'], e['matched'], e['missed']))
    text = text.replace('digest_w=6605372942e1ea2c\nenv-sndinit-catalogue n=0 when=end named=2069 nonzero_size=0', 'digest_w=6605372942e1ea2c\nenv-sndinit-catalogue n=0 when=end named=2069 nonzero_size=%d' % e['nonzero_size'], 1)
    lines, out, seen_end = text.split('\n'), [], 0
    for line in lines:
        if line.startswith('env-sndinit-catalogue n=0 when=end') or line.startswith('env-sndinit-catalogue n=now when=report'):
            line = re.sub(r'nonzero_size=\d+', 'nonzero_size=%d' % e['nonzero_size'], line)
            line = re.sub(r'digest_w=[0-9a-f]+', 'digest_w=' + e['digest_end_and_report'], line)
        if line.startswith('env-sndinit-now'):
            line = line.replace('blocked_count=0', 'blocked_count=%d' % e['header_count'])
        if line.startswith('env-sndinit-sample '):
            index = re.search(r'index=(\d+)', line).group(1)
            if index in e['slot_sizes']:
                line = re.sub(r' size=\d+', ' size=%d' % e['slot_sizes'][index], line)
        if line.startswith('env-sndinit-samples'):
            line = line.replace('lightning_size=0', 'lightning_size=%d' % e['lightning_size'])
        if line.startswith('env-sndinit-file kind=header'):
            line = 'env-sndinit-file kind=header attempts=2 ok=2'
        if line.startswith('env-sndinit-file kind=data'):
            line = 'env-sndinit-file kind=data attempts=2 ok=2'
        if line.startswith('env-sndinit-file-log'):
            line = 'env-sndinit-file-log attempts=4 recorded=4'
        if line.startswith('env-sndinit-file-attempt'):
            continue
        if line.startswith('env-sndinit-file-log'):
            pass
        out.append(line)
        if line.startswith('env-sndinit-file-log'):
            for n, kind in enumerate(e['file_log_kinds']):
                out.append('env-sndinit-file-attempt n=%d kind=%s ok=1 installation_root_retry=0 read_mode=1' % (n, kind))
    synthetic = check('\n'.join(out), declaration)
    assert synthetic['ok'], synthetic['failed']
    # a first-match violation (the duplicate's length) and a wrong digest must each be caught
    wrong = '\n'.join(out).replace('lightning_size=%d' % e['lightning_size'], 'lightning_size=48')
    assert 'samples.lightning_size (FIRST of two records for the key)' in check(wrong, declaration)['failed']
    print('SELF-TEST OK: the retained header-absent report fails %d checks as it must, and a synthetic report with the declared values passes all %d' % (len(absent['failed']), len(synthetic['results'])))


def main(argv):
    declaration = json.loads(DECLARATION.read_text(encoding='utf-8'))
    if argv[1:] == ['--self-test']:
        self_test(declaration)
        return 0
    if len(argv) < 2:
        print(__doc__)
        return 1
    out = Path(argv[1])
    raw = out / 'environment.jsonl.case15.raw'
    if not raw.is_file():
        print('NO SIDECAR: ' + str(raw))
        return 1
    verdict = check(raw.read_bytes().decode('utf-8'), declaration)
    verdict.update(run=str(out), raw=str(raw), declaration=str(DECLARATION))
    if '--out' in argv:
        target = Path(argv[argv.index('--out') + 1])
        with target.open('x', encoding='utf-8', newline='\n') as f:
            f.write(json.dumps(verdict, indent=1) + '\n')
    for r in verdict['results']:
        if not r['ok']:
            print('FAILED [%s] %s: expected %r observed %r' % (r['cls'], r['check'], r['expected'], r['observed']))
    print('%s: %d checks, %d failed %s' % ('PASS' if verdict['ok'] else 'FAIL', len(verdict['results']), len(verdict['failed']), json.dumps(verdict['classes'])))
    return 0 if verdict['ok'] else 3


if __name__ == '__main__':
    sys.exit(main(sys.argv))
