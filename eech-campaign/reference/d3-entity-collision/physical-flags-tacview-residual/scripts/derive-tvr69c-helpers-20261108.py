"""Derives the helper files of the Tacview-guard RESIDUAL batch (family tvr69c) from the previous batch's helpers (family tvr459), as byte copies with ONE replacement, the family token `tvr459` -> `tvr69c`.
Nothing else is replaced (the shared helpers and retained records the scripts name keep their own file names and dates); the derived scripts are then edited by hand with the Edit and Write tools and the result
inspected. Every target is created EXCLUSIVELY: the script stops before writing anything if any target exists.
Usage: derive.py (no arguments); prints each copy with its source and target SHA-256 and the number of replacements."""
import hashlib
import pathlib
import sys

E = pathlib.Path("R:/d3-evidence")
PAIRS = [
    ("run-physical-flags-tvr459-direct-20261107.py", "run-physical-flags-tvr69c-direct-20261107.py"),
    ("check-physical-flags-tvr459-linux-20261107.sh", "check-physical-flags-tvr69c-linux-20261107.sh"),
    ("run-tvr459-final-set-20261107.sh", "run-tvr69c-final-set-20261107.sh"),
    ("tvr459-extra-identity-20261107.py", "tvr69c-extra-identity-20261107.py"),
    ("freeze-tvr459-20261107.py", "freeze-tvr69c-20261107.py"),
    ("publish-tvr459-20261107.py", "publish-tvr69c-20261107.py"),
    ("tvr459-name-check-20261107.py", "tvr69c-name-check-20261107.py"),
]
existing = [target for _, target in PAIRS if (E / target).exists()]
if existing:
    sys.exit("refusing: a target already exists: " + ", ".join(existing))
for source, target in PAIRS:
    data = (E / source).read_bytes()
    out = data.replace(b"tvr459", b"tvr69c")
    with open(E / target, "xb") as handle:
        handle.write(out)
    print(source, hashlib.sha256(data).hexdigest()[:16], "->", target, hashlib.sha256(out).hexdigest()[:16], "replacements:", data.count(b"tvr459"))
