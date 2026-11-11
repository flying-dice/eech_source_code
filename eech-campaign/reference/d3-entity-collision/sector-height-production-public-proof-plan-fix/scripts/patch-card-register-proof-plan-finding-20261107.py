"""Metadata-only first checkpoint: register the SAME reviewer's P2 finding OPEN in the card (textual insertion, no JSON round trip). Usage: patch.py <repo>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
card = card_path.read_bytes().decode("utf-8")
row = {
    "id": "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1", "status": "open", "blocks_cutover": False,
    "evidence": "P2 of the SAME combined independent review 49d7739edb0855e5593bce91602eebcdedd8fb88c820677ee6695259ee9ce58c (independent-sector-height-production-0be-source-20261107.json; CHANGES_REQUIRED, native_launch_clearance=false, no new functional/math/protocol defect established) of production source 0be1dd8465d9bf38da69cd3b303206cf5a64574d: the promised public word-comparison and Host authority/refusal gate is not executable, the observer and restore control descriptions overstate their scope (SH10/SH11 still capture the Host-returned FIRST/fact12 word; the observer is not globally unusable and a whole original-LINK report comparison is invalid), the clearance helper reads retired raw storage, section 8 claims guarded fw execution was controlled, section 7 claims a restore-after-insertion refusal the provider deliberately does not make.",
    "missing_fact": "An executable scoped public baseline/comparison and Host authority/refusal witness through the existing bridge/provider paths (commands, features, phase, row schema, eligibility, complete checks, expected outcomes), the observer scope and clearance helper corrected, sections 7-10, the packet README and the card aligned with the actual semantics, and the SAME reviewer's changed-material review.",
    "next_fix": "Being corrected in the sole lane (test-only witness/plan, no production solver change); closes only by the SAME reviewer. M5 (ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1), M6 (ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1), sector-height-host-move-open-facts and all ten D3 requirements stay required.",
}
marker = '\n  ],\n  "next_action":'
assert card.count(marker) == 1
card = card.replace(marker, ",\n    " + json.dumps(row, indent=2, ensure_ascii=True).replace("\n", "\n    ") + marker)
json.loads(card)
card_path.write_bytes(card.encode("utf-8"))
print("ok")
