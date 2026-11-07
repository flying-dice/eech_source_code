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
    "R:/d3-evidence/run-physical-flags-direct-20261107.py",
    "R:/d3-evidence/check-physical-flags-linux-20261107.sh",
    "R:/d3-evidence/run-hwflags-final-set-20261107.sh",
    "R:/d3-evidence/hwflags-join-20261107.py",
    "R:/d3-evidence/hwflags-extra-identity-20261107.py",
    "R:/d3-evidence/freeze-hwflags-20261107.py",
    "R:/d3-evidence/sector-roster-inputmap-batched-20261106.py",
    "R:/d3-evidence/sho-compare-maps-20261107.py",
    "R:/d3-evidence/independent-sector-height-M6-native-764-3c2-20261107.json",
    "R:/d3-evidence/height-core-load-native-764-Root-binding-20261107.json",
    "R:/d3-evidence/hwload-inputmap-POST-20261107.json",
    "R:/d3-evidence/Root-height-reference-reader-9d247-lease-20261107.json",
    "R:/d3-evidence/hwflags-retained-dev-runs-20261107.json",
    "R:/d3-evidence/hwflags-feedback-files-20261107.json",
    "R:/d3-evidence/Root-physical-flags-feedback-20261107-1550.json",
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
out.write_text(json.dumps({"kind": "files and tool identity read by the final checks of the physical-flags characterisation (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(agg)
