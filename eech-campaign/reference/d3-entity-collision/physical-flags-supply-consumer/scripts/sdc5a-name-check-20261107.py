"""Collision check of the supply-descriptor batch (family `sdc5a`): FAILS if ANY file or directory this batch is about to write already exists, run before the real PRE. It also refuses if any intended name carries the prefix of an older
family (hwfix, hwcr, hwwit, hwprod, hwload, hwkey, pw3dcb). Declared mutable developer artifacts of THIS batch (the flags-sdc5a-d* run records, the sdc5a-dry- namespace, its scratch files, the files it already wrote) are not intended outputs of the real
freeze and are not listed. Usage: name-check.py <repo-root> [out.json] (the output is created EXCLUSIVELY)"""
import json
import pathlib
import sys

EXT = pathlib.Path("R:/d3-evidence")
REPO = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
assert out is None or not out.exists(), out
intended = [EXT / n for n in (
    "sdc5a-PRE-utc-20261107.txt", "sdc5a-POST-utc-20261107.txt", "sdc5a-inputmap-PRE-20261107.json", "sdc5a-inputmap-POST-20261107.json", "sdc5a-extra-identity-PRE-20261107.json", "sdc5a-extra-identity-POST-20261107.json",
    "sdc5a-inputmaps-compare-20261107.json", "sdc5a-readers-PRE-20261107.json", "sdc5a-final-set-20261107.progress", "sdc5a-final-set-lease-before-20261107.txt", "sdc5a-final-set-lease-after-20261107.txt",
    "sdc5a-freeze-attempt1-20261107.stdout.txt", "sdc5a-freeze-attempt1-20261107.stderr.txt", "sdc5a-freeze-attempt2-20261107.stdout.txt", "sdc5a-freeze-attempt2-20261107.stderr.txt",
    "D3-PHYSICAL-FLAGS-SUPPLY-DESCRIPTOR-CODE-FROZEN-20261107.json")]
for label in ("s1-fmt-check", "s2-clippy-engine-hosted", "s3-clippy-engine-original", "s4-clippy-defaults", "s5-supply-hosted", "s6-supply-original", "s7-flags-guard", "s8-flags-controls-hosted", "s9-flags-controls-original", "s10-engine-default", "s11-build-dc"):
    intended += [EXT / ("flags-sdc5a-%s.%s" % (label, suffix)) for suffix in ("json", "stdout.log", "stderr.log")]
intended.append(REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-supply-consumer")
OLD = ("hwfix", "hwcr", "hwwit", "hwprod", "hwload", "hwkey", "pw3dcb")
old_prefixed = [str(p) for p in intended if any(p.name.startswith(o) or ("-" + o) in p.name for o in OLD)]
existing = [str(p) for p in intended if p.exists()]
readers = ["run-physical-flags-sdc5a-direct-20261107.py", "check-physical-flags-sdc5a-linux-20261107.sh", "run-sdc5a-final-set-20261107.sh", "sdc5a-extra-identity-20261107.py", "freeze-sdc5a-20261107.py", "publish-sdc5a-20261107.py", "sdc5a-name-check-20261107.py"]
missing = [r for r in readers if not (EXT / r).exists()]
result = {"intended_outputs_checked": len(intended), "already_existing": existing, "carrying_an_older_family_prefix": old_prefixed, "readers_missing": missing, "ok": not existing and not old_prefixed and not missing}
if out is not None:
    with open(out, "xb") as handle:
        handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps(result))
sys.exit(0 if result["ok"] else 1)
