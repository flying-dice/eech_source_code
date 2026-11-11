"""Derives the helper files of the supply-descriptor batch (family sdc5a) from the previous batch's helpers (family pw3dcb), as byte copies with ONE replacement, the family token `pw3dcb` -> `sdc5a` (the previous family's own
file names carry no other family token; the batch-specific content is then edited by hand with the Edit tool and the result inspected). Every target is created EXCLUSIVELY: the script stops before writing anything if any target exists.
Usage: derive.py (no arguments); prints each copy with its source and target SHA-256."""
import hashlib
import pathlib
import sys

E = pathlib.Path("R:/d3-evidence")
PAIRS = [
    ("run-pw3dcb-final-set-20261107.sh", "run-sdc5a-final-set-20261107.sh"),
    ("pw3dcb-extra-identity-20261107.py", "sdc5a-extra-identity-20261107.py"),
    ("freeze-pw3dcb-20261107.py", "freeze-sdc5a-20261107.py"),
    ("publish-pw3dcb-20261107.py", "publish-sdc5a-20261107.py"),
    ("pw3dcb-name-check-20261107.py", "sdc5a-name-check-20261107.py"),
]
existing = [target for _, target in PAIRS if (E / target).exists()]
if existing:
    sys.exit("refusing: a target already exists: " + ", ".join(existing))
for source, target in PAIRS:
    data = (E / source).read_bytes()
    out = data.replace(b"pw3dcb", b"sdc5a")
    with open(E / target, "xb") as handle:
        handle.write(out)
    print(source, hashlib.sha256(data).hexdigest()[:16], "->", target, hashlib.sha256(out).hexdigest()[:16], "replacements:", data.count(b"pw3dcb"))
