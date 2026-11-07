"""Identity of the files and tools the final checks of the WITNESS-RESIDUAL batch (family pw3dcb) read that the batched input map does not cover (it lists compiled and tested inputs only), and of the tools they ran with.
Usage: extra.py <repo> <out.json> (the output is created EXCLUSIVELY: it must not exist)"""
import hashlib
import json
import pathlib
import subprocess
import sys

repo = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
assert not out.exists(), out
files = [
    "eech-campaign/docs/d3-physical-flags-contract.md",
    "eech-campaign/docs/d3-sector-height-production-contract.md",
    "eech-campaign/tools/build-windows-sector-height-witness.sh",
    "eech-campaign/tools/build-windows-collision-fixture.sh",
    "eech-campaign/tools/build-windows.sh",
    "eech-campaign/tools/migration-progress.py",
    "eech-campaign/rustfmt.toml",
    "docs/governance/D3-FIVE-WHYS-20261102.md",
]
E = "R:/d3-evidence/"
external = [E + name for name in (
    # the readers of THIS batch
    "run-physical-flags-pw3dcb-direct-20261107.py",
    "check-physical-flags-pw3dcb-linux-20261107.sh",
    "run-pw3dcb-final-set-20261107.sh",
    "pw3dcb-extra-identity-20261107.py",
    "freeze-pw3dcb-20261107.py",
    "publish-pw3dcb-20261107.py",
    "pw3dcb-name-check-20261107.py",
    "sector-roster-inputmap-batched-20261106.py",
    "sho-compare-maps-20261107.py",
    "preflight-receipts-20261107.py",
    # this batch's declared records
    "pw3dcb-retained-dev-runs-20261107.json",
    "pw3dcb-feedback-files-20261107.json",
    "pw3dcb-name-check-result-20261107.json",
    "pw3dcb-name-check-negative-control-20261107.json",
    "pw3dcb-preflight-receipts-20261107.json",
    "pw3dcb-preflight-negative-control-20261107.json",
    # the report and Root's records this batch answers, and the immediate predecessor's freeze
    "independent-physical-flags-public-witness-3dcb-source-20261107.json",
    "physical-flags-witfix-3dcb-Root-binding-20261107.json",
    "physical-flags-witfix-6fce-publication-Root-20261107.json",
    "hwcr-namespace-recovery-Root-binding-20261107.json",
    "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-FIX-CODE-FROZEN-20261107.json",
    "Root-physical-flags-witness-feedback-20261107-2012.json",
    "Root-physical-flags-witness-fix-feedback-20261107-2056.json",
    "physical-flags-native-windows-020-20261107-completion.json",
    "physical-flags-020-WINDOWS-CACHE-RELEASE-20261107.json",
)]
rows = []
for f in files:
    data = (repo / f).read_bytes()
    rows.append({"path": f, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_hash_object": subprocess.check_output(["git", "-C", str(repo), "hash-object", f], text=True).strip()})
for f in external:
    data = pathlib.Path(f).read_bytes()
    rows.append({"path": f, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_hash_object": None})


def first_line(*cmd):
    try:
        result = subprocess.run(list(cmd), capture_output=True, text=True, timeout=60)
        return (result.stdout or result.stderr).splitlines()[0]
    except Exception as error:  # a missing tool is recorded, never invented
        return "UNAVAILABLE: %s" % error


wsl = ["wsl", "-d", "EECH-D2-Build", "--cd", "/", "--"]
image = "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3"
tools = {
    "image_id": first_line(*wsl, "docker", "image", "inspect", "--format", "{{.Id}}", image),
    "docker": first_line(*wsl, "docker", "--version"),
    "python": first_line(sys.executable, "--version"),
    "git": first_line("git", "--version"),
    "volume": "eech-linux-d3-physical-flags-9d247-20261107",
    "image_repo_tags": first_line(*wsl, "docker", "image", "inspect", "--format", "{{.RepoTags}}", image),
}
agg = hashlib.sha256("".join("%s %s\n" % (r["path"], r["sha256"]) for r in rows).encode()).hexdigest()
with open(out, "xb") as handle:
    handle.write((json.dumps({"kind": "files and tool identity read by the final checks of the witness-residual batch (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n").encode("utf-8"))
print(agg)
