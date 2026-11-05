#!/usr/bin/env python3
"""Verify every retained host-identity artifact against README's manifest."""

from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPECTED = {
    "BUILD-INFO.txt",
    "accepted-lifecycle.summary.txt",
    "host-lifecycle.summary.txt",
    "boot.summary.txt",
    "audit-3h.summary.txt",
    "controls-puc.results.jsonl",
    "controls-puc.summary.txt",
    "controls-dcs.results.jsonl",
    "controls-dcs.summary.txt",
    "pack.report.json",
    "pack.report.md",
}
ROW = re.compile(r"^\| \[([^]]+)]\(([^)]+)\) \| `([0-9a-f]{64})` \|$")
SECTION = "## Retained file hashes"
HEADER = "| File | SHA-256 |"
SEPARATOR = "|---|---|"


def retained_hash_rows(readme: str) -> list[tuple[str, str, str]]:
    lines = readme.splitlines()
    headings = [index for index, line in enumerate(lines) if line == SECTION]
    if len(headings) != 1:
        raise SystemExit(f"expected exactly one {SECTION!r} section, found {len(headings)}")

    rows: list[tuple[str, str, str]] = []
    saw_header = False
    saw_separator = False
    for line in lines[headings[0] + 1 :]:
        if line.startswith("## "):
            break
        if not line:
            continue
        if line == HEADER:
            if saw_header or saw_separator:
                raise SystemExit("malformed retained-hash table header")
            saw_header = True
            continue
        if line == SEPARATOR:
            if not saw_header or saw_separator:
                raise SystemExit("malformed retained-hash table separator")
            saw_separator = True
            continue
        if not line.lstrip().startswith("|"):
            continue

        match = ROW.fullmatch(line)
        if not match:
            raise SystemExit(f"malformed retained-hash row: {line}")
        if not saw_separator:
            raise SystemExit(f"retained-hash row precedes table separator: {line}")
        rows.append(match.groups())

    if not saw_header or not saw_separator:
        raise SystemExit("retained-hash table header or separator missing")
    return rows


def main() -> int:
    records: dict[str, str] = {}
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    for label, path, digest in retained_hash_rows(readme):
        if label != path or Path(path).name != path:
            raise SystemExit(f"invalid retained-hash path: {label!r} -> {path!r}")
        if path in records:
            raise SystemExit(f"duplicate retained-hash row: {path}")
        records[path] = digest

    if set(records) != EXPECTED:
        missing = sorted(EXPECTED - set(records))
        extra = sorted(set(records) - EXPECTED)
        raise SystemExit(f"retained-hash manifest mismatch: missing={missing}, extra={extra}")

    for path, expected in records.items():
        artifact = ROOT / path
        if not artifact.is_file():
            raise SystemExit(f"missing retained artifact: {path}")
        actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
        if actual != expected:
            raise SystemExit(f"retained artifact hash mismatch: {path}: {actual}, expected {expected}")

    print(f"PASS: {len(records)} retained host-identity artifact hashes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
