"""Explicit actual-input join of the KEY-FIX batch (the two source findings of the consolidated review V2 of the production candidate 0d0c2545b: the flag key domain and the issued group key's uniqueness, with the collision consumer control) to the reference final
POST map (the production batch, whose compiled inputs equal the committed HEAD's): which map rows changed or were added, and, from the Git diff and the file text, what each change IS:
  - the campaign C that carries the corrections (mb_hostentity.h, mb_hostmodel.c): the diff against the reference removes ONLY the lines of the key validator, of the unconditional cache write and of the two comment lines the corrections replace; mb_int.c, up_update.c and up_update.h
    are UNCHANGED rows (the production move itself is not touched, so the original profile's code is still the reference's);
  - the test-only C controls and the new collision control: each wholly inside its feature gate; the hosted header is an unchanged row;
  - the Rust host bound, bridge tests, public driver and test sources, listed and bound to the declared set; the manifest, build script and lock lines.
No other compiled or tested input changed: the unchanged rows are aggregated and compared. Usage: join.py <repo> <reference-post-map> <new-post-map> <out.json>"""
import hashlib
import json
import re
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
old = json.load(open(old_path, encoding="utf-8"))["files"]
new = json.load(open(new_path, encoding="utf-8"))["files"]
SYS = "eech-campaign/crates/eech-engine-sys/"
WORLD = "eech-campaign/crates/eech-world/"
ENGINE = "eech-campaign/crates/eech-engine/"
DC = "eech-campaign/crates/eech-dc/"
MOBILE = "aphavoc/source/entity/mobile/"
CAMPAIGN_C_UNCHANGED = [MOBILE + "mb_int.c", "aphavoc/source/entity/special/update/up_update.c", "aphavoc/source/entity/special/update/up_update.h", MOBILE + "mb_hostmodel.h", "aphavoc/source/entity/en_misc/en_maths/en_cllsn.c", MOBILE + "mb_hostoffer.h"]
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
COMPILED = (
    "aphavoc/source/", "modules/", SYS + "csrc/", SYS + "build/", SYS + "src/", SYS + "Cargo.toml", SYS + "compat/", ENGINE + "src/", ENGINE + "Cargo.toml", DC + "src/", DC + "Cargo.toml",
    WORLD + "src/", WORLD + "Cargo.toml", "eech-campaign/Cargo.toml", "eech-campaign/Cargo.lock",
)
problems = []


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout


REFERENCE = git("rev-parse", "HEAD").decode().strip()
changed = sorted(k for k in new if k in old and new[k]["raw_sha256"] != old[k]["raw_sha256"])
added = sorted(k for k in new if k not in old)
removed = sorted(k for k in old if k not in new)
if changed != EXPECTED_CHANGED:
    problems.append("the changed map rows are not the declared set: only now %s, only declared %s" % (sorted(set(changed) - set(EXPECTED_CHANGED)), sorted(set(EXPECTED_CHANGED) - set(changed))))
if added != EXPECTED_ADDED:
    problems.append("the added map rows are not the declared set: only now %s, only declared %s" % (sorted(set(added) - set(EXPECTED_ADDED)), sorted(set(EXPECTED_ADDED) - set(added))))
if removed:
    problems.append("map rows removed: %s" % removed)
rows = []


def code_only(text):
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    return re.sub(r"\s+", " ", text).strip()


def view(text, hosted):
    """the text as the profile compiles it: the branches of EECH_CAMPAIGN_HOST_ENTITY chosen, every other directive kept"""
    stack = []
    result = []
    for line in text.split("\n"):
        directive = line.strip()
        active = all(entry[1] for entry in stack if entry[0])
        if re.match(r"#\s*ifdef\s+EECH_CAMPAIGN_HOST_ENTITY\b", directive):
            stack.append([True, hosted])
        elif re.match(r"#\s*ifndef\s+EECH_CAMPAIGN_HOST_ENTITY\b", directive):
            stack.append([True, not hosted])
        elif re.match(r"#\s*if", directive):
            stack.append([False, True])
            if active:
                result.append(line)
        elif re.match(r"#\s*else\b", directive):
            if stack and stack[-1][0]:
                stack[-1][1] = not stack[-1][1]
            elif active:
                result.append(line)
        elif re.match(r"#\s*endif\b", directive):
            popped = stack.pop() if stack else [False, True]
            if not popped[0] and all(entry[1] for entry in stack if entry[0]):
                result.append(line)
        elif active:
            result.append(line)
    return "\n".join(result)


