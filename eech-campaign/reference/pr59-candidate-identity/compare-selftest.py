#!/usr/bin/env python3
"""Shows that compare-pe.py, compare-report.py and compare-logs.py fail closed.

Each case builds two small synthetic inputs (minimal PE32+ and PE32 files, JSON files, log
directories), runs the tool on them under `python -I -X dev` and checks three things: the exit
status (0 for an explained or no difference, 1 for a reported difference, 2 for an input the tool
cannot read or bad arguments), that the output names the expected verdict or diagnostic, and that
stderr holds only the expected message: a traceback or a warning (such as an unclosed file) fails
the case. compare-pe.py is also run under `python -O`, so its checks do not depend on assert.
Needs no retail data or binaries.

Some cases cannot run everywhere, and are counted and named in the summary, never dropped:
  skipped by design   a Windows junction case off Windows; the unreadable-directory and
                      unreadable-file cases on Windows, where chmod cannot remove read access
  unavailable         a symbolic-link case where links cannot be made (Windows without the
                      privilege), or a permission case where chmod does not take (root)
With --strict (as in hosted CI) an unavailable case fails the run.

Exit 0 when every attempted case behaves as expected (and, with --strict, none is unavailable),
1 otherwise, 2 on bad arguments.

Usage: python -I compare-selftest.py [--strict]
"""
import json
import os
import struct
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
WINDOWS = os.name == 'nt'
SYMBOL_SIZE = 18
PE_SIG, COFF, OPT = 0x40, 0x44, 0x58
TEXT_RAW, EDATA_RAW, RAW_END = 0x200, 0x400, 0x600
TEXT_RVA, EDATA_RVA, BSS_RVA = 0x1000, 0x2000, 0x3000
CODE, DATA = 0x60000020, 0x40000040
SYMBOLS = 2
STRTAB = RAW_END + SYMBOL_SIZE * SYMBOLS
SAME_PE = 'RESULT: same code and data'
NOT_EXPECTED = '  <- NOT EXPECTED'


class Pe:
    """Offsets of a minimal image: .text, .edata (with an export directory), .bss, symbol table."""

    def __init__(self, pe32plus=True, opt_size=None):
        self.opt_size = opt_size or (240 if pe32plus else 224)
        self.magic = 0x20B if pe32plus else 0x10B
        self.export_dir = OPT + (112 if pe32plus else 96)
        self.section_table = OPT + self.opt_size

    def section_header(self, i):
        return self.section_table + 40 * i

    def build(self):
        b = bytearray(STRTAB + 8)
        b[0:2] = b'MZ'
        struct.pack_into('<I', b, 0x3C, PE_SIG)
        b[PE_SIG:PE_SIG + 4] = b'PE\0\0'
        struct.pack_into('<HHIIIHH', b, COFF, 0x8664, 3, 0x11111111, RAW_END, SYMBOLS, self.opt_size, 0x22)
        struct.pack_into('<H', b, OPT, self.magic)
        struct.pack_into('<I', b, OPT + 64, 0x33333333)
        if self.section_table >= self.export_dir + 8:
            struct.pack_into('<I', b, self.export_dir - 4, 16)  # NumberOfRvaAndSizes
            struct.pack_into('<II', b, self.export_dir, EDATA_RVA, 40)
        sections = [(b'.text', TEXT_RVA, 0x200, TEXT_RAW, CODE), (b'.edata', EDATA_RVA, 0x200, EDATA_RAW, DATA),
                    (b'.bss', BSS_RVA, 0, 0, DATA)]
        for i, (name, va, rsize, rptr, characteristics) in enumerate(sections):
            o = self.section_header(i)
            b[o:o + 8] = name.ljust(8, b'\0')
            struct.pack_into('<IIII', b, o + 8, 0x200, va, rsize, rptr)
            struct.pack_into('<I', b, o + 36, characteristics)
        b[TEXT_RAW:TEXT_RAW + 4] = b'\x90\x90\xc3\x00'
        struct.pack_into('<I', b, EDATA_RAW + 4, 0x44444444)
        b[RAW_END:STRTAB] = b'S' * (STRTAB - RAW_END)
        struct.pack_into('<I', b, STRTAB, 8)
        b[STRTAB + 4:STRTAB + 8] = b'str\0'
        return b


def flip(*offsets):
    def change(b):
        for off in offsets:
            b[off] ^= 0xFF
    return change


def put(fmt, off, value):
    return lambda b: struct.pack_into(fmt, b, off, value)


