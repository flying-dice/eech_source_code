#!/usr/bin/env python3
"""Reproduces the M5 DCS call map's script figures from the map itself.

The map (eech-campaign/docs/m5-dcs-call-map.md) embeds six scripts in fenced
blocks. This tool extracts them from the document, runs them against the
source snapshot the map describes (read from git, never the working tree),
and checks the figures the document states as their output:
  - the headline figures, parsed from the document's own prose (ledger items,
    mapped, excluded, outstanding; rule kinds; the 57 rules deciding 750 items;
    globals; world writes), so a change to either side shows as a difference;
  - the coverage-ledger table in section 1 and the rule table in section 10,
    parsed from the document;
  - the remaining prose figures listed in EXPECTED (a copy of the document's
    numbers, kept only where the document states them in running text).
It also prints whether the snapshot still applies to the merge target:
`git diff --quiet SNAPSHOT TARGET -- aphavoc eech-campaign/crates` holds.

Usage (from anywhere in the repository):
  m5-call-map-check.py [TARGET] [--doc PATH] [--snapshot REV]
  m5-call-map-check.py --extract DIR    write the embedded scripts to DIR and stop
TARGET defaults to HEAD. Exit status: 0 every figure matches; 1 a figure or
script differs from the document; 2 usage or setup error (for example the
snapshot commit is not in the clone: fetch it).
"""
import argparse
import io
import json
import re
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

SNAPSHOT = "65892de17bf5c4bb9ab6e9f8af70779e35b0106b"
SCRIPTS = ["eechmap.py", "inventory.py", "reach.py", "dispatch.py", "globals.py", "ledger.py"]
SOURCE_PATHS = ["aphavoc", "modules", "eech-campaign/crates/eech-engine-sys"]  # everything the scripts read
MERGE_PATHS = ["aphavoc", "eech-campaign/crates"]  # the runtime source a merge target must leave unchanged
BLOCK = re.compile(r"<summary><code>([\w.]+\.py)</code></summary>\n\n```python\n(.*?)\n```", re.S)
REPO = Path(__file__).resolve().parent  # replaced by the repository root in main()


class Failure(Exception):
    """a script or the document differs from what the map states (exit 1)"""


class SetupError(Exception):
    """the check cannot run (exit 2)"""


def git(*args, binary=False, check=True):
    """stdout of a git command run at the repository root; the completed process when check is false"""
    try:
        out = subprocess.run(["git", *args], capture_output=True, cwd=REPO)
    except OSError as e:
        raise SetupError(f"cannot run git: {e}")
    if check and out.returncode:
        raise SetupError(f"git {' '.join(args)} failed: {out.stderr.decode(errors='replace').strip()}")
    return out if not check else (out.stdout if binary else out.stdout.decode())


# figures the document's prose states as script output: (script, key in its JSON summary, value).
# The headline figures are parsed from the document instead (see PROSE).
EXPECTED = [
    ("inventory", "files_by_class", {"C": 159, "E": 73, "F": 136, "W": 332, "X": 517}),
    ("inventory", "names_defined_in_several_classes_excluded", 76),
    ("inventory", "C->W", {"functions": 39, "sites": 86}),
    ("inventory", "C->E", {"functions": 25, "sites": 75}),
    ("inventory", "W->C", {"functions": 64, "sites": 261}),
    ("inventory", "E->C", {"functions": 2, "sites": 4}),
    ("inventory", "C attribute tokens", {"distinct": 208, "sites": 2698}),
    ("reach", "functions", {"C": 1139, "E": 889, "F": 1086, "H": 294, "W": 1813, "X": 6114}),
    ("reach", "reachable_functions", {"C": 997, "E": 799, "F": 914, "H": 266, "W": 1603, "X": 5677}),
    ("reach", "unresolved_ambiguous_names", 0),
    ("reach", "call_sites_examined_in_campaign", 14599),
    ("reach", "unresolved_call_names_in_campaign", ["endif", "function", "ogre_scene_get_object"]),
    ("dispatch", "registrations", 2879),
    ("dispatch", "entity_types", {"W": 23, "C": 11, "X": 2, "F": 3, "E": 1}),
    ("dispatch", "crossing_static_messages", 30),
    ("dispatch", "dynamic_send_sites", 3),
    ("dispatch", "attribute_tokens_in_campaign_code", 330),
    ("dispatch", "function_tables", 22),
    ("dispatch", "other_function_table_forms", 6),
    ("globals", "externs", 1659),
    ("globals", "only_through_macros", 36),
    ("globals", "by_class", {"X": 18, "W": 8, "F": 61, "E": 8}),
    ("ledger", "ambiguous names left", 0),
]
# the section 1 ledger table's row labels -> the ledger report's keys
LEDGER_ROWS = {
    "Campaign → host layer (H)": "calls C->H", "Host layer → campaign": "calls H->C",
    "Campaign → framework (F)": "calls C->F", "Framework → campaign": "calls F->C",
    "Campaign → presentation files (X)": "calls C->X", "Presentation files → campaign": "calls X->C",
    "Campaign → world (W)": "calls C->W", "World → campaign": "calls W->C",
    "Campaign → environment (E)": "calls C->E", "Environment → campaign": "calls E->C",
    "Static message types crossing the boundary": "messages", "Entity function tables": "tables",
    "Other function-pointer table forms": "other tables", "Dynamic message sends": "dynamic",
    "`waypoint_database` message ids": "waypoint ids", "Attribute tokens in campaign code": "attributes",
    "Campaign writes to world-implemented attributes": "writes", "Non-campaign global data": "globals",
    "Call-shaped names with no definition": "unresolved names", "**Total**": "ALL",
}
# the sentences of the document that carry the headline figures
PROSE = {
    "ledger": r"accounts for ([\d,]+) items: ([\d,]+) are mapped[^,]*, ([\d,]+) are excluded[^*]*\*\*([\d,]+) are outstanding\*\*",
    "rules": r"exact rules and fixed tables decide ([\d,]+), ([\d,]+) pattern rules decide ([\d,]+), "
             r"and ([\d,]+) fallbacks decide ([\d,]+)",
    "rule total": r"So ([\d,]+) pattern and fallback rules decide ([\d,]+) items",
    "globals": r"([\d,]+) non-campaign globals used by the campaign",
    "writes": r"([\d,]+) campaign writes to world-implemented tokens\.",
}


