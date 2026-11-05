#!/usr/bin/env python3
"""Verify retained bytes, and optionally compare them with the uncommitted originals.

The original check needs the three source directories supplied by the holder of
those files. With no arguments, this checks every retained file against the
committed RETAINED.sha256 manifest. It does not reconstruct original hashes.
"""
import argparse
import hashlib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def entries(name):
    for line in (HERE / name).read_text(encoding="utf-8").splitlines():
        if line and not line.startswith("#"):
            digest, relative = line.split("  ", 1)
            yield relative, digest


def sanitise(original, user_profile):
    text = original.decode("utf-8-sig")
    canonical = user_profile.replace("\\", "/").rstrip("/")
    forms = (canonical.replace("/", "\\\\"), canonical.replace("/", "\\"),
             canonical, canonical.lower(), canonical.lower().replace("/", "\\"))
    for form in forms:
        text = text.replace(form, "%USERPROFILE%")
    return text.replace("\r\n", "\n").encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original-runs", type=Path)
    parser.add_argument("--original-controls", type=Path)
    parser.add_argument("--original-microprobe", type=Path)
    parser.add_argument("--original-user-profile", help="original Windows profile path, e.g. C:/Users/name")
    args = parser.parse_args()
    sources = {
        "runs/": args.original_runs,
        "lead-controls/": args.original_controls,
        "microprobe/": args.original_microprobe,
    }
    supplied = [path is not None for path in sources.values()]
    if any(supplied) and not all(supplied):
        parser.error("supply all three original directories or none")
    if all(supplied) and not args.original_user_profile:
        parser.error("--original-user-profile is required with original directories")

    retained = dict(entries("RETAINED.sha256"))
    original = dict(entries("ORIGINALS.sha256"))
    problems = []
    if retained.keys() != original.keys():
        problems.append("retained and original manifests list different files")
    for relative, digest in retained.items():
        stored = (HERE / relative).read_bytes()
        if hashlib.sha256(stored).hexdigest() != digest:
            problems.append(f"retained hash mismatch: {relative}")
        if all(supplied):
            prefix = next((prefix for prefix in sources if relative.startswith(prefix)), None)
            if prefix is None:
                problems.append(f"unmapped original: {relative}")
                continue
            raw = (sources[prefix] / relative[len(prefix):]).read_bytes()
            if hashlib.sha256(raw).hexdigest() != original[relative]:
                problems.append(f"original hash mismatch: {relative}")
            if sanitise(raw, args.original_user_profile) != stored:
                problems.append(f"sanitised bytes differ: {relative}")
    for problem in problems:
        print(problem)
    print(f"{len(retained)} retained files checked, {len(problems)} problems"
          + ("; originals compared" if all(supplied) else "; original hashes need source files"))
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
