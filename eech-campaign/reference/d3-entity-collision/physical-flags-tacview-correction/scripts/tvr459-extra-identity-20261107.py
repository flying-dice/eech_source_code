"""Identity of the files and tools the final checks of the WITNESS-RESIDUAL batch (family tvr459) read that the batched input map does not cover (it lists compiled and tested inputs only), and of the tools they ran with.
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
    "run-physical-flags-tvr459-direct-20261107.py",
    "check-physical-flags-tvr459-linux-20261107.sh",
    "run-tvr459-final-set-20261107.sh",
    "tvr459-extra-identity-20261107.py",
    "freeze-tvr459-20261107.py",
    "publish-tvr459-20261107.py",
    "tvr459-name-check-20261107.py",
    "sector-roster-inputmap-batched-20261106.py",
    "sho-compare-maps-20261107.py",
    "preflight-receipts-20261107.py",
    # this batch's declared records
    "tvr459-retained-dev-runs-20261107.json",
    "tvr459-feedback-files-20261107.json",
    "tvr459-name-check-result-20261107.json",
    "tvr459-name-check-negative-control-20261107.json",
    "tvr459-preflight-receipts-20261107.json",
    "tvr459-preflight-negative-control-20261107.json",
    # the sources of this batch's text that are not in Git (scratch blocks the Write tool wrote, and the explicit derivation of the helpers), and what this batch is answering
    "derive-tvr459-helpers-20261108.py",
    "tvr459-identity-join-20261107.py",
    "tvr459-identity-join-20261107.json",
    "tvb1-dormancy-evidence-20261107.json",
    "independent-tacview-disposition-8ffd-source-20261108.json",
    "Root-tacview-correction-feedback-20261108-0008.json",
    "Root-tacview-correction-feedback-20261108-0012.json",
    "tacview-disposition-8ffd-Root-binding-20261108.json",
    "tacview-disposition-459653-Root-publication-20261108.json",
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
