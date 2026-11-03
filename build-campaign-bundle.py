#!/usr/bin/env python3
"""Concatenate the retained campaign source into campaign-bundle.txt and
campaign-bundle-appendix.txt.

The appendix file holds per-item reference data tables that matter to
weapon loadouts and damage but not to running the campaign: the
per-unit weapon loadout tables, the weapon database and the per-3D-
object armour and importance table. Everything else goes in the main bundle.

Deterministic: files are the tracked *.c / *.h sources (git ls-files),
ordered by byte-wise path. Each file is preceded by a path marker line.
Empty and whitespace-only lines are dropped; every other line is copied
byte for byte (only a trailing CR of a CRLF line ending is removed).

Usage: python3 build-campaign-bundle.py [output [appendix_output]]
"""

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "campaign-bundle.txt")
APPENDIX_OUT = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "campaign-bundle-appendix.txt")
SUFFIXES = (".c", ".h")

S = "aphavoc/source/"
APPENDIX = [re.compile(p) for p in (
    S + r"entity/mobile/weapon/configs/",   # per-unit weapon loadouts
    S + r"entity/mobile/weapon/wn_dbase\.c$",  # weapon database
    S + r"entity/fixed/fx_objdb\.c$",       # per-3D-object armour/importance
)]


def is_appendix(path):
    return any(p.search(path) for p in APPENDIX)


def source_files():
    out = subprocess.check_output(["git", "-C", ROOT, "ls-files", "-z"])
    paths = [p.decode("utf-8") for p in out.split(b"\0") if p]
    paths = [p for p in paths if p.lower().endswith(SUFFIXES)]
    return sorted(paths, key=lambda p: p.encode("utf-8"))


def write_bundle(paths, out):
    chunks = []
    for path in paths:
        with open(os.path.join(ROOT, path), "rb") as f:
            data = f.read()
        lines = [l[:-1] if l.endswith(b"\r") else l for l in data.split(b"\n")]
        kept = [l for l in lines if l.strip()]
        if not kept:
            continue
        chunks.append(b"=== " + path.encode("utf-8") + b" ===")
        chunks.extend(kept)
    with open(out, "wb") as f:
        f.write(b"\n".join(chunks) + b"\n")


def main():
    paths = source_files()
    write_bundle([p for p in paths if not is_appendix(p)], OUT)
    write_bundle([p for p in paths if is_appendix(p)], APPENDIX_OUT)


if __name__ == "__main__":
    main()