def both(*changes):
    def change(b):
        for c in changes:
            c(b)
    return change


def truncate(at):
    return lambda b: b.__delitem__(slice(at, None))


def run(tool, *args, flags=()):
    """(exit status, stdout, stderr) of the tool, under -X dev so that warnings reach stderr."""
    r = subprocess.run([sys.executable, '-I', '-X', 'dev', *flags, os.path.join(HERE, tool), *args],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    return r.returncode, r.stdout, r.stderr


def check(name, result, status, shown=(), hidden=(), stderr=''):
    """('ok' or 'FAIL', name, detail): the status, every `shown` text in stdout and no `hidden` one,
    and stderr empty or, when `stderr` is given, holding it and no traceback."""
    got, out, err = result
    shown, hidden = (shown,) if isinstance(shown, str) else shown, (hidden,) if isinstance(hidden, str) else hidden
    problems = [f'exit {got}, expected {status}'] if got != status else []
    problems += [f'output lacks {s!r}' for s in shown if s not in out]
    problems += [f'output has {h!r}' for h in hidden if h in out]
    if 'Traceback' in err or (stderr not in err if stderr else err):
        problems.append(f'stderr {err.strip()[-300:]!r}')
    return ('FAIL' if problems else 'ok', name, '; '.join(problems) or f'exit {got}')


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'wb') as f:
        f.write(data if isinstance(data, (bytes, bytearray)) else data.encode('utf-8'))
    return path


