"""Safe summary of the two preserved CI failure records. Each external record embeds the whole raw job log (runner image provisioning, git fetch details, the failing step); the packet carries only identities and what the finding needs:
the jobs and their conclusions, the failed job's steps, and the three log lines of the failing step (the command, the curl error, the exit). The raw logs stay external. Usage: summary.py <out.json>"""
import hashlib
import json
import pathlib
import re
import sys

EXT = pathlib.Path("R:/d3-evidence")
ANSI = re.compile(r"\x1b\[[0-9;]*m")
RECORDS = {
    "initial": "sector-height-publication-1b89-CI-initial-download-failure-20261107.json",
    "retry": "sector-height-publication-1b89-CI-retry-download-failure-20261107.json",
}


def log_text(record):
    for key in ("raw_failed_job_log_tool_result", "raw_log_tool_result"):
        value = record.get(key)
        if isinstance(value, dict):
            structured = value.get("structuredContent")
            if isinstance(structured, dict) and isinstance(structured.get("content"), str):
                return structured["content"], key
    return None, None


out = {"kind": "safe summary of the preserved failure records; the raw job logs are not copied", "records": {}}
problems = []
for tag, name in RECORDS.items():
    path = EXT / name
    data = path.read_bytes()
    record = json.loads(data.decode("utf-8"))
    text, key = log_text(record)
    if text is None:
        problems.append("no embedded log in " + name)
        continue
    lines = [ANSI.sub("", l.lstrip("\ufeff")) for l in text.splitlines()]
    cuts = [l[l.index("Z ") + 2:] for l in lines if re.search(r"(##\[group\]Run curl |curl: \(\d+\)|##\[error\]Process completed with exit code)", l)]
    timestamps = [l[:l.index("Z ") + 1] for l in lines if re.search(r"(##\[group\]Run curl |curl: \(\d+\)|##\[error\]Process completed with exit code)", l)]
    jobs = record.get("jobs") or record.get("actual_retry_jobs") or []
    failed = [j for j in jobs if j.get("conclusion") == "failure"]
    if len(failed) != 1:
        problems.append("%s: not exactly one failed job" % name)
        continue
    exit_codes = re.findall(r"exit code (\d+)", " ".join(cuts))
    out["records"][tag] = {
        "file": str(path), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "embedded_log_field": key, "embedded_log_lines": len(lines),
        "source": record["source"], "workflow_run": record["workflow_run"], "recorded_reason": record["reason"],
        "jobs": [{"id": j["id"], "name": j["name"], "conclusion": j["conclusion"]} for j in jobs],
        "failed_job": {"id": failed[0]["id"], "name": failed[0]["name"], "steps": [{"number": s["number"], "name": s["name"], "conclusion": s["conclusion"]} for s in failed[0]["steps"]]},
        "failing_step_log_lines": [{"utc": t, "line": c} for t, c in zip(timestamps, cuts)],
        "process_exit_codes_in_log": exit_codes,
    }
    if exit_codes != ["28"] or not any("Failed to connect to www.lua.org port 443" in c for c in cuts):
        problems.append("%s: the log does not show the lua.org connect failure with exit 28: %s" % (name, cuts))
out["problems"] = problems
pathlib.Path(sys.argv[1]).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"records": list(out["records"]), "problems": problems}))
sys.exit(1 if problems else 0)
