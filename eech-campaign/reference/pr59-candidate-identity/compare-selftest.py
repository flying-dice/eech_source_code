#!/usr/bin/env python3
"""Shows that compare-pe.py, compare-report.py and compare-logs.py fail closed.

Each case builds two small synthetic inputs (minimal PE32+ and PE32 files, JSON files, log
directories), runs the tool on them and checks its exit status: 0 for an explained or no
difference, 1 for a reported difference, 2 for an input the tool cannot read. compare-pe.py is
also run under `python -O`, so its checks do not depend on assert. Needs no retail data or
binaries. Cases that need a symbolic link are skipped, and say so, where links cannot be made.

Exit 0 when every case behaves as expected, 1 otherwise.

Usage: python -I compare-selftest.py
"""
import json
import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SYMBOL_SIZE = 18
PE_SIG, COFF, OPT = 0x40, 0x44, 0x58
TEXT_RAW, EDATA_RAW, RAW_END = 0x200, 0x400, 0x600
TEXT_RVA, EDATA_RVA, BSS_RVA = 0x1000, 0x2000, 0x3000
CODE, DATA = 0x60000020, 0x40000040
SYMBOLS = 2
STRTAB = RAW_END + SYMBOL_SIZE * SYMBOLS


class Pe:
    """Offsets of a minimal image: .text, .edata (with an export directory), .bss, symbol table."""

    def __init__(self, pe32plus=True):
        self.opt_size = 240 if pe32plus else 224
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


def run(tool, *args, flags=()):
    r = subprocess.run([sys.executable, '-I', *flags, os.path.join(HERE, tool), *args],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    return r.returncode, r.stdout


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'wb') as f:
        f.write(data if isinstance(data, (bytes, bytearray)) else data.encode('utf-8'))
    return path


def pe_cases(d):
    plus, pe32 = Pe(), Pe(pe32plus=False)
    relink = flip(COFF + 4, OPT + 64, EDATA_RAW + 4, RAW_END)
    # (name, layout, change to both files, change to b only, expected exit)
    cases = [
        ('identical', plus, None, None, 0),
        ('rebuild: timestamps, checksum, symbols', plus, None, relink, 0),
        ('PE32 rebuild: timestamps, checksum, symbols', pe32, None, relink, 0),
        ('one .text byte', plus, None, flip(TEXT_RAW + 1), 1),
        ('one .edata byte beside the export timestamp', plus, None, flip(EDATA_RAW + 12), 1),
        ('section-table byte', plus, None, flip(plus.section_header(0) + 36), 1),
        ('headers padding byte', plus, None, flip(plus.section_header(3) + 8), 1),
        ('symbol table absent in both (no pointer), trailing byte', plus, put('<I', COFF + 8, 0), flip(STRTAB + 7), 1),
        ('symbol table empty in both (0 symbols), trailing byte', plus, put('<I', COFF + 12, 0), flip(STRTAB + 7), 1),
        ('0 symbols in both, size field running to EOF, trailing byte', plus,
         lambda b: (put('<I', COFF + 12, 0)(b), put('<I', RAW_END, len(b) - RAW_END)(b)), flip(STRTAB + 7), 1),
        ('symbol table not at the end of the sections in both, symbol byte', plus,
         lambda b: (put('<I', COFF + 8, RAW_END + SYMBOL_SIZE)(b), put('<I', COFF + 12, SYMBOLS - 1)(b)), flip(RAW_END + 20), 1),
        ('string-table size wrong in both, symbol byte', plus, put('<I', STRTAB, 99), flip(RAW_END), 1),
        ('string-table size under 4 in both, symbol byte', plus, put('<I', STRTAB, 2), flip(RAW_END), 1),
        ('four bytes appended', plus, None, lambda b: b.extend(b'XXXX'), 1),
        ('export directory in .text, .text byte', plus, put('<I', plus.export_dir, TEXT_RVA), flip(TEXT_RAW + 4), 2),
        ('export directory in a CNT_CODE-only section', plus,
         lambda b: (put('<I', plus.section_header(0) + 36, 0x20)(b), put('<I', plus.export_dir, TEXT_RVA)(b)), None, 2),
        ('export directory in a MEM_EXECUTE-only section', plus,
         lambda b: (put('<I', plus.section_header(0) + 36, 0x20000000)(b), put('<I', plus.export_dir, TEXT_RVA)(b)), None, 2),
        ('no data directories in both, export timestamp byte', plus, put('<I', plus.export_dir - 4, 0), flip(EDATA_RAW + 4), 1),
        ('export directory in .bss (no raw data)', plus, None, put('<I', plus.export_dir, BSS_RVA), 2),
        ('no MZ header', plus, None, flip(0), 2),
        ('no PE signature', plus, None, flip(PE_SIG), 2),
        ('unknown optional-header magic', plus, None, put('<H', OPT, 0x107), 2),
        ('section runs past the end of the file', plus, None, lambda b: b.__delitem__(slice(0x500, None)), 2),
        ('truncated inside the section table', plus, None, lambda b: b.__delitem__(slice(plus.section_header(1), None)), 2),
    ]
    results = []
    for name, pe, common, only_b, want in cases:
        a, b = pe.build(), pe.build()
        if common:
            common(a)
            common(b)
        if only_b:
            only_b(b)
        pa, pb = write(os.path.join(d, 'a.dll'), a), write(os.path.join(d, 'b.dll'), b)
        for flags in ((), ('-O',)):
            got, _ = run('compare-pe.py', pa, pb, flags=flags)
            results.append((f'compare-pe {name}{" (-O)" if flags else ""}', want, got))
    results.append(('compare-pe missing file', 2, run('compare-pe.py', pa, os.path.join(d, 'none.dll'))[0]))
    return results