def pe_cases(d):
    plus, pe32 = Pe(), Pe(pe32plus=False)
    short_opt = Pe(opt_size=64)
    relink = flip(COFF + 4, OPT + 64, EDATA_RAW + 4, RAW_END)
    relinked = ('coff.TimeDateStamp: 1 bytes', 'optional.CheckSum: 1 bytes', 'export.TimeDateStamp: 1 bytes',
                'COFF symbol/string table: 1 bytes', 'DIFFERENT, only in export.TimeDateStamp', SAME_PE)
    trailing = 'data after last section: 1 bytes' + NOT_EXPECTED
    in_code = 'NOT A READABLE PE FILE: export directory RVA 0x1000 is in code section .text'
    no_section = 'is not inside any section\'s raw data'
    # (name, layout, change to both files, change to b only, exit, text the output must show)
    cases = [
        ('identical', plus, None, None, 0, ('no differing bytes', SAME_PE)),
        ('rebuild: timestamps, checksum, symbols', plus, None, relink, 0, relinked),
        ('PE32 rebuild: timestamps, checksum, symbols', pe32, None, relink, 0, relinked),
        ('one .text byte', plus, None, flip(TEXT_RAW + 1), 1, 'section .text: 1 bytes' + NOT_EXPECTED),
        ('one .edata byte beside the export timestamp', plus, None, flip(EDATA_RAW + 12), 1,
         'section .edata: 1 bytes' + NOT_EXPECTED),
        ('section-table byte', plus, None, flip(plus.section_header(0) + 36), 1, 'LAYOUTS DIFFER'),
        ('section name differs only in a non-ASCII byte', plus, put('B', plus.section_header(0) + 6, 0x80),
         put('B', plus.section_header(0) + 6, 0x81), 1, 'LAYOUTS DIFFER'),
        ('headers padding byte', plus, None, flip(plus.section_header(3) + 8), 1,
         'headers or padding: 1 bytes' + NOT_EXPECTED),
        ('symbol table absent in both (no pointer), trailing byte', plus, put('<I', COFF + 8, 0), flip(STRTAB + 7), 1,
         trailing),
        ('symbol table empty in both (0 symbols), trailing byte', plus, put('<I', COFF + 12, 0), flip(STRTAB + 7), 1,
         trailing),
        ('0 symbols in both, size field running to EOF, trailing byte', plus,
         lambda b: both(put('<I', COFF + 12, 0), put('<I', RAW_END, len(b) - RAW_END))(b), flip(STRTAB + 7), 1,
         trailing),
        ('symbol table not at the end of the sections in both, symbol byte', plus,
         both(put('<I', COFF + 8, RAW_END + SYMBOL_SIZE), put('<I', COFF + 12, SYMBOLS - 1)), flip(RAW_END + 20), 1,
         trailing),
        ('string-table size wrong in both, symbol byte', plus, put('<I', STRTAB, 99), flip(RAW_END), 1, trailing),
        ('string-table size under 4 in both, symbol byte', plus, put('<I', STRTAB, 2), flip(RAW_END), 1, trailing),
        ('string-table size field cut short in both, symbol byte', plus, truncate(STRTAB + 2), flip(RAW_END), 1,
         trailing),
        ('string-table size changed in b only (layout differs, sizes equal)', plus, None, put('<I', STRTAB, 99), 1,
         'LAYOUTS DIFFER'),
        ('four bytes appended', plus, None, lambda b: b.extend(b'XXXX'), 1, 'SIZES DIFFER'),
        ('0 symbols in both, four bytes appended to b (sizes differ, layouts equal)', plus, put('<I', COFF + 12, 0),
         lambda b: b.extend(b'XXXX'), 1, 'SIZES DIFFER: 1580 vs 1584'),
        ('export directory in .text, .text byte', plus, put('<I', plus.export_dir, TEXT_RVA), flip(TEXT_RAW + 4), 2,
         in_code),
        ('export directory in a CNT_CODE-only section', plus,
         both(put('<I', plus.section_header(0) + 36, 0x20), put('<I', plus.export_dir, TEXT_RVA)), None, 2, in_code),
        ('export directory in a MEM_EXECUTE-only section', plus,
         both(put('<I', plus.section_header(0) + 36, 0x20000000), put('<I', plus.export_dir, TEXT_RVA)), None, 2,
         in_code),
        ('export directory running past its section\'s raw data', plus, put('<I', plus.export_dir, EDATA_RVA + 0x1F0),
         None, 2, 'export directory RVA 0x21f0 ' + no_section),
        ('no data directories in both, export timestamp byte', plus, put('<I', plus.export_dir - 4, 0),
         flip(EDATA_RAW + 4), 1, 'section .edata: 1 bytes' + NOT_EXPECTED),
        ('export directory in .bss (no raw data)', plus, None, put('<I', plus.export_dir, BSS_RVA), 2,
         'export directory RVA 0x3000 ' + no_section),
        ('optional header too short for the checksum (section name over it)', short_opt,
         put('B', short_opt.section_header(0), 0x80), put('B', short_opt.section_header(0), 0x81), 2,
         'NOT A READABLE PE FILE: optional header of 64 bytes'),
        ('optional header too short for its 17 data directories, rebuild', plus, put('<I', plus.export_dir - 4, 17),
         relink, 2, 'NOT A READABLE PE FILE: optional header of 240 bytes cannot hold 17 data directories'),
        ('sections overlap in the file, .text byte under the export timestamp', plus,
         put('<I', plus.section_header(1) + 20, TEXT_RAW), flip(TEXT_RAW + 4), 2,
         'NOT A READABLE PE FILE: sections .text and .edata overlap in the file'),
        ('section raw data inside the headers', plus, put('<I', plus.section_header(0) + 20, 0x100), None, 2,
         'NOT A READABLE PE FILE: section .text overlaps the headers'),
        ('sections overlap in memory', plus, put('<I', plus.section_header(2) + 12, EDATA_RVA + 0x100), None, 2,
         'NOT A READABLE PE FILE: sections .edata and .bss overlap in memory'),
        ('no MZ header', plus, None, flip(0), 2, 'NOT A READABLE PE FILE: no MZ header'),
        ('shorter than the DOS header', plus, None, truncate(0x30), 2,
         'NOT A READABLE PE FILE: truncated at offset 60'),
        ('no PE signature', plus, None, flip(PE_SIG), 2, 'NOT A READABLE PE FILE: no PE signature'),
        ('unknown optional-header magic', plus, None, put('<H', OPT, 0x107), 2,
         'NOT A READABLE PE FILE: unknown optional-header magic 0x107'),
        ('section runs past the end of the file', plus, None, truncate(0x500), 2,
         'NOT A READABLE PE FILE: section .edata runs past the end of the file'),
        ('truncated inside the section table', plus, None, truncate(plus.section_header(1) + 10), 2,
         'NOT A READABLE PE FILE: section table runs past the end of the file'),
    ]
    results = []
    for name, pe, common, only_b, status, shown in cases:
        a, b = pe.build(), pe.build()
        if common:
            common(a)
            common(b)
        if only_b:
            only_b(b)
        pa, pb = write(os.path.join(d, 'a.dll'), a), write(os.path.join(d, 'b.dll'), b)
        for flags in ((), ('-O',)):
            label = f'compare-pe {name}{" (-O)" if flags else ""}'
            results.append(check(label, run('compare-pe.py', pa, pb, flags=flags), status, shown))
    results.append(check('compare-pe missing file', run('compare-pe.py', pa, os.path.join(d, 'none.dll')), 2,
                         'UNREADABLE'))
    results.append(check('compare-pe one argument', run('compare-pe.py', pa), 2, stderr='Usage'))
    return results


