#!/usr/bin/env python3
"""Verify the retained SUPPLY-arrival evidence and its cross-file identities."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPECTED = {
    "M4-BUILD-INFO.txt",
    "M4-TEST-INFO.txt",
    "EXACT-HEAD-BUILD-INFO.txt",
    "m4-pack.report.json",
    "m4-pack.report.md",
    "causal-reference.out",
    "causal-withhold-pick-up.out",
    "causal-delay-drop-off.out",
    "causal.summary.json",
    "exact-head-lifecycle.summary.json",
    "controls-puc.results.jsonl",
    "controls-puc.summary.txt",
    "controls-dcs.results.jsonl",
    "controls-dcs.summary.txt",
    "independent-release-source-equivalence.json",
}
SHA = re.compile(r"[0-9a-f]{64}")
ROW = re.compile(r"^\| \[([^]]+)]\(([^)]+)\) \| `([0-9a-f]{64})` \|$")
SECTION = "## Retained file hashes"
HEADER = "| File | SHA-256 |"
SEPARATOR = "|---|---|"


def fail(message: str) -> None:
    raise SystemExit(message)


def require_sha(value: object, label: str) -> str:
    if not isinstance(value, str) or not SHA.fullmatch(value):
        fail(f"invalid SHA-256 for {label}: {value!r}")
    return value


def retained_hash_rows(readme: str) -> list[tuple[str, str, str]]:
    lines = readme.splitlines()
    headings = [index for index, line in enumerate(lines) if line == SECTION]
    if len(headings) != 1:
        fail(f"expected exactly one {SECTION!r} section, found {len(headings)}")

    rows: list[tuple[str, str, str]] = []
    saw_header = False
    saw_separator = False
    for line in lines[headings[0] + 1 :]:
        if line.startswith("## "):
            break
        if not line:
            continue
        if line == HEADER:
            if saw_header or saw_separator:
                fail("malformed retained-hash table header")
            saw_header = True
            continue
        if line == SEPARATOR:
            if not saw_header or saw_separator:
                fail("malformed retained-hash table separator")
            saw_separator = True
            continue
        if not line.lstrip().startswith("|"):
            continue
        match = ROW.fullmatch(line)
        if not match:
            fail(f"malformed retained-hash row: {line}")
        if not saw_separator:
            fail(f"retained-hash row precedes table separator: {line}")
        rows.append(match.groups())
    if not saw_header or not saw_separator:
        fail("retained-hash table header or separator missing")
    return rows


def build_hashes(name: str) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for line in (ROOT / name).read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"  ([0-9a-f]+)  (.+)", line)
        if not match:
            continue
        digest, path = match.groups()
        hashes[path.replace("\\", "/").removeprefix("./")] = require_sha(
            digest, f"{name}:{path}"
        )
    if not hashes:
        fail(f"no hashes in {name}")
    return hashes


def verify_native_controls(prefix: str, exact: dict[str, str]) -> None:
    rows = [
        json.loads(line)
        for line in (ROOT / f"controls-{prefix}.results.jsonl")
        .read_text(encoding="utf-8")
        .splitlines()
        if line
    ]
    if len(rows) != 18:
        fail(f"controls-{prefix}: expected 18 attempts, found {len(rows)}")
    wanted_lua = (
        "b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca"
        if prefix == "puc"
        else "8a228b5437520bc8dda3e77aac7bb7f81c10d23f0747adbcbafe95647533b755"
    )
    expected_attempts = {
        (control, repetition)
        for control in (
            "empty",
            "first-error",
            "host-log",
            "load-only",
            "lua-compat",
            "plain-error",
        )
        for repetition in range(1, 4)
    }
    observed_attempts: set[tuple[object, object]] = set()
    scripts: dict[object, str] = {}
    for index, row in enumerate(rows, 1):
        attempt = (row.get("control"), row.get("rep"))
        if attempt in observed_attempts:
            fail(f"controls-{prefix} attempt {index}: duplicate {attempt!r}")
        observed_attempts.add(attempt)
        if row.get("exit") != "0x00000000" or row.get("timedOut") is not False:
            fail(f"controls-{prefix} attempt {index}: exit/timeout failure")
        if row.get("pass") is not True or row.get("reachedEnd") is not True:
            fail(f"controls-{prefix} attempt {index}: incomplete control")
        for field in ("script", "host", "module", "lua"):
            require_sha(row.get(field), f"controls-{prefix} attempt {index}:{field}")
        if row["host"] != exact["eech-world.exe"]:
            fail(f"controls-{prefix} attempt {index}: host hash mismatch")
        if row["module"] != exact["eech_dc.dll"]:
            fail(f"controls-{prefix} attempt {index}: module hash mismatch")
        if row["lua"] != wanted_lua:
            fail(f"controls-{prefix} attempt {index}: Lua hash mismatch")
        prior_script = scripts.setdefault(row.get("control"), row["script"])
        if row["script"] != prior_script:
            fail(f"controls-{prefix} attempt {index}: script hash changed")
    if observed_attempts != expected_attempts:
        fail(f"controls-{prefix}: attempt set mismatch")
    summary = (ROOT / f"controls-{prefix}.summary.txt").read_text(encoding="utf-8")
    for label, digest in (
        ("host", exact["eech-world.exe"]),
        ("module", exact["eech_dc.dll"]),
        ("lua", wanted_lua),
    ):
        if f"{label} {digest}" not in summary:
            fail(f"controls-{prefix} summary {label} hash mismatch")


def main() -> int:
    records: dict[str, str] = {}
    for label, path, digest in retained_hash_rows(
        (ROOT / "README.md").read_text(encoding="utf-8")
    ):
        if label != path or Path(path).name != path:
            fail(f"invalid retained-hash path: {label!r} -> {path!r}")
        if path in records:
            fail(f"duplicate retained-hash row: {path}")
        records[path] = digest
    if set(records) != EXPECTED:
        fail(
            "retained-hash manifest mismatch: "
            f"missing={sorted(EXPECTED - set(records))}, "
            f"extra={sorted(set(records) - EXPECTED)}"
        )
    for path, expected in records.items():
        artifact = ROOT / path
        if not artifact.is_file():
            fail(f"missing retained artifact: {path}")
        actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
        if actual != expected:
            fail(f"retained artifact hash mismatch: {path}: {actual}, expected {expected}")

    m4 = build_hashes("M4-BUILD-INFO.txt")
    exact = build_hashes("EXACT-HEAD-BUILD-INFO.txt")
    causal = json.loads((ROOT / "causal.summary.json").read_text(encoding="utf-8"))
    lifecycle = json.loads(
        (ROOT / "exact-head-lifecycle.summary.json").read_text(encoding="utf-8")
    )
    independent = json.loads(
        (ROOT / "independent-release-source-equivalence.json").read_text(
            encoding="utf-8"
        )
    )
    report = json.loads((ROOT / "m4-pack.report.json").read_text(encoding="utf-8"))

    for label, value in (
        ("causal module", causal.get("eech_dc_sha256")),
        ("causal host", causal.get("eech_world_sha256")),
        ("causal script", causal.get("script_sha256")),
        ("lifecycle module", lifecycle.get("eech_dc_sha256")),
        ("lifecycle host", lifecycle.get("eech_world_sha256")),
        ("lifecycle script", lifecycle.get("script_sha256")),
        ("independent M4 module", independent.get("retained_M4_DLL_sha256")),
        ("independent M4 host", independent.get("retained_M4_exe_sha256")),
        ("independent normalized source", independent.get("normalized_host_source_sha256")),
    ):
        require_sha(value, label)
    if causal.get("executable_source") != "1578435b3557570635cccbcc416cf5fcb15d2076":
        fail("causal executable source mismatch")
    if causal.get("causal_test_source") != "13846570326de9b0a536db60cc40d143c9f7a75f":
        fail("causal test source mismatch")
    if causal.get("integration_head") != "4028c5a96510d7ef72d2e09c3fe36f3e7038ae88":
        fail("causal integration head mismatch")
    if lifecycle.get("source") != "a352c126793e3d1a375af06473c6ae656a95ab00":
        fail("lifecycle source mismatch")
    if independent.get("native_source") != causal["executable_source"]:
        fail("independent native source mismatch")
    if independent.get("lint_head") != lifecycle["source"]:
        fail("independent lint head mismatch")
    if independent.get("moved_cfg_test_module_identical") is not True:
        fail("independent cfg(test) equivalence is not true")
    production_equivalent = independent.get(
        "production_host_source_identical_after_lint_and_blank_line_removal"
    )
    if production_equivalent is not True:
        fail("independent production-source equivalence is not true")
    if causal["eech_dc_sha256"] != m4["eech_dc.dll"]:
        fail("causal module does not match M4 build")
    if causal["eech_world_sha256"] != m4["eech-world.exe"]:
        fail("causal host does not match M4 build")
    if lifecycle["eech_dc_sha256"] != exact["eech_dc.dll"]:
        fail("lifecycle module does not match exact-head build")
    if lifecycle["eech_world_sha256"] != exact["eech-world.exe"]:
        fail("lifecycle host does not match exact-head build")
    if independent["retained_M4_DLL_sha256"] != m4["eech_dc.dll"]:
        fail("independent M4 module does not match M4 build")
    if independent["retained_M4_exe_sha256"] != m4["eech-world.exe"]:
        fail("independent M4 host does not match M4 build")
    report_files = report.get("head", {}).get("build", {}).get("files", {})
    for path in ("eech_dc.dll", "eech-world.exe", "lua.dll"):
        if require_sha(report_files.get(path), f"M4 report:{path}") != m4[path]:
            fail(f"M4 report {path} does not match M4 build")
    if len(report.get("checks", {})) != 40:
        fail("M4 report does not contain exactly 40 checks")
    if any(check.get("status") != "PASS" for check in report["checks"].values()):
        fail("M4 report contains a non-passing check")
    expected_cases = {
        "causal": {"reference", "withhold_pick_up", "delay_drop_off"},
        "lifecycle": {"missing", "malformed", "error", "reentry"},
    }
    for group, summary in (("causal", causal), ("lifecycle", lifecycle)):
        cases = summary.get("cases")
        if not isinstance(cases, list):
            fail(f"{group}: cases are missing")
        names = [case.get("case") for case in cases]
        if len(names) != len(set(names)) or set(names) != expected_cases[group]:
            fail(f"{group}: case set mismatch")
        for case in cases:
            if case.get("exit") != "0x00000000" or case.get("timedOut") is not False:
                fail(f"{group} {case.get('case')}: exit/timeout failure")
            if case.get("pass") is not True:
                fail(f"{group} {case.get('case')}: not passing")
            require_sha(case.get("output_sha256"), f"{group} {case.get('case')}:output")
            require_sha(case.get("host_log_sha256"), f"{group} {case.get('case')}:log")
            retained_output = case.get("retained_output")
            if retained_output is not None:
                if retained_output not in records:
                    fail(f"{group} {case.get('case')}: unknown retained output")
                if case["output_sha256"] != records[retained_output]:
                    fail(f"{group} {case.get('case')}: retained output hash mismatch")
    verify_native_controls("puc", exact)
    verify_native_controls("dcs", exact)

    print(f"PASS: {len(records)} retained SUPPLY-arrival artifacts and identities")
    return 0


if __name__ == "__main__":
    sys.exit(main())