def report_cases(d):
    long_b = 'x' * 200 + 'B'
    cases = [
        ('equal', {'claims': {'M1.x': 'PASS'}}, {'claims': {'M1.x': 'PASS'}}, 0),
        ('PASS becomes FAIL', {'claims': {'M1.x': 'PASS'}}, {'claims': {'M1.x': 'FAIL'}}, 1),
        ('null against absent', {'x': None}, {}, 1),
        ('empty list against absent', {'x': []}, {}, 1),
        ('empty list against empty object', {'x': []}, {'x': {}}, 1),
        ('dotted key against nesting', {'a.b': 1}, {'a': {'b': 1}}, 1),
        ('key "0" against index 0', {'x': {'0': 1}}, {'x': [1]}, 1),
        ('1 against true', {'n': 1}, {'n': True}, 1),
        ('1 against 1.0', {'n': 1}, {'n': 1.0}, 1),
        ('NaN against NaN', {'n': float('nan')}, {'n': float('nan')}, 0),
        ('non-ASCII value changes', {'s': 'Mi-17 → Beirut'}, {'s': 'Mi-17 → Halat'}, 1),
    ]
    results = []
    for name, va, vb, want in cases:
        pa = write(os.path.join(d, 'a.json'), json.dumps(va, ensure_ascii=False))
        pb = write(os.path.join(d, 'b.json'), json.dumps(vb, ensure_ascii=False))
        results.append((f'compare-report {name}', want, run('compare-report.py', pa, pb)[0]))
    texts = [
        ('1e400 against 2e400', '{"n": 1e400}', '{"n": 2e400}', 1),
        ('0.1 against 0.10000000000000000001', '{"n": 0.1}', '{"n": 0.10000000000000000001}', 1),
        ('duplicated key', '{"a": 1}', '{"a": 1, "a": 2}', 2),
        ('malformed JSON', '{"a": 1}', '{not json', 2),
        ('nested 5000 deep', '{"a": 1}', '[' * 5000 + ']' * 5000, 2),
    ]
    for name, ta, tb, want in texts:
        pa, pb = write(os.path.join(d, 'ta.json'), ta), write(os.path.join(d, 'tb.json'), tb)
        results.append((f'compare-report {name}', want, run('compare-report.py', pa, pb)[0]))
    results.append(('compare-report missing file', 2, run('compare-report.py', pa, os.path.join(d, 'none.json'))[0]))
    a = write(os.path.join(d, 'la.json'), json.dumps({'d': 'x' * 200 + 'A'}))
    b = write(os.path.join(d, 'lb.json'), json.dumps({'d': long_b}))
    got, out = run('compare-report.py', a, b)
    results.append(('compare-report difference after 200 characters printed in full', (1, True), (got, long_b in out)))
    return results


def tree(root, files):
    os.makedirs(root, exist_ok=True)
    for rel, data in files.items():
        if data is None:
            os.makedirs(os.path.join(root, rel), exist_ok=True)
        else:
            write(os.path.join(root, rel), data)
    return root


def make_link(target, link):
    """A directory symbolic link, or on Windows without that privilege a junction."""
    try:
        os.symlink(target, link, target_is_directory=True)
    except OSError:
        import _winapi  # Windows only; ImportError elsewhere
        _winapi.CreateJunction(target, link)


def symlink_trees(d, i):
    """Two trees whose sub/ is a link to directories that differ, or None where links cannot be made."""
    real_a, real_b = tree(os.path.join(d, f'real-a{i}'), {'x.txt': 'PASS\n'}), tree(os.path.join(d, f'real-b{i}'), {'x.txt': 'FAIL\n'})
    a, b = tree(os.path.join(d, f'sa{i}'), {}), tree(os.path.join(d, f'sb{i}'), {})
    try:
        make_link(real_a, os.path.join(a, 'sub'))
        make_link(real_b, os.path.join(b, 'sub'))
    except (OSError, ImportError, NotImplementedError):
        return None
    return a, b


