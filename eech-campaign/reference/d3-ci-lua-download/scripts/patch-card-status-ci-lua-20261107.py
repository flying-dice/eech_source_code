"""Metadata-only: STATUS top summary and the active card for the CI Lua pinned-download repair. Textual replacement (no JSON round trip); the card keeps its CRLF line endings. Usage: patch.py <repo> <code-commit>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
CODE = sys.argv[2]
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
card = card_path.read_bytes().decode("utf-8")
status = status_path.read_bytes().decode("utf-8")
PK = "reference/d3-ci-lua-download"
FREEZE = "R:/d3-evidence/D3-CI-LUA-PINNED-DOWNLOAD-CODE-FROZEN-20261107.json"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
freeze = json.loads(pathlib.Path(FREEZE).read_text(encoding="utf-8"))
assert not freeze["hygiene_problems_at_freeze"], freeze["hygiene_problems_at_freeze"]
PRE = freeze["input_maps"]["pre_utc"]
POST = freeze["input_maps"]["post_utc"]
checks = freeze["selftest_matrix"]["c3-selftest-actual-pinned-archive"]["checks_passed"]
eol = "\r\n" if "\r\n" in card else "\n"
assert card.count("\n") == card.count(eol), "mixed line endings in the card"


def j(t):
    return json.dumps(t, ensure_ascii=True)


def append_to_string_line(text, prefix, addition):
    s = text.index(prefix)
    assert text.count(prefix) == 1, prefix
    e = text.index(eol, s)
    line = text[s:e]
    assert line.endswith('",'), line[-40:]
    return text[:s] + line[:-2] + j(addition)[1:-1] + '",' + text[e:]


def edit_row(text, ident, note):
    at = text.index('"id": "%s"' % ident)
    ev = '"evidence": "'
    e = text.index(ev, at)
    assert e - at < 400, ident
    return text[:e + len(ev)] + j(note)[1:-1] + " HISTORICAL TEXT OF THE ROW: " + text[e + len(ev):]


def edit_field(text, ident, field, value):
    at = text.index('"id": "%s"' % ident)
    key = '"%s": ' % field
    s = text.index(key, at)
    assert s - at < 4500, (ident, field)
    e = text.index(eol, s)
    trailing = "," if text[e - 1] == "," else ""
    return text[:s] + key + j(value) + trailing + text[e:]


ID = "ENV-CI-LUA-ARCHIVE-AVAILABILITY-1"
card = edit_row(card, ID, "CORRECTED in source " + CODE + " (UNREVIEWED; the finding stays OPEN until the SAME independent review AND an actual green CI of the windows-cross job on the corrected workflow; Root controls the rerun): the Lua step now uses tools/fetch-pinned-archive.sh with the unchanged pinned SHA-256 (lua.org first with bounded connect/overall times, then the verified MacPorts URL for the same bytes; only the exact hash is ever moved to the destination or extracted), plus one no-network selftest step (" + str(checks) + " checks over the actual pinned archive, a mutation control and a loopback real-curl control). No version, SHA-256, compiler flag, link flag, matrix, Dockerfile, Cargo or source change; no input row changed against the frozen " + NATIVE[:9] + " native source. Pack " + PK + ".")
card = edit_field(card, ID, "next_fix", "Corrected in " + CODE[:9] + " (UNREVIEWED); closes only after the SAME independent review and an actual green CI (Root controls the rerun and publication services). All ten D3 requirements, M5/M6 witnesses and sector-height-host-move-open-facts stay required.")

card = append_to_string_line(card, '  "phase": ', " CI RELIABILITY REPAIR (ENV-CI-LUA-ARCHIVE-AVAILABILITY-1, OPEN): the exact " + NATIVE[:9] + " CI windows-cross job failed twice at the lua.org download (curl 28); the verified bounded fallback to the same pinned archive is in " + CODE[:9] + " (UNREVIEWED); the frozen native " + NATIVE[:9] + " jobs and data are not relabelled.")
card = append_to_string_line(card, '  "next_action": ', " ALSO (CI, not physical): ONE SAME independent review of the Lua pinned-download repair " + CODE[:9] + " (pack " + PK + "), then Root's actual CI rerun of the exact published head; ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 stays OPEN until both. It affects the workflow, one helper and prose only and does not change the native " + NATIVE[:9] + " source or its frozen jobs.")

item = ("CI LUA PINNED-DOWNLOAD REPAIR (UNREVIEWED; ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 stays OPEN until the SAME independent review and an actual green CI): code " + CODE + " over checkpoint 04bd7e71b (finding registered OPEN there; native source " + NATIVE + " untouched). "
        "Defect (actual): exact CI run 42359965325, windows-cross jobs 128366083321 and the failed-only retry 128369471075 both failed in 'Lua 5.1.5 as lua.dll' before any compile or test (curl 28, www.lua.org:443, 135185 ms and 136790 ms); every other job passed. The pinned archive identity is unchanged (SHA-256 2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333); Root verified the same 221213 bytes from https://distfiles.macports.org/lua/lua-5.1.5.tar.gz over HTTPS. "
        "Change: eech-campaign/tools/fetch-pinned-archive.sh (URLs in order; connect 15 s, overall 120 s, no retry, https only, no certificate bypass; mktemp file removed on every exit; only the pinned hash is moved to the destination; a valid existing destination is reused, an invalid one never used) used by windows-cross, and tools/fetch-pinned-archive-selftest.sh (no network; fake curl; positive, primary failure to fallback, bad/truncated/empty bytes never used, all sources failing, existing valid/invalid destination, usage errors) run as one step of evidence-selftest. "
        "Checks between the PRE (" + PRE + ") and POST (" + POST + ") maps (equal; the rows are identical to the e326 final POST map): the selftest over a generated payload and over the actual pinned archive in sh and bash --posix (" + str(checks) + " checks each), a mutation control that disables the hash comparison and must fail, a real-curl option check against a refused loopback port, the workflow structural check against 1b89, syntax, git diff --check; all exit 0. The 0be and e326 matrices and Root's native 1b89 evidence are reused only through the explicit join. "
        "Limits: fake transport and the real archive prove the decisions, not the runner's reachability; no YAML parser locally; tools/Dockerfile unchanged. Pack " + PK + "; external freeze " + FREEZE + ". Guards: production-cutover 0, fix-blocker " + ID + " 0, deliver unmet. Preparation count stays 2. No self-review, approval, merge or Task49/D3/native authority claim.")
marker = eol + "  ]," + eol + '  "blockers": ['
assert card.count(marker) == 1
card = card.replace(marker, "," + eol + "    " + j(item) + marker)
json.loads(card)

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT \u2014 D3 Task49: sector-height SOURCE and proof guards REVIEWED"), head
new = (
    "# CURRENT \u2014 D3 Task49: CI reliability repair of the windows-cross Lua download in `" + CODE[:9] + "` (UNREVIEWED); native `" + NATIVE[:9] + "` jobs and data untouched (7 November 2026)\n\n"
    "The exact `" + NATIVE[:9] + "` CI (run `42359965325`) failed its `windows-cross` job twice, the failed-only retry included, in the step that downloads Lua 5.1.5 from `www.lua.org` (curl exit 28 after 135185 ms and 136790 ms), before any compile or test; every other job passed. "
    "The archive identity is not in doubt: the workflow's SHA-256 pin is unchanged and Root verified the same 221213 bytes from the MacPorts project distfiles. Registered `ENV-CI-LUA-ARCHIVE-AVAILABILITY-1` OPEN at `04bd7e71b` (`blocks_cutover=false`: the frozen native evidence is cleared, delivery still needs a green CI). "
    "Corrected here: the step uses the new `tools/fetch-pinned-archive.sh` (lua.org first with bounded connect and overall times, then the MacPorts URL; only bytes with the exact pinned SHA-256 are moved to the destination and extracted; nothing else is ever used) plus one no-network selftest step. "
    "Focused controls drive the actual helper through a fake transport over the generated payload and the actual pinned archive (" + str(checks) + " checks each, in `sh` and `bash --posix`), a mutation control that must fail, and a real-curl option check on a refused loopback port; the input map rows are identical to the e326 source, with no compiled, tested or linked path changed against `" + NATIVE[:9] + "`. "
    "Packet [" + PK + "](eech-campaign/" + PK + "/README.md); external freeze `" + FREEZE + "`.\n\n"
    "**Still OPEN:** the CI finding until the SAME independent review and an actual green CI (Root controls the rerun); the native height witnesses (healthy eligible control first, then the matched modes), `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1`, `sector-height-host-move-open-facts`, all ten D3 requirements, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census. "
    "No self-approval, merge or Task49/D3/native authority claim; `D3-CURRENT.json` is Root's and was not edited. Preparation count stays 2. No D4/D5/TypeScript.\n\n"
)
demoted = "# HISTORICAL (superseded by the CI Lua repair " + CODE[:9] + ") \u2014 " + head[len("# CURRENT \u2014 "):]
status = new + demoted + status[first_end:]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
