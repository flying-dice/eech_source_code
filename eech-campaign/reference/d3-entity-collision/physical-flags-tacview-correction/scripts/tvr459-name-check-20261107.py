"""Collision check of the supply-descriptor batch (family `tvr459`): FAILS if ANY file or directory this batch is about to write already exists, run before the real PRE. It also refuses if any intended name carries the prefix of an older
family (hwfix, hwcr, hwwit, hwprod, hwload, hwkey, pw3dcb). Declared mutable developer artifacts of THIS batch (the flags-tvr459-d* run records, the tvr459-dry- namespace, its scratch files, the files it already wrote) are not intended outputs of the real
freeze and are not listed. Usage: name-check.py <repo-root> [out.json] (the output is created EXCLUSIVELY)"""
import json
import pathlib
import sys

EXT = pathlib.Path("R:/d3-evidence")
REPO = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
assert out is None or not out.exists(), out
intended = [EXT / n for n in (
    "tvr459-PRE-utc-20261107.txt", "tvr459-POST-utc-20261107.txt", "tvr459-inputmap-PRE-20261107.json", "tvr459-inputmap-POST-20261107.json", "tvr459-extra-identity-PRE-20261107.json", "tvr459-extra-identity-POST-20261107.json",
    "tvr459-inputmaps-compare-20261107.json", "tvr459-readers-PRE-20261107.json", "tvr459-final-set-20261107.progress", "tvr459-final-set-lease-before-20261107.txt", "tvr459-final-set-lease-after-20261107.txt",
    "tvr459-freeze-attempt1-20261107.stdout.txt", "tvr459-freeze-attempt1-20261107.stderr.txt", "tvr459-freeze-attempt2-20261107.stdout.txt", "tvr459-freeze-attempt2-20261107.stderr.txt",
    "D3-TACVIEW-DISPOSITION-CORRECTION-FROZEN-20261107.json")]
for label in ("s1-fmt-check", "s2-clippy-engine-sys", "s3-flags-guard"):
    intended += [EXT / ("flags-tvr459-%s.%s" % (label, suffix)) for suffix in ("json", "stdout.log", "stderr.log")]
intended.append(REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-tacview-correction")
OLD = ("hwfix", "hwcr", "hwwit", "hwprod", "hwload", "hwkey", "pw3dcb", "sdc5a", "sb120", "tvb1")
old_prefixed = [str(p) for p in intended if any(p.name.startswith(o) or ("-" + o) in p.name for o in OLD)]
existing = [str(p) for p in intended if p.exists()]
readers = ["run-physical-flags-tvr459-direct-20261107.py", "check-physical-flags-tvr459-linux-20261107.sh", "run-tvr459-final-set-20261107.sh", "tvr459-extra-identity-20261107.py", "freeze-tvr459-20261107.py", "publish-tvr459-20261107.py", "tvr459-name-check-20261107.py"]
missing = [r for r in readers if not (EXT / r).exists()]
result = {"intended_outputs_checked": len(intended), "already_existing": existing, "carrying_an_older_family_prefix": old_prefixed, "readers_missing": missing, "ok": not existing and not old_prefixed and not missing}
if out is not None:
    with open(out, "xb") as handle:
        handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps(result))
sys.exit(0 if result["ok"] else 1)
