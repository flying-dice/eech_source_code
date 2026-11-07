"""Identity of the files and tools the final checks read that the batched input map does not cover (it lists compiled and tested inputs only), and of the tools they ran with."""
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
    "eech-campaign/rustfmt.toml",
    "docs/governance/D3-FIVE-WHYS-20261102.md",
]
external = [
    "R:/d3-evidence/run-physical-flags-keyfix-direct-20261107.py",
    "R:/d3-evidence/run-physical-flags-keyfix-direct-20261107.before-root-supervision-20261107-185728.py",
    "R:/d3-evidence/check-physical-flags-keyfix-linux-20261107.sh",
    "R:/d3-evidence/run-hwkey-final-set-20261107.sh",
    "R:/d3-evidence/hwkey-join-20261107.py",
    "R:/d3-evidence/make-hwkey-join-20261107.py",
    "R:/d3-evidence/hwkey-extra-identity-20261107.py",
    "R:/d3-evidence/freeze-hwkey-20261107.py",
    "R:/d3-evidence/sector-roster-inputmap-batched-20261106.py",
    "R:/d3-evidence/sho-compare-maps-20261107.py",
    "R:/d3-evidence/preflight-receipts-20261107.py",
    "R:/d3-evidence/independent-physical-flags-production-0d0-source-20261107-v2.json",
    "R:/d3-evidence/independent-physical-flags-production-0d0-source-20261107.json",
    "R:/d3-evidence/physical-flags-production-0d0-Root-binding-20261107.json",
    "R:/d3-evidence/physical-flags-production-c126-publication-Root-20261107.json",
    "R:/d3-evidence/D3-PHYSICAL-FLAGS-PRODUCTION-CODE-FROZEN-20261107.json",
    "R:/d3-evidence/hwprod-inputmap-POST-20261107.json",
    "R:/d3-evidence/hwprod-inputmaps-compare-20261107.json",
    "R:/d3-evidence/Root-physical-flags-key-fix-feedback-20261107-1848.json",
    "R:/d3-evidence/Root-physical-flags-key-fix-feedback-20261107-1909.json",
    "R:/d3-evidence/Root-physical-flags-key-fix-feedback-20261107-1916.json",
    "R:/d3-evidence/Root-keyfix-supervision-IPC-20261107-185728.json",
    "R:/d3-evidence/Root-height-reference-reader-9d247-lease-20261107.json",
    "R:/d3-evidence/hwkey-retained-dev-runs-20261107.json",
    "R:/d3-evidence/hwkey-feedback-files-20261107.json",
    "R:/d3-evidence/keyfix-fixed-copies/mb_hostentity.h",
    "R:/d3-evidence/keyfix-fixed-copies/mb_hostmodel.c",
]
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
tools = {
    "image_id": first_line(*wsl, "docker", "image", "inspect", "--format", "{{.Id}}", "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3"),
    "docker": first_line(*wsl, "docker", "--version"),
    "python": first_line(sys.executable, "--version"),
    "git": first_line("git", "--version"),
    "volume": "eech-linux-d3-physical-flags-9d247-20261107",
    "image_repo_tags": first_line(*wsl, "docker", "image", "inspect", "--format", "{{.RepoTags}}", "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3"),
}
agg = hashlib.sha256("".join("%s %s\n" % (r["path"], r["sha256"]) for r in rows).encode()).hexdigest()
out.write_text(json.dumps({"kind": "files and tool identity read by the final checks of the key-fix batch (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(agg)
