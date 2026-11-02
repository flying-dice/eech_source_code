"""Generate the ONE controlled valid sound.hdr / sound.dat fixture for the original-init header-present boot characterization.

Everything is derived from the frozen 7574caa2c source snapshot (the enum and the descriptor table of aphavoc/source/appsound), and the table parse is PROVEN before anything is written:
the all-sizes-zero catalogue it builds must reproduce the measured native report (2122 slots, 2069 named, FNV digest 6605372942e1ea2c). Nothing is guessed and no engine runs.
New exclusive names only (stem original-sound-header-present-20261101-0400); never touches a root, a manifest or an existing output.
"""
import hashlib
import json
import re
import struct
import sys
from pathlib import Path

B = Path('R:/d3-evidence')
STEM = 'original-sound-header-present-20261101-0400'
SNAPSHOT = B / 'environment-original-sound-init-source-7574caa2c'
SRC = SNAPSHOT / 'aphavoc/source/appsound'
OUT = B / (STEM + '-fixture')
MEASURED_BASELINE = dict(slots=2122, named=2069, rate=2069, orate=2069, size=0, digest='6605372942e1ea2c')
FNV_OFFSET, FNV_PRIME, M64 = 0xcbf29ce484222325, 0x100000001b3, (1 << 64) - 1


def strip_comments(text):
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.S)
    return re.sub(r'//[^\n]*', '', text)


def parse_enum():
    """SOUND_SAMPLE_INDEX_* in declaration order; the only preprocessor lines in the body are `#if !DEMO_VERSION` / `#endif` and project.h defines DEMO_VERSION 0 (checked), so every enumerator counts."""
    assert re.search(r'^#define DEMO_VERSION\s+0\b', (SNAPSHOT / 'aphavoc/source/project.h').read_text(encoding='latin-1'), re.M)
    text = strip_comments((SRC / 'snd_data.h').read_text(encoding='latin-1'))
    match = re.search(r'enum\s*\{(.*?)NUM_SOUND_SAMPLE_INDICES\s*,', text, re.S)
    body = match.group(1)
    lines = [x for x in body.splitlines() if not x.strip().startswith('#')]
    assert all(x.strip().startswith('#') or '=' not in x for x in body.splitlines()), 'an enumerator with an explicit value'
    names = re.findall(r'[A-Za-z_][A-Za-z_0-9]*', '\n'.join(lines))
    assert len(names) == len(set(names)) and all(n.startswith('SOUND_SAMPLE_INDEX_') for n in names)
    return names


def parse_table(index_of):
    text = strip_comments((SRC / 'snd_data.c').read_text(encoding='latin-1'))
    start = text.index('application_sound_samples[]')
    end = text.index('};', start)
    entries = []
    for m in re.finditer(r'\{\s*(SOUND_SAMPLE_INDEX_[A-Z0-9_]+)\s*,\s*(ENTITY_SIDE_[A-Z_]+)\s*,\s*(NULL|"[^"]*")\s*,\s*(\d+)\s*,', text[start:end]):
        name = None if m.group(3) == 'NULL' else m.group(3)[1:-1]
        entries.append(dict(symbol=m.group(1), index=index_of[m.group(1)], name=name, rate=int(m.group(4))))
    return entries


def fold(digest, word):
    for byte in range(4):
        digest ^= (word >> (8 * byte)) & 0xff
        digest = (digest * FNV_PRIME) & M64
    return digest


def catalogue(slots, count):
    named = sum(1 for s in slots.values() if s['name'] is not None)
    digest = FNV_OFFSET
    for i in range(count):
        s = slots.get(i, dict(name=None, rate=0, size=0, orate=0))
        for word in (i, s['rate'], s['size'], s['orate']):
            digest = fold(digest, word & 0xffffffff)
    return dict(named=named, nonzero_size=sum(1 for s in slots.values() if s['size']), nonzero_rate=sum(1 for s in slots.values() if s['rate']),
                nonzero_orate=sum(1 for s in slots.values() if s['orate']), digest='%016x' % digest)


