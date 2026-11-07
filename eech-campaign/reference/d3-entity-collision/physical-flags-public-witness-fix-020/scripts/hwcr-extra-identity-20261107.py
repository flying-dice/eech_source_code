"""Identity of the files and tools the final checks of the WITNESS-FIX batch read that the batched input map does not cover (it lists compiled and tested inputs only), and of the tools they ran with.
Usage: extra.py <repo> <out.json> (the output must not exist)"""
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
    "run-physical-flags-witfix-direct-20261107.py",
    "check-physical-flags-witfix-linux-20261107.sh",
    "run-hwcr-final-set-20261107.sh",
    "hwcr-extra-identity-20261107.py",
    "freeze-hwcr-20261107.py",
    "publish-hwcr-20261107.py",
    "sector-roster-inputmap-batched-20261106.py",
    "sho-compare-maps-20261107.py",
    "preflight-receipts-20261107.py",
    # Root's literal records this batch binds (the cache release, the 020 build and profile receipts, the default-plan actuals, the publication checkpoint, the learnings and the supervision note)
    "physical-flags-020-WINDOWS-CACHE-RELEASE-20261107.json",
    "independent-physical-flags-public-witness-f4d-source-20261107-v2.json",
    "independent-physical-flags-public-witness-f4d-source-20261107.json",
    "physical-flags-witness-f4d-Root-binding-20261107.json",
    "physical-flags-witness-c5fe-publication-Root-20261107.json",
    "physical-flags-020-closed-Windows-cache-cleanup-20261107.json",
    "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-CODE-FROZEN-20261107.json",
    "physical-flags-native-source-snapshot-020-20261107.json",
    "physical-flags-native-windows-020-20261107-completion.json",
    "physical-flags-native-020-profile-20261107-completion.json",
    "physical-flags-default-020-plan-20261107.json",
    "independent-physical-flags-020-default-plan-20261107.json",
    "physical-flags-default-020-Root-binding-20261107.json",
    "physical-flags-default-020-ordinary_short-20261107-completion.json",
    "physical-flags-default-020-original_short-20261107-completion.json",
    "physical-flags-3e74-publication-1e31-Root-20261107.json",
    "physical-flags-3e74-publication-1e31-all9-CI-readback-20261107.json",
    "Root-publication-and-image-retention-learnings-20261107.json",
    "Root-physical-flags-witness-feedback-20261107-2012.json",
    "independent-physical-flags-keyfix-020-source-20261107.json",
    "hwcr-retained-dev-runs-20261107.json",
    "hwcr-feedback-files-20261107.json",
    "hwcr-name-collision-erratum-20261107.json",
    "hwcr-recovery-receipt-20261107.json",
    "hwcr-name-check-20261107.py",
    "hwcr-name-check-result-20261107.json",
    "hwcr-name-check-negative-control-20261107.json",
    "Root-physical-flags-witness-fix-feedback-20261107-2056.json",
    "hwcr-preflight-receipts-20261107.json",
    "hwcr-preflight-negative-control-20261107.json",
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
out.write_text(json.dumps({"kind": "files and tool identity read by the final checks of the public witness batch (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(agg)
