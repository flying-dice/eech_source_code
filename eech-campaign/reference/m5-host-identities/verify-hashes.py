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


def main() -> int:
    records: dict[str, str] = {}
    for line in (ROOT / "README.md").read_text(encoding="utf-8").splitlines():
        match = ROW.match(line)
        if not match:
            continue
        label, path, digest = match.groups()
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
