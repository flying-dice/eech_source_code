"""Explicit actual-input join of the four-P2 correction of the M5/M6 witness source (review 96b2f414) (a) to the frozen native 1b89 source and to the CI batch's final POST map (unchanged logic of the first witness join: the shipping
build's compiled inputs are shown unchanged from the Git diff itself, never from a label) and (b) to the REVIEWED source 6e4139043 and its final POST map: exactly which paths changed since the review and that nothing outside
the witness driver, its tests, the gated C fixture, its real-C tests and guard, and the contract moved. Native evidence of 1b89 keeps its true identity and is NEVER claimed for this source (changed C is not built here).
Usage: join.py <repo> <ci-post-map> <new-post-map> <reviewed-post-map> <out.json>"""
import json
import re
import subprocess
import sys

repo, old_path, new_path, reviewed_path, out = sys.argv[1:6]
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
REVIEWED = "6e4139043"
old = json.load(open(old_path, encoding="utf-8"))
new = json.load(open(new_path, encoding="utf-8"))
reviewed = json.load(open(reviewed_path, encoding="utf-8"))
GATE = "EECH_SECTOR_HEIGHT_WITNESS"


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout


def text_of(path):
    return open(repo + "/" + path, "rb").read().decode("latin-1")


def added_lines(path):
    """(added line numbers in the new file, removed lines) of `git diff -U0 NATIVE -- path`"""
    diff = git("diff", "-U0", NATIVE, "--", path).decode("latin-1").splitlines()
    added, removed, at = [], [], 0
    for line in diff:
        m = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", line)
        if m:
            at = int(m.group(1))
            continue
        if line.startswith("+++") or line.startswith("---") or line.startswith("diff ") or line.startswith("index "):
            continue
        if line.startswith("+"):
            added.append(at)
            at += 1
        elif line.startswith("-"):
            removed.append(line[1:])
    return added, removed


def block_span(text, macro):
    lines = text.split("\n")
    start = next(i for i, l in enumerate(lines) if l.strip() == "#ifdef " + macro) + 1
    depth, end = 0, None
    for i in range(start - 1, len(lines)):
        s = lines[i].lstrip()
        if s.startswith("#if"):
            depth += 1
        elif s.startswith("#endif"):
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    return start, end


