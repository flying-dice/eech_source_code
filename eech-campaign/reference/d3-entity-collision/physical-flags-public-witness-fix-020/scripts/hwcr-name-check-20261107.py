"""Collision check of the witness-correction batch (family `hwcr`): FAILS if ANY file or directory this batch is about to write already exists, run before the real PRE (the Root supervision note of 20:56 on the `hwfix` collision).
Only this batch's declared mutable developer artifacts (the flags-witfix-d* run records, the hwcr-dry-* namespace, the retained-runs, feedback, preflight, recovery and reader files it has already written) are exempt, and they are listed.
Usage: name-check.py <repo-root> [out.json] (the output must not exist)"""
import json
import pathlib
import sys

EXT = pathlib.Path("R:/d3-evidence")
REPO = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None
assert out is None or not out.exists(), out
intended = [EXT / n for n in (
    "hwcr-PRE-utc-20261107.txt", "hwcr-POST-utc-20261107.txt", "hwcr-inputmap-PRE-20261107.json", "hwcr-inputmap-POST-20261107.json", "hwcr-extra-identity-PRE-20261107.json", "hwcr-extra-identity-POST-20261107.json",
    "hwcr-inputmaps-compare-20261107.json", "hwcr-readers-PRE-20261107.json", "hwcr-final-set-20261107.progress", "hwcr-final-set-lease-before-20261107.txt", "hwcr-final-set-lease-after-20261107.txt",
    "hwcr-freeze-attempt1-20261107.stdout.txt", "hwcr-freeze-attempt1-20261107.stderr.txt", "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-FIX-CODE-FROZEN-20261107.json", "hwcr-freeze-attempt2-20261107.stdout.txt", "hwcr-freeze-attempt2-20261107.stderr.txt")]
for label in ("f1-fmt-check", "f2-clippy-world-engine-dc", "f3-witness-driver", "f4-world-all", "f5-engine-sys-lua-guards"):
    intended += [EXT / ("flags-witfix-%s.%s" % (label, suffix)) for suffix in ("json", "stdout.log", "stderr.log")]
intended.append(REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-public-witness-fix-020")
existing = [str(p) for p in intended if p.exists()]
# the readers of the family: they must exist, and none may carry the name of an older family's script
readers = ["run-physical-flags-witfix-direct-20261107.py", "check-physical-flags-witfix-linux-20261107.sh", "run-hwcr-final-set-20261107.sh", "hwcr-extra-identity-20261107.py", "freeze-hwcr-20261107.py", "publish-hwcr-20261107.py", "hwcr-name-check-20261107.py"]
missing = [r for r in readers if not (EXT / r).exists()]
result = {"intended_outputs_checked": len(intended), "already_existing": existing, "readers_missing": missing, "ok": not existing and not missing}
if out is not None:
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(result))
sys.exit(0 if result["ok"] else 1)