def report_cases(d):
    long_b = 'x' * 200 + 'B'
    cases = [
        ('equal', {'claims': {'M1.x': 'PASS'}}, {'claims': {'M1.x': 'PASS'}}, 0, '0 differing values'),
        ('PASS becomes FAIL', {'claims': {'M1.x': 'PASS'}}, {'claims': {'M1.x': 'FAIL'}}, 1, 'b: str "FAIL"'),
        ('null against absent', {'x': None}, {}, 1, 'b: absent'),
        ('empty list against absent', {'x': []}, {}, 1, 'a: list []'),
        ('empty list against empty object', {'x': []}, {'x': {}}, 1, 'b: dict {}'),
        ('dotted key against nesting', {'a.b': 1}, {'a': {'b': 1}}, 1, ('["a.b"]', '["a", "b"]')),
        ('key "0" against index 0', {'x': {'0': 1}}, {'x': [1]}, 1, ('["x", "0"]', '["x", 0]')),
        ('1 against true', {'n': 1}, {'n': True}, 1, 'b: bool true'),
        ('1 against 1.0', {'n': 1}, {'n': 1.0}, 1, 'b: float 1.0'),
        ('NaN against NaN', {'n': float('nan')}, {'n': float('nan')}, 0, '0 differing values'),
        ('non-ASCII value changes', {'s': 'Mi-17 → Beirut'}, {'s': 'Mi-17 → Halat'}, 1, 'b: str "Mi-17 → Halat"'),
    ]
    results = []
    for name, va, vb, status, shown in cases:
        pa = write(os.path.join(d, 'a.json'), json.dumps(va, ensure_ascii=False))
        pb = write(os.path.join(d, 'b.json'), json.dumps(vb, ensure_ascii=False))
        results.append(check(f'compare-report {name}', run('compare-report.py', pa, pb), status, shown))
    texts = [
        ('1e400 against 2e400', '{"n": 1e400}', '{"n": 2e400}', 1, 'b: float 2e400'),
        ('0.1 against 0.10000000000000000001', '{"n": 0.1}', '{"n": 0.10000000000000000001}', 1,
         'b: float 0.10000000000000000001'),
        ('duplicated key', '{"a": 1}', '{"a": 1, "a": 2}', 2, "UNREADABLE: duplicated key(s) ['a']"),
        ('two duplicated keys, named in order', '{"a": 1}', '{"b": 1, "a": 1, "b": 2, "a": 2}', 2,
         "UNREADABLE: duplicated key(s) ['a', 'b']"),
        ('malformed JSON', '{"a": 1}', '{not json', 2, 'UNREADABLE: Expecting property name'),
        ('nested 5000 deep', '{"a": 1}', '[' * 5000 + ']' * 5000, 2, 'UNREADABLE: maximum recursion depth'),
    ]
    for name, ta, tb, status, shown in texts:
        pa, pb = write(os.path.join(d, 'ta.json'), ta), write(os.path.join(d, 'tb.json'), tb)
        results.append(check(f'compare-report {name}', run('compare-report.py', pa, pb), status, shown))
    results.append(check('compare-report missing file', run('compare-report.py', pa, os.path.join(d, 'none.json')), 2,
                         'UNREADABLE'))
    results.append(check('compare-report one argument', run('compare-report.py', pa), 2, stderr='Usage'))
    a = write(os.path.join(d, 'la.json'), json.dumps({'d': 'x' * 200 + 'A'}))
    b = write(os.path.join(d, 'lb.json'), json.dumps({'d': long_b}))
    results.append(check('compare-report difference after 200 characters printed in full',
                         run('compare-report.py', a, b), 1, long_b))
    big = write(os.path.join(d, 'big.json'), json.dumps({f'k{n}': n for n in range(100000)}))
    start = time.monotonic()
    result = run('compare-report.py', big, big)
    seconds = time.monotonic() - start
    status, name, detail = check('compare-report 100,000 distinct keys within 10 s', result, 0, '0 differing values')
    results.append(('FAIL', name, f'{detail}; took {seconds:.1f} s') if seconds > 10 else (status, name, detail))
    return results


