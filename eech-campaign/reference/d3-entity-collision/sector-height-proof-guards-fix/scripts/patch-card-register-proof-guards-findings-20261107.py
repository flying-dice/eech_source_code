"""Metadata-only first checkpoint: register the SAME reviewer's two P2 findings OPEN in the card (textual insertion). Usage: patch.py <repo>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
card = card_path.read_bytes().decode("utf-8")
REVIEW = "409a2622b5314d82c02dc7710dcf3528f8390678e5b11162108847bcb83aa27b (independent-sector-height-proofplan-5aae-source-20261107.json; CHANGES_REQUIRED, native_launch_clearance=false, no new production defect) of correction source 5aae2256f02ca3d79e762d71c9b79c20dc34856b"
rows = [
    {"id": "ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1", "status": "open", "blocks_cutover": False,
     "evidence": "P2 of the SAME review " + REVIEW + ": sector-height-baseline.lua's scoped comparison accepts parsed identity/totals/sector maps without validating the accepted report's terminal completeness or the declared scan-row structure; duplicates silently overwrite, missing fields are not a schema failure, and hosted scan_whole checks only scan_out_of_range and a positive read total, so a report truncated before its verdict or a duplicated/omitted row can compare EQUAL_AT_SCOPE with scan_whole=1.",
     "missing_fact": "Both sides validated (actual terminal verdict of the accepted original, unique contiguous row indices and sector keys, declared sector count, exact scoped fields/domains, totals and word consistency) with positive-first single-fault controls; the hosted scope stays narrower (no original-link sho-complete, event, raise or session equality); the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (Lua diagnostic + driver tests); closes only by the SAME reviewer. ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 stays partially addressed and OPEN until the SAME reviewer closes all."},
    {"id": "ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1", "status": "open", "blocks_cutover": False,
     "evidence": "P2 of the SAME review " + REVIEW + ": an expected-marker fatal with height_altered=0 bypasses the other-marker failure branch and the clean UNREACHED branch; the generic alteration sum includes the configured forced-hit secondary input, so an unrelated fatal bearing the mode's expected marker, after a configured narrow-phase hit and before any eligible height intervention, can satisfy sector_height_withheld (and the other non-consumption height refusal modes).",
     "missing_fact": "Credit requires EXACTLY the one actual eligible height intervention, independent of all secondary alteration counters and of marker equality; any fatal before it is FAILURE (the planned generic marker included); only a clean run with no eligible read is UNREACHED; wrong_word keeps its same-entry fact-12 consumption and exact candidate refusal; positive-first controls per affected mode and an injected unrelated fatal that must fail; the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (Lua diagnostic + driver tests); closes only by the SAME reviewer. M5 (ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1), M6 (ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1), sector-height-host-move-open-facts and all ten D3 requirements stay required."},
]
marker = '\n  ],\n  "next_action":'
assert card.count(marker) == 1
block = ""
for row in rows:
    block += ",\n    " + json.dumps(row, indent=2, ensure_ascii=True).replace("\n", "\n    ")
card = card.replace(marker, block + marker)
json.loads(card)
card_path.write_bytes(card.encode("utf-8"))
print("ok")
