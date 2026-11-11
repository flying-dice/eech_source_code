"""Derives the helper files of the supply-descriptor BOOLEAN-CORRECTION batch (family sb120) from the previous batch's helpers (family sdc5a), as byte copies with ONE replacement, the family token `sdc5a` -> `sb120`
(the file names and contents of the sdc5a helpers carry no other family token that this replacement could touch: the one `supplyc5a` in the sdc5a publisher does not contain `sdc5a`). The batch-specific content is then edited
by hand with the Edit tool and the result inspected. Every target is created EXCLUSIVELY: the script stops before writing anything if any target exists. Usage: derive.py (no arguments); prints each copy with its source
and target SHA-256 and the number of replacements."""
import hashlib
import pathlib
import sys

E = pathlib.Path("R:/d3-evidence")
PAIRS = [
    ("run-physical-flags-sdc5a-direct-20261107.py", "run-physical-flags-sb120-direct-20261107.py"),
    ("check-physical-flags-sdc5a-linux-20261107.sh", "check-physical-flags-sb120-linux-20261107.sh"),
    ("run-sdc5a-final-set-20261107.sh", "run-sb120-final-set-20261107.sh"),
    ("sdc5a-extra-identity-20261107.py", "sb120-extra-identity-20261107.py"),
    ("freeze-sdc5a-20261107.py", "freeze-sb120-20261107.py"),
    ("publish-sdc5a-20261107.py", "publish-sb120-20261107.py"),
    ("sdc5a-name-check-20261107.py", "sb120-name-check-20261107.py"),
]
existing = [target for _, target in PAIRS if (E / target).exists()]
if existing:
    sys.exit("refusing: a target already exists: " + ", ".join(existing))
for source, target in PAIRS:
    data = (E / source).read_bytes()
    out = data.replace(b"sdc5a", b"sb120")
    with open(E / target, "xb") as handle:
        handle.write(out)
    print(source, hashlib.sha256(data).hexdigest()[:16], "->", target, hashlib.sha256(out).hexdigest()[:16], "replacements:", data.count(b"sdc5a"))