def head_text(path):
    return git("show", REFERENCE + ":" + path).decode("latin-1")


def now_text(path):
    return open(repo + "/" + path, "rb").read().decode("latin-1")


for path in CAMPAIGN_C_UNCHANGED:
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


def outer_gate_only(text, gate):
    lines = [l.strip() for l in re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", text, flags=re.S)).splitlines() if l.strip()]
    if not lines or lines[0] != "#ifdef " + gate or lines[-1] != "#endif":
        return False
    depth = 0
    for i, l in enumerate(lines):
        if re.match(r"#\s*(if|ifdef|ifndef)\b", l):
            depth += 1
        elif re.match(r"#\s*endif\b", l):
            depth -= 1
            if depth == 0 and i != len(lines) - 1:
                return False
    return depth == 0


for path, gate in GATED_C.items():
    ok = outer_gate_only(now_text(path), gate)
    rows.append({"path": path, "class": "test-only C control wholly inside its feature gate " + gate, "verified": ok})
    if not ok:
        problems.append(path + ": not wholly inside " + gate)
header_unchanged = HOSTED_HEADER not in changed and HOSTED_HEADER not in added and HOSTED_HEADER in new
rows.append({"path": HOSTED_HEADER, "class": "test-only hosted header: an unchanged row", "verified": header_unchanged})
if not header_unchanged:
    problems.append(HOSTED_HEADER + ": changed")
production_rust = [WORLD + "src/flags.rs", ENGINE + "src/lib.rs", DC + "src/module.rs", WORLD + "lua/entity-authority.lua"]
rows.append({"class": "Rust host key bound, bridge-side tests, and the public Lua driver's key checks (compiled or loaded inputs)", "paths": production_rust, "verified": all(p in changed or p in added for p in production_rust)})
unchanged_compiled = sorted(k for k in new if k.startswith(COMPILED) and k in old and k not in changed)
agg_old = hashlib.sha256("".join("%s %s\n" % (k, old[k]["raw_sha256"]) for k in unchanged_compiled).encode()).hexdigest()
agg_new = hashlib.sha256("".join("%s %s\n" % (k, new[k]["raw_sha256"]) for k in unchanged_compiled).encode()).hexdigest()
if agg_old != agg_new:
    problems.append("the unchanged compiled rows differ")
since = sorted(set(git("diff", "--name-only", REFERENCE).decode().splitlines()) | set(git("ls-files", "--others", "--exclude-standard").decode().splitlines()))
ALLOWED_GIT = set(EXPECTED_CHANGED) | set(EXPECTED_ADDED) | {"eech-campaign/docs/d3-physical-flags-contract.md", "STATUS.md", "docs/handover/ACTIVE-EXTRACTION.json"}
for p in since:
    if p in ALLOWED_GIT or p.startswith("eech-campaign/reference/d3-entity-collision/"):
        continue
    problems.append("changed since the reference outside the declared set: " + p)
result = {
    "kind": "explicit join of the KEY-FIX batch against the reference POST map",
    "reference_commit": REFERENCE, "map_rows_changed": changed, "map_rows_added": added, "map_rows_removed": removed,
    "compiled_rows_changed": sorted(k for k in changed if k.startswith(COMPILED)), "compiled_rows_added": sorted(k for k in added if k.startswith(COMPILED)),
    "unchanged_compiled_rows": len(unchanged_compiled), "unchanged_compiled_aggregate_sha256": agg_new, "checks": rows, "changed_in_git_since_reference": since,
    "conclusion": "the key-fix batch changes the campaign C in two files only (the key validator and the staged group binding; the production move's files are unchanged rows), every control source is inside its feature gate, and the Rust host bound, bridge tests, public driver and test sources are the declared set; no other compiled or tested input changed",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": len(changed), "added": len(added), "unchanged_compiled": len(unchanged_compiled), "checks": [(r.get("path", r.get("class"))[:60], r["verified"]) for r in rows], "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
