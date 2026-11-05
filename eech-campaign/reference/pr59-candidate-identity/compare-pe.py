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

Before any byte is excused, each file must be a PE file with an unambiguous layout, or it is
rejected as unreadable (exit 2): the optional header must hold its fixed fields (so the checksum
lies inside it) and every data directory it declares; the section table must lie inside the
file, and each section's raw data too; no section's raw data may overlap the headers or another
section's, and no two sections may overlap in memory; an export directory must lie in the raw
data of a section that holds neither code nor executable memory. Then the two files must have
the same size and the same layout (section names as raw bytes, addresses, sizes, flags, field
offsets, symbol table), or the result is a difference (exit 1). The symbol and string table is
excused only if it holds at least one symbol, starts where the last section ends and, with its
size field, runs exactly to the end of the file; otherwise its bytes count as data after the
last section. Every other differing byte is a difference.

Exit 0 when every differing byte is in an expected region (or none differs), 1 when any is
not, 2 when a file cannot be read or is not a PE file this tool can read, or on bad arguments.

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


def label(section):
    return section.name.rstrip(b'\0').decode('ascii', 'backslashreplace')


def read_section(data, o):
    name = data[o:o + 8]
    vsize, va, rsize, rptr = unpack('<IIII', data, o + 8)
    characteristics, = unpack('<I', data, o + 36)
    s = Section(name, va, vsize, rptr, rsize, characteristics)
    if rptr + rsize > len(data):
        raise NotPE(f'section {label(s)} runs past the end of the file')
    return s


def reject_overlap(sections, extent, where):
    """Raises NotPE when any two of the sections' (start, size) extents overlap."""
    ordered = sorted(sections, key=lambda s: extent(s)[0])
    for x, y in zip(ordered, ordered[1:]):
        start, size = extent(x)
        if extent(y)[0] < start + size:
            raise NotPE(f'sections {label(x)} and {label(y)} overlap {where}')


def check_layout(sections, headers_end):
    """Raises NotPE when a section's raw data starts inside the headers, or two sections overlap in the file
    or in memory."""
    in_file = [s for s in sections if s.rsize]
    for s in in_file:
        if s.rptr < headers_end:
            raise NotPE(f'section {label(s)} overlaps the headers')
    reject_overlap(in_file, lambda s: (s.rptr, s.rsize), 'in the file')
    reject_overlap(sections, lambda s: (s.va, max(s.vsize, s.rsize)), 'in memory')


def export_timestamp_offset(sections, exp_rva, exp_size):
    """File offset of the export directory's TimeDateStamp, or None without an export directory."""
    if not exp_rva:
        return None
    for s in sections:
        if s.rsize and s.va <= exp_rva and exp_rva + max(exp_size, 8) <= s.va + s.rsize:
            if s.characteristics & CODE_OR_EXECUTABLE:
                raise NotPE(f'export directory RVA {exp_rva:#x} is in code section {label(s)}')
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
    fixed = 112 if magic == 0x20B else 96  # the optional header before its data directories
    if opt_size < fixed:
        raise NotPE(f'optional header of {opt_size} bytes is shorter than its {fixed} bytes of fixed fields')
    rva_count, = unpack('<I', data, opt + fixed - 4)
    if fixed + 8 * rva_count > opt_size:
        raise NotPE(f'optional header of {opt_size} bytes cannot hold {rva_count} data directories')
    exp_rva, exp_size = unpack('<II', data, opt + fixed) if rva_count else (0, 0)
    table = opt + opt_size
    headers_end = table + 40 * nsec
    if headers_end > len(data):
        raise NotPE('section table runs past the end of the file')
    sections = [read_section(data, table + 40 * i) for i in range(nsec)]
    check_layout(sections, headers_end)
    end = max((s.rptr + s.rsize for s in sections), default=headers_end)
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
            return f'section {label(s)}'
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
    for s in pa['sections']:
        name, rptr, rsize = label(s), s.rptr, s.rsize
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


def read(path):
    with open(path, 'rb') as f:
        return f.read()


def main(argv):
    sys.stdout.reconfigure(encoding='utf-8', errors='backslashreplace')
    if len(argv) != 3:
        print(USAGE, file=sys.stderr)
        return 2
    try:
        a, b = read(argv[1]), read(argv[2])
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
