"""PRE-FLIGHT of the run receipts a freeze will read (the process improvement of Root's feedback 16:26): derives every receipt path from the ACTUAL run records on disk and asserts that each record, its stdout log and its stderr log exist, that the record is terminal and that its
helper is the one the batch declared, BEFORE the timed PRE. A syntax check of a copied publisher cannot catch a stale path prefix; this does. Usage: preflight.py <prefix> <label-regex> <expected-helper-name> [<out.json>]
Example: preflight.py flags-consumer- '^[ck]\\d+-' check-physical-flags-consumer-linux-20261107.sh"""
import json
import pathlib
import re
import sys

EXT = pathlib.Path("R:/d3-evidence")
prefix, label_re, helper = sys.argv[1:4]
out = sys.argv[4] if len(sys.argv) > 4 else None
rows, problems = [], []
records = sorted(p for p in EXT.glob(prefix + "*.json") if re.match(label_re, p.name[len(prefix):]))
if not records:
    problems.append("no run record matches %s%s" % (prefix, label_re))
for record in records:
    stem = record.name[: -len(".json")]
    data = json.loads(record.read_text(encoding="utf-8"))
    parts = {suffix: (EXT / ("%s.%s" % (stem, suffix))) for suffix in ("stdout.log", "stderr.log")}
    missing = [name for name, path in parts.items() if not path.exists()]
    uses_helper = any(helper in a for a in data.get("args", []))
    rows.append({"record": record.name, "state": data.get("state"), "actual_exit": data.get("actual_exit"), "stdout_exists": parts["stdout.log"].exists(), "stderr_exists": parts["stderr.log"].exists(), "uses_declared_helper": uses_helper})
    if missing:
        problems.append("%s: missing %s" % (record.name, missing))
    if data.get("state") != "terminal":
        problems.append(record.name + ": not terminal")
    if not uses_helper:
        problems.append(record.name + ": not run with " + helper)
result = {"prefix": prefix, "label_regex": label_re, "declared_helper": helper, "records": len(rows), "rows": rows, "problems": problems}
if out:
    pathlib.Path(out).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"records": len(rows), "problems": problems}))
sys.exit(1 if problems else 0)
