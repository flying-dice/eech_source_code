"""Metadata-only first checkpoint: register the CI finding ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 OPEN in the card (textual insertion). Usage: patch.py <repo>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
card = card_path.read_bytes().decode("utf-8")
row = {
    "id": "ENV-CI-LUA-ARCHIVE-AVAILABILITY-1", "status": "open", "blocks_cutover": False,
    "evidence": "Exact CI of 1b89c0b0ab613d2f6a08ea02dd6612457efa8982 (campaign run 42359965325): the windows-cross job 128366083321 failed in the step 'Lua 5.1.5 as lua.dll', BEFORE any compile or test, with curl exit 28 (connect to www.lua.org:443 timed out after 135185 ms); Root's single failed-only retry (windows-cross job 128369471075) failed at the SAME download after 136790 ms while every other job passed. Safe summaries preserved: R:/d3-evidence/sector-height-publication-1b89-CI-initial-download-failure-20261107.json (SHA-256 816bcd919cab4f57b3df11c941135a02937bbd2470e3e86ce0d78d84204abba7) and ...-CI-retry-download-failure-20261107.json (SHA-256 52b2049c32d7129861768cca75bfe4650b4b1ec90ee209ba7d63d20ea5bb197b). The failure is the single origin's availability, not the archive identity: the workflow (and tools/Dockerfile) pin lua-5.1.5.tar.gz at SHA-256 2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333. Root downloaded the SAME bytes (221213) over HTTPS from https://distfiles.macports.org/lua/lua-5.1.5.tar.gz and verified that exact hash (R:/d3-evidence/lua-5.1.5-MacPorts-verification-20261107.json, SHA-256 be4adb3d7d4c8b051a605a130f87e98f3ddc99de300d8be48c8754f4e537cedf). This is a CI reliability finding only: frozen native jobs of 1b89 are unaffected and not relabelled; blocks_cutover=false, delivery still requires the actual CI green.",
    "missing_fact": "A bounded verified download of the unchanged pinned archive (official URL first with bounded connect/overall times, then the verified MacPorts fallback; the exact SHA-256 verified before any extract/compile; failed, corrupt or truncated bytes never used), exercised by focused controls against the actual helper, the SAME independent review, and an actual CI green of the windows-cross job on the corrected workflow (Root controls the rerun).",
    "next_fix": "Being corrected in the sole lane (workflow + one small repo shell helper + focused selftest; no version, SHA, compiler flag, link flag, Dockerfile, Cargo or source change); stays OPEN until reviewed AND the actual CI is green. All ten D3 requirements, M5/M6 witnesses and sector-height-host-move-open-facts stay required.",
}
eol = "\r\n" if "\r\n" in card else "\n"  # the lead publication left the card CRLF: keep its line endings, never normalise
assert card.count("\n") == card.count(eol), "mixed line endings"
marker = eol + '  ],' + eol + '  "next_action":'
assert card.count(marker) == 1
block = "," + eol + "    " + json.dumps(row, indent=2, ensure_ascii=True).replace("\n", eol + "    ")
card = card.replace(marker, block + marker)
json.loads(card)
card_path.write_bytes(card.encode("utf-8"))
print("ok")
