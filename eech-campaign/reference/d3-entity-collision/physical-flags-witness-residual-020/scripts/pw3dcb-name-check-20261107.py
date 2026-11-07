"""Collision check of the witness-residual batch (family `pw3dcb`): FAILS if ANY file or directory this batch is about to write already exists, run before the real PRE. It also refuses if any intended name carries the prefix of an older
family (hwfix, hwcr, hwwit, hwprod, hwload, hwkey). Declared mutable developer artifacts of THIS batch (the flags-pw3dcb-d* run records, the pw3dcb-dry- namespace, its scratch files, the files it already wrote) are not intended outputs of the real
freeze and are not listed. Usage: name-check.py <repo-root> [out.json] (the output is created EXCLUSIVELY)"""
import json
import pathlib
import sys

EXT = pathlib.Path("R:/d3-evidence")
REPO = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
assert out is None or not out.exists(), out
intended = [EXT / n for n in (
    "pw3dcb-PRE-utc-20261107.txt", "pw3dcb-POST-utc-20261107.txt", "pw3dcb-inputmap-PRE-20261107.json", "pw3dcb-inputmap-POST-20261107.json", "pw3dcb-extra-identity-PRE-20261107.json", "pw3dcb-extra-identity-POST-20261107.json",
    "pw3dcb-inputmaps-compare-20261107.json", "pw3dcb-readers-PRE-20261107.json", "pw3dcb-final-set-20261107.progress", "pw3dcb-final-set-lease-before-20261107.txt", "pw3dcb-final-set-lease-after-20261107.txt",
    "pw3dcb-freeze-attempt1-20261107.stdout.txt", "pw3dcb-freeze-attempt1-20261107.stderr.txt", "pw3dcb-freeze-attempt2-20261107.stdout.txt", "pw3dcb-freeze-attempt2-20261107.stderr.txt",
    "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-RESIDUAL-CODE-FROZEN-20261107.json")]
for label in ("f1-fmt-check", "f2-clippy-world-engine-dc", "f3-witness-driver", "f4-world-all", "f5-engine-sys-lua-guards"):
    intended += [EXT / ("flags-pw3dcb-%s.%s" % (label, suffix)) for suffix in ("json", "stdout.log", "stderr.log")]
intended.append(REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-witness-residual-020")
OLD = ("hwfix", "hwcr", "hwwit", "hwprod", "hwload", "hwkey")
old_prefixed = [str(p) for p in intended if any(p.name.startswith(o) or ("-" + o) in p.name for o in OLD)]
existing = [str(p) for p in intended if p.exists()]
readers = ["run-physical-flags-pw3dcb-direct-20261107.py", "check-physical-flags-pw3dcb-linux-20261107.sh", "run-pw3dcb-final-set-20261107.sh", "pw3dcb-extra-identity-20261107.py", "freeze-pw3dcb-20261107.py", "publish-pw3dcb-20261107.py", "pw3dcb-name-check-20261107.py"]
missing = [r for r in readers if not (EXT / r).exists()]
result = {"intended_outputs_checked": len(intended), "already_existing": existing, "carrying_an_older_family_prefix": old_prefixed, "readers_missing": missing, "ok": not existing and not old_prefixed and not missing}
if out is not None:
    with open(out, "xb") as handle:
        handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps(result))
sys.exit(0 if result["ok"] else 1)
