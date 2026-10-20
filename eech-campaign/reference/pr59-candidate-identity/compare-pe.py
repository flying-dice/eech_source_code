#!/usr/bin/env python3
"""Compares two builds of the same PE file (eech_dc.dll, eech-world.exe) region by region.

eech-campaign/docs/m1-baseline.md ("Rebuild check") found that a from-scratch rebuild reproduces
the code and data byte for byte, but not the file hash: the link timestamps (COFF header, export
directory), the header checksum and the COFF symbol table after the last section differ. This
tool repeats that byte comparison for two given files.

It prints each file's size and SHA-256, the full SHA-256 of each section's raw data, and the
number of differing bytes in each region:
  coff.TimeDateStamp, optional.CheckSum, export.TimeDateStamp  (link metadata: expected)
  COFF symbol/string table                                      (unmapped by the loader: expected)
  section <name>, headers or padding, data after last section  (NOT expected)
The export timestamp lies inside a mapped section (.edata in these builds), so that section's
hash differs; the section line then names the explained bytes.

The layout must agree between the two files, and each expected region is validated in each
file before it is accepted: the export directory must lie in the raw data of a section that
holds neither code nor executable memory, and the symbol and string table must hold at least one
symbol, start where the last section ends and, with its size field, run exactly to the end of
the file. Anything else is reported as not expected.

Exit 0 when every differing byte is in an expected region (or none differs), 1 when any is
not, 2 when a file cannot be read or is not a PE file this tool can read.

Usage: python -I compare-pe.py <a.dll> <b.dll>
"""
import hashlib
import struct
import sys
from collections import namedtuple

USAGE = 'Usage: python -I compare-pe.py <a.dll> <b.dll>'
SYMBOL_SIZE = 18
CODE_OR_EXECUTABLE = 0x00000020 | 0x20000000  # IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE
METADATA_FIELDS = ('coff.TimeDateStamp', 'optional.CheckSum', 'export.TimeDateStamp')
SYMBOL_TABLE = 'COFF symbol/string table'
EXPECTED = METADATA_FIELDS + (SYMBOL_TABLE,)


Section = namedtuple('Section', 'name va vsize rptr rsize characteristics')


class NotPE(Exception):
    pass


def unpack(fmt, data, off):
    if off < 0 or off + struct.calcsize(fmt) > len(data):
        raise NotPE(f'truncated at offset {off}')
    return struct.unpack_from(fmt, data, off)


def export_timestamp_offset(sections, exp_rva, exp_size):
    """File offset of the export directory's TimeDateStamp, or None without an export directory."""
    if not exp_rva:
        return None
    for s in sections:
        if s.rsize and s.va <= exp_rva and exp_rva + max(exp_size, 8) <= s.va + s.rsize:
            if s.characteristics & CODE_OR_EXECUTABLE:
                raise NotPE(f'export directory RVA {exp_rva:#x} is in code section {s.name}')
            return s.rptr + exp_rva - s.va + 4
    raise NotPE(f'export directory RVA {exp_rva:#x} is not inside any section\'s raw data')


def symbol_table(data, end, sym_ptr, nsym):
    """The (start, stop) of the COFF symbol and string table, if it starts at `end` and runs to EOF."""
    if not nsym or sym_ptr != end:
        return None
    strtab = sym_ptr + SYMBOL_SIZE * nsym
    if strtab + 4 > len(data):
        return None
    strtab_size, = unpack('<I', data, strtab)
    return (sym_ptr, len(data)) if strtab + strtab_size == len(data) else None


