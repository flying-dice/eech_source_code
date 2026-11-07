"""Explicit actual-input join of the public-proof-plan correction against the frozen production source's final POST map (0be1dd846): which rows changed, were added or removed, and what compiled input
each can reach. Usage: join.py <repo> <old-post-map> <new-post-map> <out.json>"""
import json
import re
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
old = json.load(open(old_path, encoding="utf-8"))["files"]
new = json.load(open(new_path, encoding="utf-8"))["files"]


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout


def strip_comments(text):
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def outer_gate_only(text, macro):
    lines = [l.strip() for l in strip_comments(text).splitlines() if l.strip()]
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


changed = sorted(k for k in new if k in old and new[k]["raw_sha256"] != old[k]["raw_sha256"])
added = sorted(k for k in new if k not in old)
removed = sorted(k for k in old if k not in new)
identical = sum(1 for k in new if k in old and new[k]["raw_sha256"] == old[k]["raw_sha256"])
rows, problems = [], []
for p in changed + added:
    status = "changed" if p in old else "added"
    if p.startswith("aphavoc/") or p.startswith("modules/"):
        cls = "UNCLASSIFIED: could enter the ordinary build"
        problems.append(p + ": a path under aphavoc/ or modules/ changed")
    elif "/tests/" in p or p.endswith(".md"):
        cls = "not compiled into the DLL (test source or documentation)"
    elif p.endswith("/lua/entity-authority.lua") or p.endswith("/lua/sector-height-baseline.lua"):
        cls = "host Lua driver run by eech-world (not compiled into the campaign DLL); its tests are in this change"
    elif p.endswith("csrc/eech_sho.c") or p.endswith("csrc/eech_sector_height_check.c"):
        macro = "EECH_SECTOR_HEIGHT_OBSERVER" if p.endswith("eech_sho.c") else "EECH_SECTOR_HEIGHT_CHECK"
        now = open(repo + "/" + p, "rb").read().decode("latin-1")
        was = git("show", "5aae2256f02ca3d79e762d71c9b79c20dc34856b:" + p).decode("latin-1")
        ok = outer_gate_only(now, macro) and outer_gate_only(was, macro)
        cls = "translation unit that preprocesses to nothing without " + macro + " (one outer #ifdef...#endif pair in the old and the new text)"
        if not ok:
            problems.append(p + ": not wholly inside " + macro)
    elif p.endswith("eech-engine-sys/Cargo.toml"):
        diff = git("diff", "-U0", "5aae2256f02ca3d79e762d71c9b79c20dc34856b", "--", p).decode().splitlines()
        body = [l for l in diff if (l.startswith("+") or l.startswith("-")) and not l.startswith("+++") and not l.startswith("---")]
        ok = body == ['+[[test]]', '+name = "sector_height_hosted_clearance"', '+path = "tests/sector_height_hosted_clearance.rs"', '+required-features = ["sector-height-check", "sector-height-observer"]', "+"]
        cls = "one added [[test]] target (feature-gated); no dependency, feature or build change"
        if not ok:
            problems.append(p + ": the diff is not exactly the added test target")
    else:
        cls = "UNCLASSIFIED"
        problems.append(p + ": unclassified")
    rows.append({"path": p, "status": status, "class": cls})
if removed:
    problems.append("rows removed: %s" % removed)
result = {"kind": "explicit actual-input join of the proof-guards correction against the previous correction (5aae2256f) final POST map", "old_map": old_path, "new_map": new_path,
          "rows_identical": identical, "rows_changed": len(changed), "rows_added": len(added), "rows_removed": len(removed), "rows": rows, "problems": problems,
          "conclusion": "the ordinary hosted DLL's compiled inputs are unchanged (no aphavoc/ or modules/ path, no non-test crate source, only observer- or check-gated translation units, test sources, host Lua drivers and documentation changed): the 0be1dd846 matrices of the ordinary build (workspace tests, clippy of the unchanged crate sources, shared kernel and original-solver control) are reused at their own scope, and every changed row has its own affected check in this batch" if not problems else "NOT EQUIVALENT: see problems"}
json.dump(result, open(out, "w", encoding="utf-8", newline="\n"), indent=2)
print(json.dumps({"changed": len(changed), "added": len(added), "removed": len(removed), "identical": identical, "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