def extract_scripts(doc_text, directory):
    found = {name: code for name, code in BLOCK.findall(doc_text)}
    missing = [s for s in SCRIPTS if s not in found]
    if missing:
        raise Failure(f"the document embeds no block for {', '.join(missing)}")
    try:
        Path(directory).mkdir(parents=True, exist_ok=True)
    except OSError as e:
        raise SetupError(f"cannot create script directory {directory}: {e}") from e
    for name in SCRIPTS:
        try:
            (Path(directory) / name).write_text(found[name] + "\n", encoding="utf-8", newline="\n")
        except OSError as e:
            raise SetupError(f"cannot write {name} in {directory}: {e}") from e


def materialise_snapshot(rev, directory):
    """the files the scripts read, from git at rev"""
    if git("cat-file", "-e", f"{rev}^{{commit}}", check=False).returncode:
        raise SetupError(f"the source snapshot {rev} is not in this clone; fetch it with: git fetch origin {rev}")
    archive = git("archive", "--format=tar", rev, *SOURCE_PATHS, binary=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(directory, **({"filter": "data"} if hasattr(tarfile, "data_filter") else {}))


def run(script_dir, script, *args, cwd):
    out = subprocess.run([sys.executable, "-I", str(Path(script_dir) / script), *args], cwd=cwd,
                         capture_output=True, text=True, encoding="utf-8")
    if out.returncode:
        raise Failure(f"{script} exited {out.returncode}:\n{out.stderr.strip()}")
    try:
        end = out.stdout.rindex("}") + 1
        report = json.loads(out.stdout[:end])
        if not isinstance(report, dict):
            raise ValueError("top-level report is not an object")
        return report, out.stdout
    except ValueError as e:
        raise Failure(f"{script} exited successfully but produced no valid JSON report: {e}") from e


def number(text):
    return int(text.strip().strip("*").replace(",", ""))


def prose_figures(doc_text):
    """the headline figures the document's sentences state: {name: tuple of ints}"""
    figures = {}
    for name, pattern in PROSE.items():
        m = re.search(pattern, doc_text)
        if not m:
            raise Failure(f"the document no longer states its {name} figures (pattern: {pattern})")
        figures[name] = tuple(number(g) for g in m.groups())
    return figures


def ledger_table(doc_text):
    """section 1's coverage-ledger table: report key -> {mapped, excluded, outstanding}"""
    rows = {}
    for line in doc_text.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 5 and cells[0] in LEDGER_ROWS:
            rows[LEDGER_ROWS[cells[0]]] = {"mapped": number(cells[2]), "excluded": number(cells[3]),
                                          "outstanding": number(cells[4])}
    return rows


def pattern_table(doc_text):
    """section 10's rule table: sorted (rule id, disposition, items)"""
    return sorted((m.group(1), m.group(2), int(m.group(3))) for m in
                  re.finditer(r"^\| [\w ]+ \| `((?:pattern|fallback):[\w:#-]+)` \| (\w+) \| .*? \| (\d+) \|$",
                              doc_text, re.M))


class Report:
    def __init__(self):
        self.failures = []

    def compare(self, label, actual, expected, summary=None):
        if actual == expected:
            print(f"ok   {label}: {summary if summary is not None else json.dumps(actual)}")
        else:
            print(f"FAIL {label}:\n       expected (document) {json.dumps(expected)}\n"
                  f"       actual   (scripts)  {json.dumps(actual)}")
            self.failures.append(label)


def check_figures(doc_text, scripts, tree, report):
    out = {"inventory": run(scripts, "inventory.py", ".", cwd=tree)[0]}
    for name in ("reach", "dispatch", "globals"):
        out[name] = run(scripts, f"{name}.py", tree, "--detail", f"{scripts}/{name}.json", cwd=tree)[0]
    out["ledger"], ledger_text = run(scripts, "ledger.py", f"{scripts}/reach.json", f"{scripts}/dispatch.json",
                                     f"{scripts}/globals.json", "--items", f"{scripts}/items.json", cwd=tree)
    items = json.loads(Path(scripts, "items.json").read_text(encoding="utf-8"))
    prose = prose_figures(doc_text)
    ledger, kinds = out["ledger"], out["ledger"]["items by rule kind"]
    outstanding = len([l for l in ledger_text.splitlines() if l.startswith("OUTSTANDING")])

    # headline figures: the document's sentences against the scripts
    report.compare("ledger items / mapped / excluded / outstanding",
                   [len(items), ledger["ALL"]["mapped"], ledger["ALL"]["excluded"], outstanding], list(prose["ledger"]))
    report.compare("rule kinds exact / pattern / fallback", [kinds["exact"], kinds["pattern"], kinds["fallback"]],
                   [prose["rules"][0], prose["rules"][2], prose["rules"][4]])
    deciding = [i for i in items if i["rule"].startswith(("pattern", "fallback"))]
    report.compare("pattern + fallback rules / items they decide",
                   [ledger["pattern and fallback rules that decide items"], len(deciding)], list(prose["rule total"]))
    report.compare("pattern rules + fallbacks, as the document sums them",
                   prose["rules"][1] + prose["rules"][3], prose["rule total"][0])
    report.compare("globals used by the campaign", out["globals"]["non_campaign_globals_used_by_campaign"],
                   prose["globals"][0])
    report.compare("campaign writes to world-implemented tokens",
                   out["dispatch"]["campaign_writes_to_world_implemented_attributes"], prose["writes"][0])

    for script, key, want in EXPECTED:
        report.compare(f"{script}: {key}", out[script].get(key), want)

    documented = ledger_table(doc_text)
    report.compare("section 1 ledger table: rows", sorted(documented), sorted(LEDGER_ROWS.values()))
    for key, want in sorted(documented.items()):
        got = {d: ledger.get(key, {}).get(d, 0) for d in ("mapped", "excluded", "outstanding")}
        report.compare(f"section 1 ledger table: {key}", got, want)
    counts = {}
    for i in deciding:
        counts[(i["rule"], i["disposition"])] = counts.get((i["rule"], i["disposition"]), 0) + 1
    rules = sorted([r, d, n] for (r, d), n in counts.items())
    report.compare("section 10 rule table", rules, [list(r) for r in pattern_table(doc_text)],
                   summary=f"{len(rules)} rules")


def main():
    global REPO
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("target", nargs="?", default="HEAD", help="merge target to compare with the snapshot")
    parser.add_argument("--doc", default="eech-campaign/docs/m5-dcs-call-map.md")
    parser.add_argument("--snapshot", default=SNAPSHOT)
    parser.add_argument("--extract", metavar="DIR", help="write the embedded scripts to DIR and stop")
    args = parser.parse_args()
    try:
        REPO = Path(git("rev-parse", "--show-toplevel").strip())
        try:
            doc_text = (REPO / args.doc).read_text(encoding="utf-8")
        except OSError as e:
            raise SetupError(f"cannot read the document: {e}")
        if args.extract:
            extract_scripts(doc_text, args.extract)
            print(f"extracted {', '.join(SCRIPTS)} to {args.extract}")
            return 0
        if git("rev-parse", "--verify", "-q", f"{args.target}^{{commit}}", check=False).returncode:
            raise SetupError(f"merge target {args.target} is not a commit in this clone")
        report = Report()
        with tempfile.TemporaryDirectory() as scripts, tempfile.TemporaryDirectory() as tree:
            extract_scripts(doc_text, scripts)
            materialise_snapshot(args.snapshot, tree)
            print(f"scripts from {args.doc}; source snapshot {args.snapshot} (git archive)")
            try:
                check_figures(doc_text, scripts, tree, report)
            except (KeyError, TypeError) as e:
                raise Failure(f"an embedded script report has a missing or invalid field: {e}") from e
        command = f"git diff --quiet {args.snapshot[:8]} {args.target} -- {' '.join(MERGE_PATHS)}"
        target_diff = git("diff", "--quiet", args.snapshot, args.target, "--", *MERGE_PATHS, check=False)
        if target_diff.returncode == 0:
            print(f"merge target: HOLDS ({command}): the snapshot applies to {args.target}")
        elif target_diff.returncode == 1:
            names = git("diff", "--name-only", args.snapshot, args.target, "--", *MERGE_PATHS).split()
            print(f"merge target: DOES NOT HOLD ({command}): {len(names)} files differ, first {names[0]}; "
                  f"the map's figures describe {args.snapshot[:8]}, not {args.target}")
        else:
            raise SetupError(f"cannot compare the merge target with the snapshot: "
                             f"{target_diff.stderr.decode(errors='replace').strip()}")
        print(f"{'PASS' if not report.failures else 'FAIL'}: {len(report.failures)} figures differ from the document")
        return 1 if report.failures else 0
    except Failure as e:
        print(f"FAIL m5-call-map-check: {e}", file=sys.stderr)
        return 1
    except SetupError as e:
        print(f"m5-call-map-check: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
