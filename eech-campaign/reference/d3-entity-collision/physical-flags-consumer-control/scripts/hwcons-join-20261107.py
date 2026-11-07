"""Explicit actual-input join of the consumer batch to the reference final POST map (the physical-flags characterisation batch): which map rows changed or were added, and the verification, from the Git diff and the file text, that EVERY compiled-input change is test-only and
inert without its feature: the new translation unit csrc/eech_physical_flags_consumer_check.c lies wholly inside ONE `#ifdef EECH_PHYSICAL_FLAGS_CONSUMER_CHECK` ... `#endif` pair; csrc/eech_physical_flags_check.c changed in COMMENTS ONLY (its code with comments removed is byte-equal to the committed one);
the engine-sys manifest gains only the feature and its [[test]] entry (added lines only); build/main.rs gains only the one define block (added lines only). No original-campaign C source (aphavoc/, modules/) and no other compiled input changed. Usage: join.py <repo> <reference-post-map> <new-post-map> <out.json>"""
import hashlib
import json
import re
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
GATE = "EECH_PHYSICAL_FLAGS_CONSUMER_CHECK"
SYS = "eech-campaign/crates/eech-engine-sys/"
old = json.load(open(old_path, encoding="utf-8"))["files"]
new = json.load(open(new_path, encoding="utf-8"))["files"]
FIRST = SYS + "csrc/eech_physical_flags_check.c"
CONSUMER = SYS + "csrc/eech_physical_flags_consumer_check.c"
ALLOWED_CHANGED = {
    SYS + "Cargo.toml": "manifest: ONLY added lines (the consumer feature and its [[test]] entry)",
    SYS + "build/main.rs": "build script: ONLY the added block that defines the macro for the consumer feature",
    FIRST: "the sealed first control: COMMENTS ONLY (the high-index persistence wording); its code is byte-equal",
    SYS + "tests/physical_flags_guard.rs": "the source census gained one test (a test source, not a compiled input)",
}
ALLOWED_ADDED = {CONSUMER: "new translation unit wholly inside the consumer feature gate", SYS + "tests/physical_flags_consumer.rs": "the consumer control's Rust runner (needs the feature)"}
COMPILED = (
    "aphavoc/source/", "modules/", SYS + "csrc/", SYS + "build/", SYS + "src/", SYS + "Cargo.toml", SYS + "compat/",
    "eech-campaign/crates/eech-engine/src/", "eech-campaign/crates/eech-engine/Cargo.toml", "eech-campaign/crates/eech-dc/src/", "eech-campaign/crates/eech-dc/Cargo.toml",
    "eech-campaign/crates/eech-world/src/", "eech-campaign/crates/eech-world/Cargo.toml", "eech-campaign/Cargo.toml", "eech-campaign/Cargo.lock",
)
problems = []


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout


REFERENCE = git("rev-parse", "HEAD").decode().strip()
changed = sorted(k for k in new if k in old and new[k]["raw_sha256"] != old[k]["raw_sha256"])
added = sorted(k for k in new if k not in old)
removed = sorted(k for k in old if k not in new)
for k in changed:
    if k not in ALLOWED_CHANGED:
        problems.append("a map row changed outside the allowed set: " + k)
for k in added:
    if k not in ALLOWED_ADDED:
        problems.append("a map row was added outside the allowed set: " + k)
if removed:
    problems.append("map rows removed: %s" % removed)
compiled_changed = sorted(k for k in changed if k.startswith(COMPILED))
compiled_added = sorted(k for k in added if k.startswith(COMPILED))
rows = []