def tree(root, files):
    os.makedirs(root, exist_ok=True)
    for rel, data in files.items():
        if data is None:
            os.makedirs(os.path.join(root, rel), exist_ok=True)
        else:
            write(os.path.join(root, rel), data)
    return root


def linked_trees(d, tag, make_link, same_target):
    """Two trees whose only entry, sub/, links outside them to a directory holding sentinel.txt."""
    target_a = tree(os.path.join(d, f'{tag}-target-a'), {'sentinel.txt': 'PASS\n'})
    target_b = target_a if same_target else tree(os.path.join(d, f'{tag}-target-b'), {'sentinel.txt': 'FAIL\n'})
    a, b = tree(os.path.join(d, f'{tag}-a'), {}), tree(os.path.join(d, f'{tag}-b'), {})
    make_link(target_a, os.path.join(a, 'sub'))
    make_link(target_b, os.path.join(b, 'sub'))
    return a, b


def symlink(target, link):
    os.symlink(target, link, target_is_directory=True)


def junction(target, link):
    import _winapi  # Windows only
    _winapi.CreateJunction(target, link)


def link_cases(d):
    """Directory links must be reported and never entered: the targets' sentinel.txt must not appear."""
    results = []
    for kind, make_link in (('symbolic link', symlink), ('junction', junction)):
        for same in (False, True):
            name = f'compare-logs {kind} to {"the same directory" if same else "differing directories"}'
            if kind == 'junction' and not WINDOWS:
                results.append(('skip', name, 'junctions exist only on Windows'))
                continue
            try:
                trees = linked_trees(d, f'{kind[0]}{int(same)}', make_link, same)
            except OSError as e:
                results.append(('unavailable', name, f'cannot make the link: {e}'))
                continue
            results.append(check(name, run('compare-logs.py', *trees), 1,
                                 ('== sub: LINK, not followed or compared', '0 file(s) compared; 1 differ'),
                                 hidden='sentinel'))
    return results


def readable(path):
    try:
        if os.path.isdir(path):
            os.listdir(path)
        else:
            with open(path, 'rb'):
                pass
    except OSError:
        return False
    return True


def permission_cases(d):
    """An unreadable subdirectory and an unreadable file are errors (exit 2), not skipped entries."""
    cases = [('subdirectory', 'sub'), ('file', 'sub/x.txt')]
    results = []
    for what, locked in cases:
        name = f'compare-logs unreadable {what}'
        if WINDOWS:
            results.append(('skip', name, 'chmod cannot remove read access on Windows'))
            continue
        a = tree(os.path.join(d, f'u{what}-a'), {'top.txt': 'PASS\n', 'sub/x.txt': 'PASS\n'})
        b = tree(os.path.join(d, f'u{what}-b'), {'top.txt': 'PASS\n', 'sub/x.txt': 'FAIL\n'})
        paths = [os.path.join(root, locked) for root in (a, b)]
        try:
            for p in paths:
                os.chmod(p, 0)
            if readable(paths[0]):
                results.append(('unavailable', name, 'chmod did not remove read access (running as root?)'))
            else:
                results.append(check(name, run('compare-logs.py', a, b), 2, ('UNREADABLE', 'Permission denied')))
        finally:
            for p in paths:
                os.chmod(p, 0o755)
    return results


