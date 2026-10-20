#!/usr/bin/env python3
"""Compares two builds of the same PE file (eech_dc.dll, eech-world.exe) region by region.

docs/m1-baseline.md ("Rebuild check") found that a from-scratch rebuild reproduces the code and
data byte for byte, but not the file hash: the link timestamps (COFF header, export directory),
the header checksum and the COFF symbol table after the last section differ. This tool repeats
that byte comparison for two given files, so the claim "same code and data" is checked, not assumed.

For each file it prints the size, the SHA-256 of the whole file and of each section's raw data,
and it locates every differing byte:
  coff.TimeDateStamp, optional.CheckSum, export.TimeDateStamp  (expected link metadata)
  section <name>                                                (code or data: NOT expected)
  headers (other)                                               (NOT expected)
  after last section (COFF symbol/string table)                 (unmapped by the loader)
Exit 0 when every difference is in an expected region (or none), 1 otherwise.

Usage: python -I compare-pe.py <a.dll> <b.dll>
"""
import hashlib
import struct
import sys


def parse(data):
    pe = struct.unpack_from('<I', data, 0x3C)[0]
    assert data[pe:pe + 4] == b'PE\0\0', 'not a PE file'
    coff = pe + 4
    nsec, = struct.unpack_from('<H', data, coff + 2)
    sym_ptr, nsym = struct.unpack_from('<II', data, coff + 8)
    opt_size, = struct.unpack_from('<H', data, coff + 16)
    opt = coff + 20
    magic, = struct.unpack_from('<H', data, opt)
    dd = opt + (112 if magic == 0x20B else 96)
    exp_rva, exp_size = struct.unpack_from('<II', data, dd)
    sections = []
    st = opt + opt_size
    for i in range(nsec):
        o = st + 40 * i
        name = data[o:o + 8].rstrip(b'\0').decode('ascii', 'replace')
        vsize, va, rsize, rptr = struct.unpack_from('<IIII', data, o + 8)
        sections.append((name, va, vsize, rptr, rsize))
    fields = {'coff.TimeDateStamp': (coff + 4, 4), 'optional.CheckSum': (opt + 64, 4)}
    if exp_rva:
        for name, va, vsize, rptr, rsize in sections:
            if va <= exp_rva < va + max(vsize, rsize):
                fields['export.TimeDateStamp'] = (rptr + exp_rva - va + 4, 4)
    end = max(rptr + rsize for _, _, _, rptr, rsize in sections)
    return {'sections': sections, 'fields': fields, 'end': end, 'headers_end': st + 40 * nsec, 'symbols': (sym_ptr, nsym)}


def region(p, off):
    for name, (o, n) in p['fields'].items():
        if o <= off < o + n:
            return name
    for name, va, vsize, rptr, rsize in p['sections']:
        if rptr <= off < rptr + rsize:
            return f'section {name}'
    if off >= p['end']:
        return 'after last section (COFF symbol/string table)'
    return 'headers (other)'


EXPECTED = ('coff.TimeDateStamp', 'optional.CheckSum', 'export.TimeDateStamp', 'after last section (COFF symbol/string table)')


def main():
    a_path, b_path = sys.argv[1:3]
    a, b = open(a_path, 'rb').read(), open(b_path, 'rb').read()
    pa, pb = parse(a), parse(b)
    ok = True
    for path, d in ((a_path, a), (b_path, b)):
        print(f'{path}: {len(d)} bytes, SHA-256 {hashlib.sha256(d).hexdigest()}')
    if len(a) != len(b):
        print(f'SIZES DIFFER: {len(a)} vs {len(b)}')
        ok = False
    if [s[:5] for s in pa['sections']] != [s[:5] for s in pb['sections']]:
        print('SECTION TABLES DIFFER')
        ok = False
    print('sections (name, raw size, raw-data SHA-256 a / b):')
    for (name, va, vsize, rptr, rsize), s2 in zip(pa['sections'], pb['sections']):
        ha = hashlib.sha256(a[rptr:rptr + rsize]).hexdigest()
        hb = hashlib.sha256(b[s2[3]:s2[3] + s2[4]]).hexdigest()
        print(f'  {name:10} {rsize:10}  {ha[:16]} / {hb[:16]}  {"same" if ha == hb else "DIFFERENT"}')
    counts = {}
    for off in range(min(len(a), len(b))):
        if a[off] != b[off]:
            r = region(pa, off)
            counts[r] = counts.get(r, 0) + 1
    print('differing bytes by region:' if counts else 'no differing bytes')
    for r, n in sorted(counts.items()):
        # the export directory's timestamp lies inside a section but is classified by its field first
        print(f'  {r}: {n} bytes{"" if r in EXPECTED else "  <- NOT EXPECTED"}')
        if r not in EXPECTED:
            ok = False
    print('RESULT: same code and data; only link metadata and the unmapped symbol table differ' if ok
          else 'RESULT: CODE OR DATA DIFFER')
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
