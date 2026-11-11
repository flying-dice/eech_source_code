"""Extracts the compiled, build and retained-runtime facts of the Tacview/MOVED-ROTATED consumer disposition (family tvb1) from the READ-ONLY retained Windows profile reports and the closed default roots of the 3abf
candidate, as literal excerpts with the identity of every file read. Nothing is copied wholesale (the nm reports are 8 MB) and nothing under any running job's root is touched. The output is created EXCLUSIVELY.
Usage: extract.py <repo-root> <out.json>"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
OUT = pathlib.Path(sys.argv[2])
assert not OUT.exists(), OUT
PROFILE = pathlib.Path("D:/EECH-Migration-Evidence/supply-boolean-native-3abf-profile-20261107")
ROOTS = pathlib.Path("D:/EECH-Migration-Evidence/supply-boolean-default-3abf-roots-20261107")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, check=True).stdout.decode("utf-8", "replace")


def short(line):
    return re.sub(r"^.*libeech_engine\.a:", "", line.rstrip())


result = {
    "kind": "retained compiled and runtime facts for the Tacview/MOVED-ROTATED consumer disposition (read-only; literal excerpts)",
    "repo_head": git("rev-parse", "HEAD").strip(),
    "built_source": "3abf03e51 (the snapshot D:/EECH-Migration-Evidence/supply-boolean-3abf-20261107-source)",
    "product_and_crate_source_changed_since_3abf": subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "3abf03e51", "HEAD", "--", "aphavoc", "eech-campaign/crates"]).returncode != 0,
    "profiles": {},
    "closed_default_roots": {},
}
patterns = {
    "start_tacview_logging (definition T and every reference U)": re.compile(r" [UT] start_tacview_logging$"),
    "tacview_log_file definition": re.compile(r" [BDC] tacview_log_file$"),
    "tacview.o defined functions": re.compile(r"-tacview\.o:\S* T "),
    "flight (definition and references)": re.compile(r" [UT] flight$"),
    "write_tacview_unit_update (definition and references)": re.compile(r" [UT] write_tacview_unit_update$"),
    "references to assign_entity_to_user": re.compile(r" [UT] assign_entity_to_user$"),
    "references to tacview_reset_frame": re.compile(r" [UT] tacview_reset_frame$"),
}
for profile in ("ordinary", "original"):
    report = PROFILE / profile / "engine-archive-nm.txt"
    meta = json.loads((PROFILE / profile / "profile.json").read_text(encoding="utf-8"))
    lines = report.read_text(encoding="utf-8", errors="replace").splitlines()
    excerpt = {name: [short(l) for l in lines if rx.search(l)] for name, rx in patterns.items()}
    objects_using_file = sorted({short(l).split(":")[0] for l in lines if re.search(r" U tacview_log_file$", l)})
    engine_o_tacview = [short(l) for l in lines if re.search(r"eech_engine\.o:", l) and "tacview" in l]
    result["profiles"][profile] = {
        "report": str(report), "report_sha256": sha(report), "archive_sha256": meta["archive_sha256"], "features": meta["features"],
        "excerpts": excerpt, "objects_referencing_tacview_log_file": objects_using_file, "eech_engine_o_lines_naming_tacview": engine_o_tacview,
    }
for root in ("ordinary_short", "original_short"):
    cohokum = ROOTS / root / "cohokum"
    ini = cohokum / "EECH.INI"
    row = {"cohokum": str(cohokum), "tacview_logs_directory_exists": (cohokum / "tacview-logs").exists(), "txt_acmi_files_under_root": [str(p.relative_to(ROOTS)) for p in (ROOTS / root).rglob("*.txt.acmi")][:5]}
    if ini.exists():
        text = ini.read_text(encoding="latin-1")
        row["EECH.INI_sha256"] = sha(ini)
        row["EECH.INI_tacview_lines"] = [l.split("#")[0].strip() for l in text.splitlines() if l.lower().startswith("tacview")]
        row["EECH.INI_dump_ini_lines"] = [l.split("#")[0].strip() for l in text.splitlines() if "dump_ini" in l.lower()]
    result["closed_default_roots"][root] = row
grep = git("grep", "-n", "-E", r"self\.out\.(update|update_attitude|event|remove)\(", "--", "eech-campaign/crates/eech-world/src/main.rs").splitlines()
result["rust_recorder_calls_at_head"] = {"file": "eech-campaign/crates/eech-world/src/main.rs", "git_blob": git("rev-parse", "HEAD:eech-campaign/crates/eech-world/src/main.rs").strip(), "lines": [l[:200] for l in grep]}
with open(OUT, "xb") as handle:
    handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps({k: v for k, v in result.items() if k in ("repo_head", "product_and_crate_source_changed_since_3abf")}))
for profile, data in result["profiles"].items():
    print(profile, {k: len(v) for k, v in data["excerpts"].items()}, len(data["objects_referencing_tacview_log_file"]), data["eech_engine_o_lines_naming_tacview"])
print({k: (v["tacview_logs_directory_exists"], v.get("EECH.INI_tacview_lines"), v.get("EECH.INI_dump_ini_lines"), v["txt_acmi_files_under_root"]) for k, v in result["closed_default_roots"].items()})
print(len(grep), "recorder call lines")