def added_only(path):
    diff = [l for l in git("diff", "-U0", REFERENCE, "--", path).decode("utf-8", "replace").splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
    return diff, all(l.startswith("+") for l in diff) and len(diff) > 0


for path, checker in (
    (SYS + "Cargo.toml", lambda diff: any("physical-flags-consumer-check" in l for l in diff) and any('name = "physical_flags_consumer"' in l for l in diff) and not any("dependencies" in l or l.startswith("+default") for l in diff)),
    (SYS + "build/main.rs", lambda diff: any("CARGO_FEATURE_PHYSICAL_FLAGS_CONSUMER_CHECK" in l for l in diff) and any(GATE in l for l in diff) and len(diff) == 3),
):
    diff, only_added = added_only(path)
    ok = only_added and checker(diff)
    rows.append({"path": path, "class": ALLOWED_CHANGED[path], "added_lines": len(diff), "verified": bool(ok)})
    if not ok:
        problems.append(path + ": the diff is not the expected added lines only")


def code_only(text):
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    return re.sub(r"\s+", " ", text).strip()


head_first = git("show", REFERENCE + ":" + FIRST).decode("latin-1")
now_first = open(repo + "/" + FIRST, "rb").read().decode("latin-1")
comments_only = code_only(head_first) == code_only(now_first) and head_first != now_first
rows.append({"path": FIRST, "class": ALLOWED_CHANGED[FIRST], "code_equal_comments_differ": comments_only, "verified": comments_only})
if not comments_only:
    problems.append(FIRST + ": the change is not comments only")


def outer_gate_only(text):
    lines = [l.strip() for l in re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", text, flags=re.S)).splitlines() if l.strip()]
    if not lines or lines[0] != "#ifdef " + GATE or lines[-1] != "#endif":
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


gate_ok = outer_gate_only(open(repo + "/" + CONSUMER, "rb").read().decode("latin-1"))
rows.append({"path": CONSUMER, "class": ALLOWED_ADDED[CONSUMER], "verified": gate_ok})
if not gate_ok:
    problems.append("the new translation unit is not wholly inside " + GATE)
expected_changed = sorted([SYS + "Cargo.toml", SYS + "build/main.rs", FIRST])
if compiled_changed != expected_changed or compiled_added != [CONSUMER]:
    problems.append("the compiled-input rows that changed or were added are not exactly the verified ones: %s %s" % (compiled_changed, compiled_added))
unchanged_compiled = sorted(k for k in new if k.startswith(COMPILED) and k in old and k not in changed)
agg_old = hashlib.sha256("".join("%s %s\n" % (k, old[k]["raw_sha256"]) for k in unchanged_compiled).encode()).hexdigest()
agg_new = hashlib.sha256("".join("%s %s\n" % (k, new[k]["raw_sha256"]) for k in unchanged_compiled).encode()).hexdigest()
if agg_old != agg_new:
    problems.append("the unchanged compiled rows differ")
since = sorted(set(git("diff", "--name-only", REFERENCE).decode().splitlines()) | set(git("ls-files", "--others", "--exclude-standard").decode().splitlines()))
ALLOWED_GIT = set(ALLOWED_CHANGED) | set(ALLOWED_ADDED) | {"eech-campaign/docs/d3-physical-flags-contract.md", "STATUS.md", "docs/handover/ACTIVE-EXTRACTION.json"}
for p in since:
    if p in ALLOWED_GIT or p.startswith("eech-campaign/reference/d3-entity-collision/"):
        continue
    problems.append("changed since the reference outside the allowed set: " + p)
result = {
    "kind": "explicit test-only join of the consumer batch against the reference POST map",
    "reference_commit": REFERENCE, "map_rows_changed": changed, "map_rows_added": added, "map_rows_removed": removed, "compiled_rows_changed": compiled_changed, "compiled_rows_added": compiled_added,
    "unchanged_compiled_rows": len(unchanged_compiled), "unchanged_compiled_aggregate_sha256": agg_new, "checks": rows, "changed_in_git_since_reference": since,
    "conclusion": "every compiled-input change is test-only and inert without its feature (one new translation unit wholly inside the consumer gate, comments-only edits of the sealed first control, added manifest lines, one added define block); no original-campaign C source and no other compiled input changed; the ordinary, original and save shipping builds are unchanged in effect. Nothing is claimed natively." if not problems else "NOT AS EXPECTED: see problems",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": changed, "added": added, "compiled_changed": compiled_changed, "compiled_added": compiled_added, "unchanged_compiled": len(unchanged_compiled), "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
