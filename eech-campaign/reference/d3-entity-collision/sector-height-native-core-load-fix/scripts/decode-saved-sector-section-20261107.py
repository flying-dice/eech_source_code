"""Read-only characterisation of a saved server-session data file against the reports of the real save and the real load that used it (EEDCS-49 D3, ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1).

It re-implements ONLY the sector section of `pack_local_sector_data` / `unpack_local_sector_data` (aphavoc/source/entity/special/sector/sc_pack.c) over the generic C bit routines of en_pack.c (`pack_bit` / `unpack_bit`: bits are
appended most-significant first inside each byte, and a value's least significant bit goes first): the side RLE (2-bit side, 16-bit length, terminated by length 0), the fog of war (two 16-bit values per sector, FLOAT_TYPE_FOG_OF_WAR
15+1 bits), then per sector the two VALID flags (1 bit each, a 20-bit entity index follows a set one), the 8-bit keysite count and the 32-bit raw height word. It compares the decoded words with the save report's `sh-save-word` rows and
with the load report's `sh-load-restore` rows. It writes AGGREGATES only (counts, equalities, hashes): no word, sector coordinate or retail value is copied into the result.
Usage: decode.py <saved .sav> <save witness.txt> <load witness.txt> <out.json>"""
import hashlib
import json
import sys

sav_path, save_report, load_report, out = sys.argv[1:5]
raw = open(sav_path, "rb").read()
data = raw[4:]  # the first int is the version number the loader strips before unpack_session
position = 0


def bits(count):
    global position
    value = 0
    for i in range(count):
        byte = data[position >> 3]
        value |= ((byte >> (7 - (position & 7))) & 1) << i
        position += 1
    return value


side, length = bits(2), bits(16)
total, runs = 0, 0
while length != 0:
    total += length
    runs += 1
    side, length = bits(2), bits(16)
rle_end = position
fog = [bits(16) for _ in range(total * 2)]
fog_end = position
words = []
for _ in range(total):
    if bits(1):
        bits(20)
    if bits(1):
        bits(20)
    bits(8)
    words.append(bits(32))
sector_end = position


def rows(path, prefix):
    result = []
    for line in open(path, encoding="utf-8"):
        if line.startswith(prefix + " "):
            result.append(dict(token.split("=", 1) for token in line.split()[1:]))
    return result


saved = rows(save_report, "sh-save-word")
restored = rows(load_report, "sh-load-restore")
saved_words = [int(r["word"], 16) for r in sorted(saved, key=lambda r: int(r["i"]))]
restored_words = [int(r["stated"], 16) for r in sorted(restored, key=lambda r: int(r["i"]))]
result = {
    "kind": "read-only decode of the sector section of the saved data against the save and load reports (aggregates only)",
    "saved_data": {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "version_int": int.from_bytes(raw[:4], "little"), "ctrl_z_bytes": raw.count(b"\x1a"), "crlf_pairs": raw.count(b"\r\n"), "first_ctrl_z_offset": raw.find(b"\x1a")},
    "decoded": {"rle_runs": runs, "rle_sector_total": total, "rle_end_bit": rle_end, "fog_values": len(fog), "fog_nonzero": sum(1 for f in fog if f), "fog_end_bit": fog_end, "sector_section_end_bit": sector_end, "sector_section_end_byte_after_version": sector_end / 8,
                "first_sector_word_byte_offset_in_file": (fog_end + 10) // 8 + 4, "words": len(words), "words_nonzero": sum(1 for w in words if w)},
    "save_report": {"word_rows": len(saved_words), "words_nonzero": sum(1 for w in saved_words if w)},
    "load_report": {"restore_rows": len(restored_words), "restored_words_nonzero": sum(1 for w in restored_words if w), "restored_words_all_zero": all(w == 0 for w in restored_words)},
    "joins": {
        "decoded_words_equal_save_report_in_order": words == saved_words,
        "restored_words_equal_save_report_in_order": restored_words == saved_words,
        "sectors_whose_restored_word_differs_from_the_saved_word": sum(1 for a, b in zip(saved_words, restored_words) if a != b),
    },
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=1))
