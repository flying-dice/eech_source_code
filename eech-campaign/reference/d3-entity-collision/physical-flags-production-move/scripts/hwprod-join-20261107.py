"""Explicit actual-input join of the PRODUCTION batch (the complete physical UPDATED / MOVED / ROTATED store moves into its host; Task49 candidate) to the reference final POST map (the consumer batch, whose compiled inputs equal the committed HEAD's): which map rows changed or
were added, and, from the Git diff and the file text, what each change IS:
  - the six campaign C files that carry the production move (mb_int.c, up_update.c, up_update.h, mb_hostentity.h, mb_hostmodel.c, mb_hostmodel.h): the ORIGINAL profile's view of the first three (every `#ifdef EECH_CAMPAIGN_HOST_ENTITY` branch dropped, its `#else` kept, every
    `#ifndef EECH_CAMPAIGN_HOST_ENTITY` branch kept) has the SAME code as the reference (comments and whitespace aside), so the original solver profile compiles the same lines; the other three only ADD lines (the enum members, the dialogue, the binding), except the two enum-terminator lines;
  - the test-only C controls and the new header: each wholly inside its feature gate (or an include guard), inert without its feature;
  - the Rust host, wire and bridge and the test sources, listed and bound to the declared set; the manifest and lock lines.
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
CAMPAIGN_C_ORIGINAL_EQUAL = [MOBILE + "mb_int.c", "aphavoc/source/entity/special/update/up_update.c", "aphavoc/source/entity/special/update/up_update.h"]
CAMPAIGN_C_ADDITIONS = [MOBILE + "mb_hostentity.h", MOBILE + "mb_hostmodel.c", MOBILE + "mb_hostmodel.h"]
GATED_C = {
    SYS + "csrc/eech_physical_flags_check.c": "EECH_PHYSICAL_FLAGS_CHECK",
    SYS + "csrc/eech_physical_flags_consumer_check.c": "EECH_PHYSICAL_FLAGS_CONSUMER_CHECK",
    SYS + "csrc/eech_entity_dialogue_check.c": "EECH_ENTITY_DIALOGUE_CHECK",
}
HOSTED_HEADER = SYS + "csrc/eech_physical_flags_hosted.h"
EXPECTED_CHANGED = sorted(
    CAMPAIGN_C_ORIGINAL_EQUAL + CAMPAIGN_C_ADDITIONS + list(GATED_C)
    + [
        "eech-campaign/Cargo.lock", DC + "Cargo.toml", DC + "src/module.rs", SYS + "Cargo.toml", SYS + "src/lib.rs", ENGINE + "src/lib.rs",
        WORLD + "lua/entity-authority.lua", WORLD + "src/lib.rs", WORLD + "src/model_host.rs", WORLD + "src/roster.rs",
        SYS + "tests/entity_dialogue.rs", SYS + "tests/physical_flags.rs", SYS + "tests/physical_flags_consumer.rs", SYS + "tests/physical_flags_guard.rs", WORLD + "tests/entity_authority.rs",
    ]
)
EXPECTED_ADDED = sorted([HOSTED_HEADER, WORLD + "src/flags.rs", WORLD + "tests/physical_flags_host.rs", SYS + "tests/physical_flags_hosted_faults.rs", SYS + "tests/support/flags_host.rs"])
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


for path in CAMPAIGN_C_ORIGINAL_EQUAL:
    before, after = head_text(path), now_text(path)
    equal = code_only(view(after, False)) == code_only(before)
    hosted_differs = code_only(view(after, True)) != code_only(before)
    rows.append({"path": path, "class": "campaign C: the ORIGINAL profile's view has the reference's code", "original_profile_code_equal_to_reference": equal, "hosted_profile_differs": hosted_differs, "verified": equal and hosted_differs})
    if not (equal and hosted_differs):
        problems.append(path + ": the original profile's view is not the reference's code, or the hosted view does not differ")
for path in CAMPAIGN_C_ADDITIONS:
    diff = [l for l in git("diff", "-U0", REFERENCE, "--", path).decode("utf-8", "replace").splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
    removed_lines = [l[1:].strip() for l in diff if l.startswith("-")]
    allowed_removed = {"CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE", "CAMPAIGN_MODEL_SCAN_RESTORED"}
    ok = all(l in allowed_removed for l in removed_lines) and any(l.startswith("+") for l in diff)
    rows.append({"path": path, "class": "campaign C: additions only (the dialogue, the binding, the enum members)", "added_lines": sum(1 for l in diff if l.startswith("+")), "removed_lines": removed_lines, "verified": ok})
    if not ok:
        problems.append(path + ": the diff removes a line other than the two enum terminators")


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
header_lines = [l.strip() for l in re.sub(r"/\*.*?\*/", "", now_text(HOSTED_HEADER), flags=re.S).splitlines() if l.strip()]
header_ok = header_lines[0] == "#ifndef EECH_PHYSICAL_FLAGS_HOSTED_H" and header_lines[1] == "#define EECH_PHYSICAL_FLAGS_HOSTED_H" and header_lines[2] == "#ifdef EECH_CAMPAIGN_HOST_ENTITY" and header_lines[-1] == "#endif" and header_lines[-2] == "#endif"
includers = [p for p in new if p.endswith((".c", ".h")) and "eech_physical_flags_hosted.h" in now_text(p) and p != HOSTED_HEADER]
header_ok = header_ok and sorted(includers) == sorted([SYS + "csrc/eech_physical_flags_check.c", SYS + "csrc/eech_physical_flags_consumer_check.c"])
rows.append({"path": HOSTED_HEADER, "class": "test-only header: include guard, wholly inside EECH_CAMPAIGN_HOST_ENTITY, included only by the two flag controls", "includers": sorted(includers), "verified": header_ok})
if not header_ok:
    problems.append(HOSTED_HEADER + ": not a guarded hosted-only header included only by the two controls")
production_rust = [WORLD + "src/lib.rs", WORLD + "src/model_host.rs", WORLD + "src/roster.rs", WORLD + "src/flags.rs", ENGINE + "src/lib.rs", SYS + "src/lib.rs", DC + "src/module.rs", WORLD + "lua/entity-authority.lua"]
rows.append({"class": "Rust host store, provider, wire constants, bridge validator and the public Lua driver's pass-through (compiled or loaded inputs)", "paths": production_rust, "verified": all(p in changed or p in added for p in production_rust)})
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
    "kind": "explicit join of the PRODUCTION batch against the reference POST map",
    "reference_commit": REFERENCE, "map_rows_changed": changed, "map_rows_added": added, "map_rows_removed": removed,
    "compiled_rows_changed": sorted(k for k in changed if k.startswith(COMPILED)), "compiled_rows_added": sorted(k for k in added if k.startswith(COMPILED)),
    "unchanged_compiled_rows": len(unchanged_compiled), "unchanged_compiled_aggregate_sha256": agg_new, "checks": rows, "changed_in_git_since_reference": since,
    "conclusion": "the production move changes the campaign C only in six files; the ORIGINAL profile's view of the three files that carry the move has the reference's code (the hosted branches are the only difference), the other three files only add lines (two enum terminators aside); every control source is inside its feature gate; the Rust host, wire, bridge and driver files are the declared set; no other compiled or tested input changed",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": len(changed), "added": len(added), "unchanged_compiled": len(unchanged_compiled), "checks": [(r.get("path", r.get("class"))[:60], r["verified"]) for r in rows], "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