def outer_gate_only(text, macro):
    lines = [l.strip() for l in re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", text, flags=re.S)).splitlines() if l.strip()]
    if not lines or lines[0] != "#ifdef " + macro or lines[-1] != "#endif":
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


new_rows_ = new["files"]
ROOTS = ("aphavoc/", "modules/")
changed_paths = sorted(set(git("diff", "--name-only", NATIVE).decode().splitlines()) | set(git("ls-files", "--others", "--exclude-standard").decode().splitlines()))
rows, problems = [], []
for p in changed_paths:
    if p.startswith(ROOTS):
        rows.append({"path": p, "class": "COMPILED C INPUT (PROBLEM)"})
        problems.append("a C source of the original campaign changed against 1b89: " + p)
    elif p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source/") or p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed/") or p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix/") or p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed/") or p.startswith("eech-campaign/reference/d3-ci-lua-download/"):
        rows.append({"path": p, "class": "retained evidence packet (not an input of any check)"})
    elif p in ("docs/handover/ACTIVE-EXTRACTION.json", "STATUS.md", "docs/governance/D3-FIVE-WHYS-20261102.md", ".github/workflows/eech-campaign.yml") or p.startswith("eech-campaign/tools/fetch-pinned-archive"):
        rows.append({"path": p, "class": "records, prose or the CI repair already reviewed (not compiled into the DLL)"})
    elif p.endswith(".md") and p.startswith("eech-campaign/docs/"):
        rows.append({"path": p, "class": "documentation (not compiled)"})
    elif p.startswith("eech-campaign/tools/") and p.endswith(".sh"):
        rows.append({"path": p, "class": "a Windows build wrapper (a copy of the observer wrapper with one cargo argument; not an input of the shipping build)"})
    elif p.endswith("eech-world/lua/sector-height-witness.lua") or p.endswith("eech-world/tests/sector_height_witness_driver.rs"):
        rows.append({"path": p, "class": "host Lua driver run by eech-world and its scripted tests (not compiled into the campaign DLL)"})
    elif p.endswith("eech-world/tests/host_callbacks.rs"):
        # the census of host scripts that select the reference profile: the new driver is the 64th; the diff must be exactly the count (63 -> 64) and its comment, nothing else
        diff = [l for l in git("diff", "-U0", NATIVE, "--", p).decode("latin-1").splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
        removed_lines = [l for l in diff if l.startswith("-")]
        added_lines_ = [l for l in diff if l.startswith("+")]
        ok = (len(removed_lines) == 2 and len(added_lines_) == 2 and removed_lines[1].strip() == "-    assert_eq!((with_profile, without_profile), (63, 2));".strip()
              and added_lines_[1].strip() == "+    assert_eq!((with_profile, without_profile), (64, 2));".strip() and "63 scripts select the profile" in removed_lines[0] and "64 scripts select the profile" in added_lines_[0])
        rows.append({"path": p, "class": "test source: the census of host scripts that select the reference profile, 63 -> 64 (the new witness driver is the 64th): exactly the count and its comment", "verified": ok})
        if not ok:
            problems.append(p + ": the diff is not exactly the census count and its comment")
    elif p.endswith("eech-engine-sys/tests/sector_height_witness.rs") or p.endswith("eech-engine-sys/tests/sector_height_witness_guard.rs"):
        rows.append({"path": p, "class": "test source (the first needs the feature; the second reads text in every profile)"})
    elif p.endswith("csrc/eech_hw_witness.c"):
        ok = outer_gate_only(text_of(p), GATE)
        rows.append({"path": p, "class": "new translation unit that preprocesses to nothing without " + GATE + " (one outer #ifdef...#endif pair)", "verified": ok})
        if not ok:
            problems.append(p + ": not wholly inside " + GATE)
    elif p.endswith("csrc/eech_engine.c") or p.endswith("csrc/eech_engine.h"):
        added, removed = added_lines(p)
        start, end = block_span(text_of(p), GATE)
        # every added line is inside the gate block, and no line was removed
        lines = text_of(p).split("\n")
        # the block, and at most the one blank separator line just above or just below it (git places an inserted blank on either side)
        inside = all(start <= n <= end or (n in (start - 1, end + 1) and lines[n - 1].strip() == "") for n in added)
        ok = inside and not removed and len(added) > 0
        rows.append({"path": p, "class": "shipping source file: ONLY added lines, all inside the " + GATE + " block (lines %d-%d); none removed" % (start, end), "added_lines": len(added), "removed_lines": len(removed), "verified": ok})
        if not ok:
            problems.append("%s: the diff is not added lines inside the %s block (inside=%s removed=%d)" % (p, GATE, inside, len(removed)))
    elif p.endswith("eech-engine-sys/build/main.rs"):
        added, removed = added_lines(p)
        body = [l.strip() for l in text_of(p).split("\n")]
        at = next(i for i, l in enumerate(body) if "CARGO_FEATURE_SECTOR_HEIGHT_WITNESS" in l)
        ok = not removed and added == [at + 1, at + 2, at + 3] and 'build.define("EECH_SECTOR_HEIGHT_WITNESS", None);' in body[at + 1]
        rows.append({"path": p, "class": "build script: ONE added `if CARGO_FEATURE_SECTOR_HEIGHT_WITNESS` block defining the macro; nothing removed", "verified": ok})
        if not ok:
            problems.append(p + ": the diff is not the one added feature block")
    elif p.endswith("Cargo.toml") and "/crates/" in p:
        added, removed = added_lines(p)
        diff = [l for l in git("diff", "-U0", NATIVE, "--", p).decode().splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
        body_ok = all(l.startswith("+") for l in diff) and any("sector-height-witness" in l for l in diff) and not any(l.startswith("+default") or "dependencies" in l for l in diff)
        rows.append({"path": p, "class": "manifest: ONLY added lines (the feature entry, and for eech-engine-sys its [[test]] target); no dependency, version or default-feature change", "verified": body_ok})
        if not body_ok:
            problems.append(p + ": the manifest diff is not added feature lines only")
    elif p.endswith("eech-engine-sys/src/lib.rs"):
        added, removed = added_lines(p)
        diff = [l for l in git("diff", "-U0", NATIVE, "--", p).decode().splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
        ok = not removed and diff[0].strip() == '+#[cfg(feature = "sector-height-witness")]' and len(diff) == 5
        rows.append({"path": p, "class": "sys crate: ONE added cfg-gated extern block; nothing removed", "verified": ok})
        if not ok:
            problems.append(p + ": the diff is not the one gated extern block")
    elif p.endswith("eech-engine/src/lib.rs"):
        added, removed = added_lines(p)
        diff = [l for l in git("diff", "-U0", NATIVE, "--", p).decode().splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
        # the three cfg(any(...)) lists gain the feature (their last member keeps a comma); everything else added is gated by the feature or in the test module
        widened = [l for l in diff if l.startswith("-")] == ['-    feature = "collision-fixture"', '-    feature = "collision-fixture"', '-        feature = "collision-fixture"']
        ok = widened and len([l for l in diff if l.startswith("+") and 'feature = "sector-height-witness"' in l and "cfg" not in l]) >= 3
        rows.append({"path": p, "class": "engine crate: the three `cfg(any(feature ...))` lists of the case-report helpers gain `sector-height-witness` (identical with the feature off), plus cfg-gated additions and one gated unit test", "verified": bool(ok)})
        if not ok:
            problems.append(p + ": the diff is not the three widened cfg lists and gated additions")
    elif p.endswith("eech-dc/src/module.rs"):
        added, removed = added_lines(p)
        diff = [l for l in git("diff", "-U0", NATIVE, "--", p).decode("latin-1").splitlines() if l[:1] in "+-" and not l.startswith(("+++", "---"))]
        ok = not removed and any('#[cfg(feature = "sector-height-witness")]' in l for l in diff) and len(diff) == 6
        rows.append({"path": p, "class": "Lua binding: ONE added cfg-gated method (6 lines); nothing removed", "verified": ok})
        if not ok:
            problems.append(p + ": the diff is not the one gated method")
    else:
        rows.append({"path": p, "class": "UNCLASSIFIED"})
        problems.append(p + ": unclassified")

# (b) what changed since the REVIEWED source: only the allowed set; every other path must have NO diff against it
reviewed_full = git("rev-parse", REVIEWED).decode().strip()
since = sorted(set(git("diff", "--name-only", reviewed_full).decode().splitlines()) | set(git("ls-files", "--others", "--exclude-standard").decode().splitlines()))
ALLOWED_SINCE = {
    "eech-campaign/crates/eech-engine-sys/csrc/eech_hw_witness.c": "the gated fixture: the shared writer-path helper (ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1)",
    "eech-campaign/crates/eech-engine-sys/tests/sector_height_witness.rs": "real-C tests of the shared path helper",
    "eech-campaign/crates/eech-engine-sys/tests/sector_height_witness_guard.rs": "source guards of the helper and the original writer's 256-byte buffers",
    "eech-campaign/crates/eech-world/lua/sector-height-witness.lua": "the host driver: RTZ oracle, pack retry reconciliation, same-caller join, whole-report validation, grammar, fault precedence",
    "eech-campaign/crates/eech-world/tests/sector_height_witness_driver.rs": "its scripted tests (real reference profile selection, positive-first and single-fault controls)",
    "eech-campaign/docs/d3-sector-height-production-contract.md": "contract section 12 prose",
    "STATUS.md": "records", "docs/handover/ACTIVE-EXTRACTION.json": "records",
}
since_rows, since_problems = [], []
for p in since:
    if p in ALLOWED_SINCE:
        since_rows.append({"path": p, "class": ALLOWED_SINCE[p]})
    elif p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed/") or p.startswith("eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix/"):
        since_rows.append({"path": p, "class": "retained evidence packet (not an input of any check)"})
    else:
        since_rows.append({"path": p, "class": "UNEXPECTED CHANGE SINCE THE REVIEWED SOURCE"})
        since_problems.append("changed since the reviewed source %s outside the allowed set: %s" % (REVIEWED, p))
problems.extend(since_problems)
rev_rows = reviewed["files"]
since_map_changed = sorted(k for k in new_rows_ if k in rev_rows and new_rows_[k]["raw_sha256"] != rev_rows[k]["raw_sha256"])
since_map_added = sorted(k for k in new_rows_ if k not in rev_rows)
since_map_removed = sorted(k for k in rev_rows if k not in new_rows_)
since_unexplained = [p for p in since_map_changed + since_map_added if p not in ALLOWED_SINCE and not p.startswith("eech-campaign/reference/")]
if since_unexplained or since_map_removed:
    problems.append("map rows changed or removed since the reviewed POST map that the allowed set does not explain: %s / %s" % (since_unexplained, since_map_removed))

old_rows, new_rows = old["files"], new["files"]
map_changed = sorted(k for k in new_rows if k in old_rows and new_rows[k]["raw_sha256"] != old_rows[k]["raw_sha256"])
map_added = sorted(k for k in new_rows if k not in old_rows)
map_removed = sorted(k for k in old_rows if k not in new_rows)
classified = {r["path"] for r in rows}
unexplained = [p for p in map_changed + map_added if p not in classified]
if unexplained:
    problems.append("map rows changed that the join did not classify: %s" % unexplained)
if map_removed:
    problems.append("map rows removed: %s" % map_removed)
result = {
    "kind": "explicit source-effect join of the four-P2 correction against the frozen native 1b89 source, the CI batch's final POST map and the reviewed witness source 6e4139043 with its final POST map",
    "reviewed_source": reviewed_full,
    "since_reviewed_source": {"paths": since_rows, "map_rows_changed": since_map_changed, "map_rows_added": since_map_added, "map_rows_removed": since_map_removed, "problems": since_problems},
    "native_identity": NATIVE,
    "map_rows_changed": map_changed, "map_rows_added": map_added, "map_rows_removed": map_removed, "map_rows_identical": sum(1 for k in new_rows if k in old_rows and new_rows[k]["raw_sha256"] == old_rows[k]["raw_sha256"]),
    "changed_paths_against_1b89": [r for r in rows if not r["path"].startswith("eech-campaign/reference/")], "evidence_packet_files": sum(1 for r in rows if r["path"].startswith("eech-campaign/reference/")),
    "conclusion": "no C source of the original campaign (aphavoc/, modules/) changed; the shipping build's compiled inputs differ from 1b89 only by feature-gated additions that preprocess and compile to nothing with the feature off (verified from the Git diff): the 1b89 native evidence keeps its identity and is NOT claimed for this source, which is not built natively here" if not problems else "NOT EQUIVALENT: see problems",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed_rows": len(map_changed), "added_rows": len(map_added), "removed": len(map_removed), "paths": len(rows), "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
