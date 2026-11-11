"""Identity of the files and tools the final checks of the MOTION-RESPONSE FLAG GROUP batch (family mfr332) read that the batched input map does not cover (it lists compiled and tested inputs only), and of the tools they ran with.
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
    "run-physical-flags-mfr332-direct-20261107.py",
    "check-physical-flags-mfr332-linux-20261107.sh",
    "run-mfr332-final-set-20261107.sh",
    "mfr332-extra-identity-20261107.py",
    "freeze-mfr332-20261107.py",
    "publish-mfr332-20261107.py",
    "mfr332-name-check-20261107.py",
    "sector-roster-inputmap-batched-20261106.py",
    "sho-compare-maps-20261107.py",
    "preflight-receipts-20261107.py",
    # this batch's declared records
    "mfr332-retained-dev-runs-20261107.json",
    "mfr332-feedback-files-20261107.json",
    "mfr332-name-check-result-20261107.json",
    "mfr332-name-check-negative-control-20261107.json",
    "mfr332-preflight-receipts-20261107.json",
    "mfr332-preflight-negative-control-20261107.json",
    # the sources of this batch's text that are not in Git (scratch blocks the Write tool wrote, and the explicit derivation of the helpers), and what this batch is answering
    "derive-mfr332-helpers-20261108.py",
    "Root-motion-flags-feedback-20261108-0038.json",
    "Root-motion-flags-feedback-20261108-0044.json",
    "Root-motion-flags-feedback-20261108-0052.json",
    "mfr332-claude-auth-expired-checkpoint-20261108.json",
    "codex-d3-motion-flags-fallback-20261108-process.json",
    "tacview-closure-live-services-20261108.json",
    "ledger-tacview-closure-v190-preservation-20261108.json",
    "tacview-reviewed-332955-Root-publication-20261108.json",
    "tacview-332955-all9-CI-readback-20261108.json",
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
    handle.write((json.dumps({"kind": "files and tool identity read by the final checks of the motion-response flag group batch (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n").encode("utf-8"))
print(agg)
