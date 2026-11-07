"""Derives hwkey-join-20261107.py from hwprod-join-20261107.py with the declared sets of the key-fix batch (kept as a script so the derivation is itself retained and its asserts are explicit)."""
import pathlib
import re

src = pathlib.Path("R:/d3-evidence/hwprod-join-20261107.py").read_text(encoding="utf-8")


def rep(old, new):
    global src
    assert src.count(old) == 1, (old[:70], src.count(old))
    src = src.replace(old, new)


def cut(start, end, new):
    global src
    i = src.index(start)
    j = src.index(end, i)
    src = src[:i] + new + src[j:]


cut('"""Explicit actual-input join', 'import hashlib', '''"""Explicit actual-input join of the KEY-FIX batch (the two source findings of the consolidated review V2 of the production candidate 0d0c2545b: the flag key domain and the issued group key's uniqueness, with the collision consumer control) to the reference final
POST map (the production batch, whose compiled inputs equal the committed HEAD's): which map rows changed or were added, and, from the Git diff and the file text, what each change IS:
  - the campaign C that carries the corrections (mb_hostentity.h, mb_hostmodel.c): the diff against the reference removes ONLY the lines of the key validator, of the unconditional cache write and of the two comment lines the corrections replace; mb_int.c, up_update.c and up_update.h
    are UNCHANGED rows (the production move itself is not touched, so the original profile's code is still the reference's);
  - the test-only C controls and the new collision control: each wholly inside its feature gate; the hosted header is an unchanged row;
  - the Rust host bound, bridge tests, public driver and test sources, listed and bound to the declared set; the manifest, build script and lock lines.
No other compiled or tested input changed: the unchanged rows are aggregated and compared. Usage: join.py <repo> <reference-post-map> <new-post-map> <out.json>"""
''')
cut('CAMPAIGN_C_ORIGINAL_EQUAL = [', 'COMPILED = (', '''CAMPAIGN_C_UNCHANGED = [MOBILE + "mb_int.c", "aphavoc/source/entity/special/update/up_update.c", "aphavoc/source/entity/special/update/up_update.h", MOBILE + "mb_hostmodel.h", "aphavoc/source/entity/en_misc/en_maths/en_cllsn.c", MOBILE + "mb_hostoffer.h"]
CAMPAIGN_C_CORRECTED = [MOBILE + "mb_hostentity.h", MOBILE + "mb_hostmodel.c"]
# the exact lines (by prefix) the corrections remove from the two files
ALLOWED_REMOVED_PREFIXES = (
    "if (key [0] != 'f')",
    "for (index = 1; key [index] != 0; index++)",
    "if ((key [index] < '0') || (key [index] > '9') || (index > CAMPAIGN_ROSTER_KEY_MAX))",
    "return (index >= 2);",
    "//     as the host binding that issued its keys, whose own lifetime is the store's",
    "campaign_entity_flag_exchange (CAMPAIGN_SCAN_PHASE_FLAG_GROUP, 0, 0, NULL, ((unsigned int) group < CAMPAIGN_FLAG_RESET_GROUPS), flag_key_of_group [group], NULL);",
    "#if defined(EECH_PHYSICAL_FLAGS_CHECK) || defined(EECH_PHYSICAL_FLAGS_CONSUMER_CHECK)",
    "// TEST-ONLY (features physical-flags-check and physical-flags-consumer-check; never in the shipping build)",
)
HOSTED_HEADER = SYS + "csrc/eech_physical_flags_hosted.h"
GATED_C = {
    SYS + "csrc/eech_physical_flags_check.c": "EECH_PHYSICAL_FLAGS_CHECK",
    SYS + "csrc/eech_entity_dialogue_check.c": "EECH_ENTITY_DIALOGUE_CHECK",
    SYS + "csrc/eech_physical_flags_collision_check.c": "EECH_PHYSICAL_FLAGS_COLLISION_CHECK",
}
EXPECTED_CHANGED = sorted(
    CAMPAIGN_C_CORRECTED
    + [
        "eech-campaign/Cargo.lock", DC + "src/module.rs", SYS + "Cargo.toml", SYS + "build/main.rs", SYS + "csrc/eech_entity_dialogue_check.c", SYS + "csrc/eech_physical_flags_check.c",
        SYS + "tests/entity_dialogue.rs", SYS + "tests/physical_flags_guard.rs", SYS + "tests/support/flags_host.rs", ENGINE + "src/lib.rs", WORLD + "lua/entity-authority.lua",
        WORLD + "src/flags.rs", WORLD + "tests/entity_authority.rs", WORLD + "tests/physical_flags_host.rs",
    ]
)
EXPECTED_ADDED = sorted([SYS + "csrc/eech_physical_flags_collision_check.c", SYS + "tests/physical_flags_collision.rs", SYS + "tests/physical_flags_identity.rs", SYS + "tests/support/lua_provider.rs"])
''')
# the checks of the campaign C
cut('for path in CAMPAIGN_C_ORIGINAL_EQUAL:', 'def outer_gate_only', '''for path in CAMPAIGN_C_UNCHANGED:
    same = path not in changed and path not in added and path in new
    rows.append({"path": path, "class": "campaign C of the production move: an UNCHANGED row (the original profile's code is still the reference's)", "verified": same})
    if not same:
        problems.append(path + ": changed, although the key-fix batch does not touch it")
for path in CAMPAIGN_C_CORRECTED:
    diff = [l for l in git("diff", "-U0", REFERENCE, "--", path).decode("utf-8", "replace").splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
    removed_lines = [l[1:].strip() for l in diff if l.startswith("-")]
    ok = all(any(l.startswith(prefix) for prefix in ALLOWED_REMOVED_PREFIXES) for l in removed_lines) and any(l.startswith("+") for l in diff)
    rows.append({"path": path, "class": "campaign C: the key validator, the staged group binding and a test hook gate; only the declared lines are removed", "added_lines": sum(1 for l in diff if l.startswith("+")), "removed_lines": len(removed_lines), "verified": ok})
    if not ok:
        problems.append(path + ": the diff removes a line other than the declared ones: " + str([l for l in removed_lines if not any(l.startswith(p) for p in ALLOWED_REMOVED_PREFIXES)]))


''')
# the header block of the previous join (an unchanged row now) is replaced by the statement
cut('header_lines = [l.strip()', 'production_rust = [', '''header_unchanged = HOSTED_HEADER not in changed and HOSTED_HEADER not in added and HOSTED_HEADER in new
rows.append({"path": HOSTED_HEADER, "class": "test-only hosted header: an unchanged row", "verified": header_unchanged})
if not header_unchanged:
    problems.append(HOSTED_HEADER + ": changed")
''')
rep('HOSTED_HEADER = SYS + "csrc/eech_physical_flags_hosted.h"\n', 'HOSTED_HEADER = SYS + "csrc/eech_physical_flags_hosted.h"\n')
rep('"kind": "explicit join of the PRODUCTION batch against the reference POST map"', '"kind": "explicit join of the KEY-FIX batch against the reference POST map"')
cut('    "conclusion":', '    "problems": problems,', '''    "conclusion": "the key-fix batch changes the campaign C in two files only (the key validator and the staged group binding; the production move's files are unchanged rows), every control source is inside its feature gate, and the Rust host bound, bridge tests, public driver and test sources are the declared set; no other compiled or tested input changed",
''')
rep('production_rust = [WORLD + "src/lib.rs", WORLD + "src/model_host.rs", WORLD + "src/roster.rs", WORLD + "src/flags.rs", ENGINE + "src/lib.rs", SYS + "src/lib.rs", DC + "src/module.rs", WORLD + "lua/entity-authority.lua"]',
    'production_rust = [WORLD + "src/flags.rs", ENGINE + "src/lib.rs", DC + "src/module.rs", WORLD + "lua/entity-authority.lua"]')
rep('rows.append({"class": "Rust host store, provider, wire constants, bridge validator and the public Lua driver\'s pass-through (compiled or loaded inputs)"', 'rows.append({"class": "Rust host key bound, bridge-side tests, and the public Lua driver\'s key checks (compiled or loaded inputs)"')
pathlib.Path("R:/d3-evidence/hwkey-join-20261107.py").write_text(src, encoding="utf-8", newline="\n")
print("ok", len(src))