def unreadable_trees(d):
    """Two trees whose sub/ cannot be listed, or None where permissions cannot remove read access."""
    a = tree(os.path.join(d, 'ua'), {'top.txt': 'PASS\n', 'sub/x.txt': 'PASS\n'})
    b = tree(os.path.join(d, 'ub'), {'top.txt': 'PASS\n', 'sub/x.txt': 'FAIL\n'})
    for root in (a, b):
        os.chmod(os.path.join(root, 'sub'), 0)
    try:
        os.listdir(os.path.join(a, 'sub'))
    except OSError:
        return a, b
    for root in (a, b):
        os.chmod(os.path.join(root, 'sub'), 0o755)
    return None


def logs_cases(d):
    base = {'regress.txt': 'georgia: PASS\r\n', 'sub/x.txt': 'PASS\n'}
    mask = ['--mask', r'pr(59|76)-candidate']
    cases = [
        ('identical (CRLF against LF)', base, {'regress.txt': 'georgia: PASS\n', 'sub/x.txt': 'PASS\n'}, [], 0),
        ('line differs', base, {**base, 'regress.txt': 'georgia: FAIL\n'}, [], 1),
        ('file only in b', base, {**base, 'new-failure.txt': 'FAIL\n'}, [], 1),
        ('file only in a', base, {'regress.txt': 'georgia: PASS\n'}, [], 1),
        ('empty directory only in b', base, {**base, 'extra': None}, [], 1),
        ('subdirectory file differs', base, {**base, 'sub/x.txt': 'FAIL\n'}, [], 1),
        ('lone CR against LF', {'r.txt': 'a\rb\n'}, {'r.txt': 'a\nb\n'}, [], 1),
        ('missing final newline', {'r.txt': 'PASS\n'}, {'r.txt': 'PASS'}, [], 1),
        ('candidate names, masked', {'r.txt': 'bin pr59-candidate\n'}, {'r.txt': 'bin pr76-candidate\n'}, mask, 0),
        ('candidate names, unmasked', {'r.txt': 'bin pr59-candidate\n'}, {'r.txt': 'bin pr76-candidate\n'}, [], 1),
        ('invalid UTF-8 bytes differ', {'r.bin': b'sha=\xff'}, {'r.bin': b'sha=\xfe'}, [], 1),
        ('non-ASCII line differs', {'r.txt': 'Mi-17 → Beirut\n'}, {'r.txt': 'Mi-17 → Halat\n'}, [], 1),
        ('a empty, b has a file', {}, {'regress.txt': 'FAIL\n'}, [], 1),
        ('both empty', {}, {}, [], 2),
    ]
    results = []
    for i, (name, fa, fb, extra, want) in enumerate(cases):
        got, _ = run('compare-logs.py', tree(os.path.join(d, f'la{i}'), fa), tree(os.path.join(d, f'lb{i}'), fb), *extra)
        results.append((f'compare-logs {name}', want, got))
    linked = symlink_trees(d, len(cases))
    if linked:
        results.append(('compare-logs linked subdirectories that differ', 1, run('compare-logs.py', *linked)[0]))
    else:
        print('skip  compare-logs linked subdirectories: links cannot be made here')
    locked = unreadable_trees(d)
    if locked:
        try:
            results.append(('compare-logs unreadable subdirectory', 2, run('compare-logs.py', *locked)[0]))
        finally:
            for root in locked:
                os.chmod(os.path.join(root, 'sub'), 0o755)
    else:
        print('skip  compare-logs unreadable subdirectory: permissions cannot remove read access here')
    long_a = tree(os.path.join(d, 'long-a'), {'r.txt': ''.join(f'{n}\n' for n in range(1, 41))})
    long_b = tree(os.path.join(d, 'long-b'), {'r.txt': ''.join(f'{n}\n' for n in range(101, 141))})
    got, out = run('compare-logs.py', long_a, long_b)
    shown = sum(1 for line in out.splitlines() if line.strip().startswith(('-', '+')) and line.strip()[1:].isdigit())
    results.append(('compare-logs long diff printed in full (80 lines)', (1, 80), (got, shown)))
    results.append(('compare-logs missing directory', 2, run('compare-logs.py', long_a, os.path.join(d, 'none'))[0]))
    results.append(('compare-logs bad mask', 2, run('compare-logs.py', long_a, long_b, '--mask', '(')[0]))
    return results


def main():
    sys.stdout.reconfigure(encoding='utf-8')
    with tempfile.TemporaryDirectory() as d:
        results = pe_cases(d) + report_cases(d) + logs_cases(d)
    failed = 0
    for name, want, got in results:
        ok = want == got
        failed += not ok
        print(f'{"ok  " if ok else "FAIL"}  {name}: expected {want}, got {got}')
    print(f'{len(results) - failed} of {len(results)} cases as expected')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