def original_hash(name):
    """get_sound_block_header_index: hash += *ptr (a signed char), then hash <<= 1, 32-bit unsigned; used only as a filter before stricmp."""
    h = 0
    for ch in name.encode('ascii'):
        h = (h + (ch if ch < 128 else ch - 256)) & 0xffffffff
        h = (h << 1) & 0xffffffff
    return h


def find_first(records, filename, index):
    """The original's lookup: the FIRST record with the same hash, a case-insensitive equal name and the same sample index."""
    for position, record in enumerate(records):
        if original_hash(record['name']) == original_hash(filename) and record['name'].lower() == filename.lower() and record['index'] == index:
            return position
    return -1


def main():
    names = parse_enum()
    assert len(names) == MEASURED_BASELINE['slots'], len(names)
    index_of = {n: i for i, n in enumerate(names)}
    entries = parse_table(index_of)
    slots = {}
    for e in entries:  # initialise_application_sound_effects: later entries of one slot overwrite earlier ones; unnamed entries are skipped
        if e['name'] is not None:
            slots[e['index']] = dict(name=e['name'], rate=e['rate'], orate=e['rate'], size=0)
    base = catalogue(slots, len(names))
    assert (base['named'], base['nonzero_rate'], base['nonzero_orate'], base['nonzero_size'], base['digest']) == (
        MEASURED_BASELINE['named'], MEASURED_BASELINE['rate'], MEASURED_BASELINE['orate'], MEASURED_BASELINE['size'], MEASURED_BASELINE['digest']), (
        'the parsed table does not reproduce the measured native catalogue: ' + json.dumps(base))
    named_entries = [e for e in entries if e['name'] is not None]

    def slot_of(key):
        symbol = names[key]
        found = [e for e in named_entries if e['index'] == key]
        assert len(found) == 1 and slots[key]['rate'] == 22050, (key, symbol, found)
        return dict(key=key, symbol=symbol, name=found[0]['name'])

    thunder, lightning = slot_of(index_of['SOUND_SAMPLE_INDEX_AMBIENT_THUNDER']), slot_of(index_of['SOUND_SAMPLE_INDEX_AMBIENT_LIGHTNING'])
    # keys the constructor read in the retained complete native report (probe sidecar, first-seen order): 2051 is the first row, 2087 the first at or above 2069, 1954 the fifth
    ka, kb, kc = slot_of(2051), slot_of(2087), slot_of(1954)
    keys = [thunder, lightning, ka, kb, kc]
    assert len({k['key'] for k in keys}) == 5 and len({k['name'].lower() for k in keys}) == 5

    def rec(who, length, offset, index=None, filename=None, role=''):
        return dict(role=role, index=who['key'] if index is None else index, offset=offset, length=length, name=('audio\\' + who['name']) if filename is None else filename)

    records = [
        rec(thunder, 0, 0, role='matched, ZERO length'),
        rec(lightning, 96, 0, role='matched, nonzero length 96 (first of two records for this key and name)'),
        rec(kb, 40, 96, role='matched, nonzero length 40, key at or above 2069'),
        rec(ka, 24, 136, role='matched, nonzero length 24'),
        rec(lightning, 48, 160, role='DUPLICATE key and name, different length 48: the original must keep the FIRST match (96)'),
        rec(ka, 16, 0, filename='audio\\fixture-unmatched.pcm', role='valid record whose name is in no descriptor: never matched'),
        rec(kc, 16, 16, index=kb['key'], role='real name of a descriptor under ANOTHER sample index: the index does not match, so that descriptor stays unmatched'),
    ]
    data_size = max(r['offset'] + r['length'] for r in records)
    assert all(r['index'] in range(len(names)) and 0 <= r['offset'] and 0 <= r['length'] and r['offset'] + r['length'] <= data_size for r in records)
    assert all(r['name'].isascii() and '\0' not in r['name'] and len(r['name']) < 200 for r in records)

    # simulate initialise_application_sound_effects over the header exactly as the source reads it
    matched = missed = 0
    for e in named_entries:  # every entry that has a name, in table order
        position = find_first(records, 'audio\\' + e['name'], e['index'])
        if position >= 0:
            slots[e['index']]['size'] = records[position]['length']
            matched += 1
        else:
            missed += 1
    after = catalogue(slots, len(names))
    assert matched + missed == len(named_entries)

    header = struct.pack('<i', len(records))
    for r in records:
        encoded = r['name'].encode('ascii') + b'\0'
        header += struct.pack('<iiii', r['index'], r['offset'], r['length'], len(encoded)) + encoded
    data = bytes((i * 7 + 13) & 0xff for i in range(data_size))
    assert len(data) == data_size

    OUT.mkdir()
    for name, content in (('sound.hdr', header), ('sound.dat', data)):
        with (OUT / name).open('xb') as f:
            f.write(content)
    sha = lambda b: hashlib.sha256(b).hexdigest()
    expected_sizes = {str(i): s['size'] for i, s in sorted(slots.items()) if s['size']}
    declaration = dict(
        stem=STEM,
        scope='CONTROLLED INPUT VARIATION of the original sound initialization: one valid sound.hdr and sound.dat under cohokum/audio of a private retail Lebanon root. Not retail audio, not a Tacview or full-pack claim; generated only from the frozen 7574caa2c enum and descriptor table.',
        source_snapshot=str(SNAPSHOT), table_parse_proof=dict(measured_native_baseline=MEASURED_BASELINE, parsed_all_sizes_zero=base),
        named_table_entries=len(named_entries), enum_slots=len(names),
        header_format='little-endian int32 count; per record int32 sample_index, int32 data_offset, int32 data_length, int32 name_length (the NUL-terminated name INCLUDING the NUL), then the name bytes; names are `audio\\<table name>` exactly as snd_data.c sprintf builds them',
        records=records, data_bytes_rule='byte i = (i * 7 + 13) & 0xff; size = max(offset + length) = %d, so every offset + length is covered' % data_size,
        files={n: dict(bytes=len(c), sha256=sha(c)) for n, c in (('sound.hdr', header), ('sound.dat', data))},
        keys={k['symbol']: dict(key=k['key'], name=k['name']) for k in keys},
        expected=dict(
            header_count=len(records), matched=matched, missed=missed, nonzero_size=after['nonzero_size'], catalogue_named=after['named'], digest_filled=base['digest'], digest_end_and_report=after['digest'],
            lightning_size=96, thunder_size=0, slot_sizes=expected_sizes, header_attempts=2, data_attempts=2, header_ok=2, data_ok=2, file_log_kinds=['header', 'data', 'header', 'data'],
            rule='initialise_application_sound_effects tries file_exist(hdr), file_exist(dat), safe_fopen(hdr), safe_fopen(dat); each succeeds on the first, relative `audio\\...` path (the engine has chdir-ed into <root>/cohokum), so four attempts and no retry; the data file is opened but not read'),
        carried_over_from_the_closed_header_absent_report=dict(
            no_sound=0, dedicated_at_init=0, dedicated_at_report=1, backend_attempted=1, directsoundcreate_calls=1, directsoundcreate_return='80004005', other_com_calls=0, enabled_exit=0, initialised_exit=0, load=0, load_side_calls=1,
            note='These are expectations carried from the header-absent measurement, not consequences of the fixture; a difference is a new finding to investigate, never something to tune away.'),
        seconds=0, frame_ms=100,
    )
    with (OUT / 'fixture-declaration.json').open('xb') as f:
        f.write((json.dumps(declaration, indent=1) + '\n').encode('utf-8'))
    print(json.dumps(dict(out=str(OUT), files=declaration['files'], matched=matched, missed=missed, nonzero_size=after['nonzero_size'], digest_end=after['digest'], expected_sizes=expected_sizes)))


if __name__ == '__main__':
    sys.exit(main())
