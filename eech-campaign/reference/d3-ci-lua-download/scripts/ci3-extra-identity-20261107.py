"""Identity of the files and tools the final downloader checks read, which the batched input map does not cover (it lists compiled and tested inputs only), and of the shell tools they ran with."""
import hashlib
import json
import pathlib
import subprocess
import sys

repo = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
assert not out.exists(), out
files = [
    ".github/workflows/eech-campaign.yml",
    "eech-campaign/tools/fetch-pinned-archive.sh",
    "eech-campaign/tools/fetch-pinned-archive-selftest.sh",
    "eech-campaign/tools/Dockerfile",
]
external = [
    "R:/d3-evidence/lua-5.1.5-MacPorts-verified-20261107.tar.gz",
    "R:/d3-evidence/lua-5.1.5-MacPorts-verification-20261107.json",
    "R:/d3-evidence/verify-lua-5.1.5-fallback-archive-20261107.py",
    "R:/d3-evidence/sector-height-publication-1b89-CI-initial-download-failure-20261107.json",
    "R:/d3-evidence/sector-height-publication-1b89-CI-retry-download-failure-20261107.json",
    "R:/d3-evidence/independent-sector-height-proofguards-e326-source-20261107.json",
    "R:/d3-evidence/run-ci3-final-set-20261107.sh",
    "R:/d3-evidence/ci3-mutation-20261107.sh",
    "R:/d3-evidence/ci3-real-curl-options-20261107.sh",
    "R:/d3-evidence/ci3-workflow-static-20261107.py",
    "R:/d3-evidence/ci3-join-20261107.py",
    "R:/d3-evidence/ci3-safe-summary-20261107.py",
    "R:/d3-evidence/ci3-extra-identity-20261107.py",
    "R:/d3-evidence/freeze-ci-lua-20261107.py",
    "R:/d3-evidence/sector-roster-inputmap-batched-20261106.py",
    "R:/d3-evidence/sho-compare-maps-20261107.py",
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
        result = subprocess.run(list(cmd), capture_output=True, text=True, timeout=30)
        return (result.stdout or result.stderr).splitlines()[0]
    except Exception as error:  # a missing tool is recorded, never invented
        return "UNAVAILABLE: %s" % error


tools = {name: first_line(*cmd) for name, cmd in (("bash", ["bash", "--version"]), ("sha256sum", ["sha256sum", "--version"]), ("curl_real_only_in_c7", ["curl", "--version"]), ("git", ["git", "--version"]),
                                                  ("python", [sys.executable, "--version"]), ("awk", ["awk", "--version"]))}
agg = hashlib.sha256("".join("%s %s\n" % (r["path"], r["sha256"]) for r in rows).encode()).hexdigest()
out.write_text(json.dumps({"kind": "files and tool identity read by the final downloader checks (outside the batched input map)", "files": rows, "tools": tools, "aggregate_sha256": agg}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(agg)
