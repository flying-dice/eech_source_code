"""Source/compiled-profile-input equivalence join between the sealed ordinary eff3b4b33 binary's inputs and the capacity-fix tree, for the FEATURE-OFF (ordinary) profile.

Source-derived and structural, NOT an object comparison and not a relabel of the sealed binary: it shows that every file that differs from eff3b4b33 either is not compiled into the DLL (tests, data,
docs, governance prose), or is a translation unit that preprocesses to nothing without the observer macro, or is one cfg-gated Rust constant. Usage: join.py <repo> <eff3-commit> <out.json>
"""
import json
import re
import subprocess
import sys

repo, base, out = sys.argv[1:4]


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout


changed = [l.split("\t", 2)[2] for l in git("diff", "--numstat", base).decode().splitlines()]
untracked = [l for l in git("ls-files", "--others", "--exclude-standard").decode().splitlines()]
paths = sorted(set(changed) | set(untracked))


def strip_comments(text):
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def outer_gate_only(text, macro):
    """True when, outside comments, the file is exactly one `#ifdef macro ... #endif` pair (nothing before, between or after)."""
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


rows = []
problems = []
for p in paths:
    if p == "STATUS.md" or p.startswith("docs/") or p.startswith("eech-campaign/docs/") or "/tests/" in p or p.startswith("eech-campaign/reference/"):
        rows.append({"path": p, "class": "not compiled into the DLL (test, data, documentation or governance prose)"})
        continue
    if p == "eech-campaign/crates/eech-engine-sys/csrc/eech_sho.c":
        now = open(repo + "/" + p, "rb").read().decode("latin-1")
        was = git("show", base + ":" + p).decode("latin-1")
        ok = outer_gate_only(now, "EECH_SECTOR_HEIGHT_OBSERVER") and outer_gate_only(was, "EECH_SECTOR_HEIGHT_OBSERVER")
        rows.append({"path": p, "class": "translation unit that preprocesses to nothing without EECH_SECTOR_HEIGHT_OBSERVER (checked on the eff3 text and the new text: one outer #ifdef...#endif pair, nothing else outside comments)",
                     "outer_gate_only_before": outer_gate_only(was, "EECH_SECTOR_HEIGHT_OBSERVER"), "outer_gate_only_after": outer_gate_only(now, "EECH_SECTOR_HEIGHT_OBSERVER")})
        if not ok:
            problems.append(p + ": not wholly inside the observer gate")
        continue
    if p == "eech-campaign/crates/eech-engine/src/lib.rs":
        diff = git("diff", "-U1", base, "--", p).decode().splitlines()
        body = [l for l in diff if (l.startswith("+") or l.startswith("-")) and not l.startswith("+++") and not l.startswith("---")]
        minus = [i for i, l in enumerate(diff) if l.startswith("-pub const SHO_EVENTS_MAX")]
        ctx_before = [diff[i - 1] for i in minus]
        ok = len(body) == 2 and len(minus) == 1 and body[1].startswith("+pub const SHO_EVENTS_MAX") and ctx_before == [' #[cfg(feature = "sector-height-observer")]']
        rows.append({"path": p, "class": "one cfg(feature = \"sector-height-observer\") constant changed (SHO_EVENTS_MAX)", "diff_body": body, "line_before_new_const": ctx_before})
        if not ok:
            problems.append(p + ": the diff is not exactly the gated constant")
        continue
    rows.append({"path": p, "class": "UNCLASSIFIED: could enter the ordinary build"})
    problems.append(p + ": unclassified")
result = {
    "base": base, "kind": "source-derived structural join for the feature-off profile; not an object comparison", "changed_or_added_paths": paths, "rows": rows,
    "no_aphavoc_or_modules_path": not any(p.startswith("aphavoc/") or p.startswith("modules/") for p in paths),
    "problems": problems,
    "conclusion": "the compiled inputs of the ORDINARY (feature-off) profile are unchanged in effect: a binary built from eff3b4b33 and one built from this tree differ in no preprocessed translation unit and no non-gated Rust item; the sealed ordinary eff3 binary keeps its true identity and is not relabelled" if not problems else "NOT EQUIVALENT: see problems",
}
json.dump(result, open(out, "w", encoding="utf-8", newline="\n"), indent=2)
print(json.dumps({"paths": paths, "problems": problems, "conclusion": result["conclusion"][:90]}, indent=1))
sys.exit(1 if problems else 0)