def logs_cases(d):
    base = {'regress.txt': 'georgia: PASS\r\n', 'sub/x.txt': 'PASS\n'}
    mask = ['--mask', r'pr(59|76)-candidate']
    cases = [
        ('identical (CRLF against LF)', base, {'regress.txt': 'georgia: PASS\n', 'sub/x.txt': 'PASS\n'}, [], 0,
         '2 file(s) compared; 0 differ'),
        ('line differs', base, {**base, 'regress.txt': 'georgia: FAIL\n'}, [], 1,
         ('== regress.txt: DIFFERS', '+georgia: FAIL')),
        ('file only in b', base, {**base, 'new-failure.txt': 'FAIL\n'}, [], 1, '== new-failure.txt: file ONLY IN b'),
        ('file only in a', base, {'regress.txt': 'georgia: PASS\n'}, [], 1, '== sub/x.txt: file ONLY IN a'),
        ('empty directory only in b', base, {**base, 'extra': None}, [], 1, '== extra: dir ONLY IN b'),
        ('file in a, directory in b', {'x': 'PASS\n'}, {'x': None}, [], 1, '== x: file in a, dir in b'),
        ('subdirectory file differs', base, {**base, 'sub/x.txt': 'FAIL\n'}, [], 1, '== sub/x.txt: DIFFERS'),
        ('lone CR against LF', {'r.txt': 'a\rb\n'}, {'r.txt': 'a\nb\n'}, [], 1, '== r.txt: DIFFERS'),
        ('missing final newline', {'r.txt': 'PASS\n'}, {'r.txt': 'PASS'}, [], 1, '== r.txt: DIFFERS'),
        ('candidate names, masked', {'r.txt': 'bin pr59-candidate\n'}, {'r.txt': 'bin pr76-candidate\n'}, mask, 0,
         '== r.txt: same after masking'),
        ('candidate names, unmasked', {'r.txt': 'bin pr59-candidate\n'}, {'r.txt': 'bin pr76-candidate\n'}, [], 1,
         '== r.txt: DIFFERS'),
        ('invalid UTF-8 bytes differ', {'r.bin': b'sha=\xff'}, {'r.bin': b'sha=\xfe'}, [], 1,
         '== r.bin: DIFFERS (not UTF-8, compared unmasked)'),
        ('invalid UTF-8 bytes equal', {'r.bin': b'sha=\xff'}, {'r.bin': b'sha=\xff'}, [], 0,
         '== r.bin: same bytes (not UTF-8, compared unmasked)'),
        ('non-ASCII line differs', {'r.txt': 'Mi-17 → Beirut\n'}, {'r.txt': 'Mi-17 → Halat\n'}, [], 1,
         ('-Mi-17 → Beirut', '+Mi-17 → Halat')),
        ('a empty, b has a file', {}, {'regress.txt': 'FAIL\n'}, [], 1, '== regress.txt: file ONLY IN b'),
        ('both empty', {}, {}, [], 2, 'NOTHING TO COMPARE'),
    ]
    results = []
    for i, (name, fa, fb, extra, status, shown) in enumerate(cases):
        trees = tree(os.path.join(d, f'la{i}'), fa), tree(os.path.join(d, f'lb{i}'), fb)
        results.append(check(f'compare-logs {name}', run('compare-logs.py', *trees, *extra), status, shown))
    results += link_cases(d) + permission_cases(d)
    long_a = tree(os.path.join(d, 'long-a'), {'r.txt': ''.join(f'{n}\n' for n in range(1, 41))})
    long_b = tree(os.path.join(d, 'long-b'), {'r.txt': ''.join(f'{n}\n' for n in range(101, 141))})
    result = run('compare-logs.py', long_a, long_b)
    status, name, detail = check('compare-logs long diff printed in full (80 lines)', result, 1, '== r.txt: DIFFERS')
    shown = sum(1 for line in result[1].splitlines() if line.strip()[:1] in '-+' and line.strip()[1:].isdigit())
    results.append(('FAIL', name, f'{detail}; {shown} lines shown') if shown != 80 else (status, name, detail))
    results.append(check('compare-logs missing directory', run('compare-logs.py', long_a, os.path.join(d, 'none')), 2,
                         stderr='both arguments must be directories'))
    results.append(check('compare-logs bad mask', run('compare-logs.py', long_a, long_b, '--mask', '('), 2,
                         stderr="bad regular expression '('"))
    results.append(check('compare-logs one argument', run('compare-logs.py', long_a), 2, stderr='usage:'))
    return results


def main(argv):
    sys.stdout.reconfigure(encoding='utf-8')
    strict = argv[1:] == ['--strict']
    if argv[1:] not in ([], ['--strict']):
        print('Usage: python -I compare-selftest.py [--strict]', file=sys.stderr)
        return 2
    with tempfile.TemporaryDirectory() as d:
        results = pe_cases(d) + report_cases(d) + logs_cases(d)
    counts = {}
    for status, name, detail in results:
        counts[status] = counts.get(status, 0) + 1
        print(f'{status:11}  {name}: {detail}')
    ok, failed = counts.get('ok', 0), counts.get('FAIL', 0)
    skipped, unavailable = counts.get('skip', 0), counts.get('unavailable', 0)
    strict_failure = ' (--strict: these fail the run)' if strict and unavailable else ''
    print(f'{ok + failed} attempted: {ok} as expected, {failed} not; '
          f'{skipped} skipped by design, {unavailable} unavailable{strict_failure}')
    return 1 if failed or (strict and unavailable) else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