def parse(data):
    if data[:2] != b'MZ':
        raise NotPE('no MZ header')
    pe, = unpack('<I', data, 0x3C)
    if data[pe:pe + 4] != b'PE\0\0':
        raise NotPE('no PE signature')
    coff = pe + 4
    nsec, = unpack('<H', data, coff + 2)
    sym_ptr, nsym = unpack('<II', data, coff + 8)
    opt_size, = unpack('<H', data, coff + 16)
    opt = coff + 20
    magic, = unpack('<H', data, opt)
    if magic not in (0x10B, 0x20B):
        raise NotPE(f'unknown optional-header magic {magic:#x}')
    directories = opt + (112 if magic == 0x20B else 96)
    rva_count, = unpack('<I', data, directories - 4)
    has_export = rva_count >= 1 and opt_size >= directories + 8 - opt
    exp_rva, exp_size = unpack('<II', data, directories) if has_export else (0, 0)
    table = opt + opt_size
    sections = []
    for i in range(nsec):
        o = table + 40 * i
        name = data[o:o + 8].rstrip(b'\0').decode('ascii', 'replace')
        vsize, va, rsize, rptr = unpack('<IIII', data, o + 8)
        characteristics, = unpack('<I', data, o + 36)
        if rptr + rsize > len(data):
            raise NotPE(f'section {name} runs past the end of the file')
        sections.append(Section(name, va, vsize, rptr, rsize, characteristics))
    end = max((s.rptr + s.rsize for s in sections), default=table + 40 * nsec)
    fields = {'coff.TimeDateStamp': (coff + 4, 4), 'optional.CheckSum': (opt + 64, 4)}
    export = export_timestamp_offset(sections, exp_rva, exp_size)
    if export is not None:
        fields['export.TimeDateStamp'] = (export, 4)
    return {'sections': sections, 'fields': fields, 'end': end,
            'symbols': symbol_table(data, end, sym_ptr, nsym)}


def region(p, off):
    for name, (o, n) in p['fields'].items():
        if o <= off < o + n:
            return name
    for s in p['sections']:
        if s.rptr <= off < s.rptr + s.rsize:
            return f'section {s.name}'
    if p['symbols'] and p['symbols'][0] <= off < p['symbols'][1]:
        return SYMBOL_TABLE
    return 'data after last section' if off >= p['end'] else 'headers or padding'


def compare(a, b):
    """Prints the comparison; returns True when every differing byte is in an expected region."""
    pa, pb = parse(a), parse(b)
    if len(a) != len(b):
        print(f'SIZES DIFFER: {len(a)} vs {len(b)}')
        return False
    if pa != pb:
        print('LAYOUTS DIFFER (sections, field offsets or symbol table)')
        return False
    counts = {}
    for off in range(len(a)):
        if a[off] != b[off]:
            r = region(pa, off)
            counts[r] = counts.get(r, 0) + 1
    print('sections (name, raw size, raw-data SHA-256 a / b):')
    for name, _, _, rptr, rsize, _ in pa['sections']:
        ha = hashlib.sha256(a[rptr:rptr + rsize]).hexdigest()
        hb = hashlib.sha256(b[rptr:rptr + rsize]).hexdigest()
        if ha == hb:
            status = 'same'
        elif counts.get(f'section {name}'):
            status = 'DIFFERENT'
        else:
            inside = [f for f, (o, n) in pa['fields'].items() if rptr <= o < rptr + rsize and counts.get(f)]
            status = f'DIFFERENT, only in {", ".join(inside)}' if inside else 'DIFFERENT'
        print(f'  {name:8} {rsize:10}  {ha}')
        print(f'  {"":8} {"":10}  {hb}  {status}')
    print('differing bytes by region:' if counts else 'no differing bytes')
    for r, n in sorted(counts.items()):
        print(f'  {r}: {n} bytes{"" if r in EXPECTED else "  <- NOT EXPECTED"}')
    return all(r in EXPECTED for r in counts)


def main(argv):
    sys.stdout.reconfigure(encoding='utf-8', errors='backslashreplace')
    if len(argv) != 3:
        print(USAGE, file=sys.stderr)
        return 2
    try:
        a, b = (open(p, 'rb').read() for p in argv[1:3])
    except OSError as e:
        print(f'UNREADABLE: {e}')
        return 2
    for path, d in zip(argv[1:3], (a, b)):
        print(f'{path}: {len(d)} bytes, SHA-256 {hashlib.sha256(d).hexdigest()}')
    try:
        ok = compare(a, b)
    except NotPE as e:
        print(f'NOT A READABLE PE FILE: {e}')
        return 2
    print('RESULT: same code and data, apart from link metadata (COFF and export timestamps, checksum) '
          'and the unmapped COFF symbol table' if ok else 'RESULT: CODE OR DATA DIFFER')
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv))
