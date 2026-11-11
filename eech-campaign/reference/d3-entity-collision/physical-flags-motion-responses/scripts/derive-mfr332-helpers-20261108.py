"""Derives the helper files of the motion-response flag batch (family mfr332) from the previous batch's helpers (family tvr69c), as byte copies with ONE replacement, the family token `tvr69c` -> `mfr332`.
Nothing else is replaced: the file names keep the date stems the scripts' own internal references expect (the lesson of the tvr69c derivation, whose first outputs carried a stem that disagreed with its references), and
the shared helpers and retained records the scripts name keep their own names. The derived scripts are then edited by hand with the Edit and Write tools and the result inspected. Every target is created
EXCLUSIVELY: the script stops before writing anything if any target exists. Usage: derive.py (no arguments); prints each copy with its source and target SHA-256 and the number of replacements."""
import hashlib
import pathlib
import sys

E = pathlib.Path("R:/d3-evidence")
PAIRS = [
    ("run-physical-flags-tvr69c-direct-20261107.py", "run-physical-flags-mfr332-direct-20261107.py"),
    ("check-physical-flags-tvr69c-linux-20261107.sh", "check-physical-flags-mfr332-linux-20261107.sh"),
    ("run-tvr69c-final-set-20261107.sh", "run-mfr332-final-set-20261107.sh"),
    ("tvr69c-extra-identity-20261107.py", "mfr332-extra-identity-20261107.py"),
    ("freeze-tvr69c-20261107.py", "freeze-mfr332-20261107.py"),
    ("publish-tvr69c-20261107.py", "publish-mfr332-20261107.py"),
    ("tvr69c-name-check-20261107.py", "mfr332-name-check-20261107.py"),
]
existing = [target for _, target in PAIRS if (E / target).exists()]
if existing:
    sys.exit("refusing: a target already exists: " + ", ".join(existing))
for source, target in PAIRS:
    data = (E / source).read_bytes()
    out = data.replace(b"tvr69c", b"mfr332")
    with open(E / target, "xb") as handle:
        handle.write(out)
    print(source, hashlib.sha256(data).hexdigest()[:16], "->", target, hashlib.sha256(out).hexdigest()[:16], "replacements:", data.count(b"tvr69c"))
