"""Private evidence analysis: strict bytes except the documented f41 blast COUNTER type.

Never writes either trace. This does not claim raw byte equality or replace the
repository's completeness audit, process exits, or strict campaign comparisons.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

MAX_BYTES = 16 * 1024 * 1024
COUNTER = re.compile(rb'"blast":"(0|[1-9][0-9]*)"')


def declared_change(old, new):
    row = json.loads(old)
    if row.get("event") != "cmb-kind" or not isinstance(row.get("blast"), str):
        return None
    expected, replacements = COUNTER.subn(rb'"blast":\1', old)
    if replacements != 1 or expected != new:
        return None
    return {"scope": row["scope"], "type": row["type"], "value": int(row["blast"])}


def read_bounded(path):
    with path.open("rb") as stream:
        raw = stream.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise ValueError(f"Trace exceeds declared {MAX_BYTES}-byte analysis bound: {path}")
    return raw


def self_test():
    old = b'{"blast":"69","event":"cmb-kind","scope":"run","type":"SITE"}\n'
    good = old.replace(b'"blast":"69"', b'"blast":69')
    assert declared_change(old, good)["value"] == 69
    assert declared_change(old, good.replace(b':69', b':70')) is None
    assert declared_change(old, good.replace(b'"SITE"', b'"PYLON"')) is None
    assert declared_change(old.replace(b'cmb-kind', b'cmb-hash'), good.replace(b'cmb-kind', b'cmb-hash')) is None
    assert declared_change(old, good.rstrip(b'\n')) is None
    assert declared_change(old, old) is None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--baseline-sha256", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        raise FileExistsError("Refusing to overwrite an analysis result")
    self_test()
    old, new = read_bounded(args.baseline), read_bounded(args.candidate)
    baseline_sha = hashlib.sha256(old).hexdigest()
    if baseline_sha != args.baseline_sha256:
        raise ValueError("Baseline raw SHA-256 does not match the declared immutable original")
    a, b = old.splitlines(keepends=True), new.splitlines(keepends=True)
    changes, differences = [], []
    for index in range(max(len(a), len(b))):
        left = a[index] if index < len(a) else None
        right = b[index] if index < len(b) else None
        if left == right:
            continue
        change = declared_change(left, right) if left is not None and right is not None else None
        if change is not None:
            changes.append({"line": index + 1, **change})
        else:
            differences.append(index + 1)
    result = {"scope": "Strict complete trace bytes with ONLY declared cmb-kind blast string-to-number typing differences; raw files unchanged",
              "baseline": {"path": str(args.baseline), "bytes": len(old), "sha256": baseline_sha},
              "candidate": {"path": str(args.candidate), "bytes": len(new), "sha256": hashlib.sha256(new).hexdigest()},
              "raw_bytes_identical": old == new, "baseline_lines": len(a), "candidate_lines": len(b),
              "declared_schema_changes": changes, "unexpected_difference_count": len(differences),
              "first_unexpected_line_numbers": differences[:20], "passed": not differences,
              "separate_gates": "Actual native/driver exits, repository completeness/schema audit, build/input identity, and strict full campaign recording comparisons remain required"}
    with args.out.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps({key: result[key] for key in ("raw_bytes_identical", "unexpected_difference_count", "passed")}))
    print("declared schema-only lines:", len(changes))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
