#!/usr/bin/env python3
"""Concatenate the retained campaign source into campaign-bundle.txt.

Deterministic: files are the tracked *.c / *.h sources (git ls-files),
ordered by byte-wise path. Each file is preceded by a path marker line.
Empty and whitespace-only lines are dropped; every other line is copied
byte for byte (only a trailing CR of a CRLF line ending is removed).

Usage: python3 build-campaign-bundle.py [output]   (default: campaign-bundle.txt)
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "campaign-bundle.txt")
SUFFIXES = (".c", ".h")


def source_files():
    out = subprocess.check_output(["git", "-C", ROOT, "ls-files", "-z"])
    paths = [p.decode("utf-8") for p in out.split(b"\0") if p]
    paths = [p for p in paths if p.lower().endswith(SUFFIXES)]
    return sorted(paths, key=lambda p: p.encode("utf-8"))


def main():
    chunks = []
    for path in source_files():
        with open(os.path.join(ROOT, path), "rb") as f:
            data = f.read()
        lines = [l[:-1] if l.endswith(b"\r") else l for l in data.split(b"\n")]
        kept = [l for l in lines if l.strip()]
        if not kept:
            continue
        chunks.append(b"=== " + path.encode("utf-8") + b" ===")
        chunks.extend(kept)
    with open(OUT, "wb") as f:
        f.write(b"\n".join(chunks) + b"\n")


if __name__ == "__main__":
    main()
