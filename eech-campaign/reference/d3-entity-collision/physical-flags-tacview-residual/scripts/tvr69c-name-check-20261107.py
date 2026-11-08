"""Collision check of the supply-descriptor batch (family `tvr69c`): FAILS if ANY file or directory this batch is about to write already exists, run before the real PRE. It also refuses if any intended name carries the prefix of an older
family (hwfix, hwcr, hwwit, hwprod, hwload, hwkey, pw3dcb). Declared mutable developer artifacts of THIS batch (the flags-tvr69c-d* run records, the tvr69c-dry- namespace, its scratch files, the files it already wrote) are not intended outputs of the real
freeze and are not listed. Usage: name-check.py <repo-root> [out.json] (the output is created EXCLUSIVELY)"""
import json
import pathlib
import sys

EXT = pathlib.Path("R:/d3-evidence")
REPO = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
assert out is None or not out.exists(), out
intended = [EXT / n for n in (
    "tvr69c-PRE-utc-20261107.txt", "tvr69c-POST-utc-20261107.txt", "tvr69c-inputmap-PRE-20261107.json", "tvr69c-inputmap-POST-20261107.json", "tvr69c-extra-identity-PRE-20261107.json", "tvr69c-extra-identity-POST-20261107.json",
    "tvr69c-inputmaps-compare-20261107.json", "tvr69c-readers-PRE-20261107.json", "tvr69c-final-set-20261107.progress", "tvr69c-final-set-lease-before-20261107.txt", "tvr69c-final-set-lease-after-20261107.txt",
    "tvr69c-freeze-attempt1-20261107.stdout.txt", "tvr69c-freeze-attempt1-20261107.stderr.txt", "tvr69c-freeze-attempt2-20261107.stdout.txt", "tvr69c-freeze-attempt2-20261107.stderr.txt",
    "D3-TACVIEW-GUARD-RESIDUAL-FROZEN-20261108.json")]
for label in ("s1-fmt-check", "s2-clippy-engine-sys", "s3-flags-guard"):
    intended += [EXT / ("flags-tvr69c-%s.%s" % (label, suffix)) for suffix in ("json", "stdout.log", "stderr.log")]
intended.append(REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-tacview-residual")
OLD = ("hwfix", "hwcr", "hwwit", "hwprod", "hwload", "hwkey", "pw3dcb", "sdc5a", "sb120", "tvb1", "tvr459")
old_prefixed = [str(p) for p in intended if any(p.name.startswith(o) or ("-" + o) in p.name for o in OLD)]
existing = [str(p) for p in intended if p.exists()]
readers = ["run-physical-flags-tvr69c-direct-20261107.py", "check-physical-flags-tvr69c-linux-20261107.sh", "run-tvr69c-final-set-20261107.sh", "tvr69c-extra-identity-20261107.py", "freeze-tvr69c-20261107.py", "publish-tvr69c-20261107.py", "tvr69c-name-check-20261107.py"]
missing = [r for r in readers if not (EXT / r).exists()]
result = {"intended_outputs_checked": len(intended), "already_existing": existing, "carrying_an_older_family_prefix": old_prefixed, "readers_missing": missing, "ok": not existing and not old_prefixed and not missing}
if out is not None:
    with open(out, "xb") as handle:
        handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps(result))
sys.exit(0 if result["ok"] else 1)
