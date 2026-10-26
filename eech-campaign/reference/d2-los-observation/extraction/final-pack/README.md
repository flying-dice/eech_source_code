# Final LOS shipping regression — EEDCS-42

Runtime source: `61e85664540770ab10e3ac7e9f5dfa916291c52f`, clean ordinary-feature
shipping build with the unchanged recipe and a fresh target volume. Build identity
is in `../checks/final-shipping-BUILD-INFO.txt`. The supplied-binary destination
baseline is in `../destination-baseline/`; it exited 0 with all 40 checks passing.

**40/40 candidate checks passed; complete artifact comparisons matched. The
original runner exited 1 on a cross-drive report-display error; separate report-only
recovery exited 0 without rerunning scenarios or changing results.**

The original command finished both runtime tiers and wrote `report.json` and
`report.md` at15:54:05 UTC on26October2026. Its final display then called
`os.path.relpath(R:report, C:cwd)` and raised `ValueError`; see
`original-report-display-failure.log`. The original reports remain intact.

`complete-report-only.py` imports the **unchanged** frozen report writer and calls
only `write_report` with the original results and an explicit R: native working
directory. It writes to a separate `report-completion/` directory. The report-only
process exited0. Checks, claims, counts, blind spots, disposition, baseline, roots,
build identity and source revision are unchanged; only reporting time and command
differ. `report-only-completion.json` records hashes of the original reports,
console and eight scenario artifacts before/after recovery. The same independent
Codex reviewer inspected this new recovery and independently rehashed the originals:
no blocking evidence issue. This was a focused evidence check, not another source pass.

`complete-file-comparison.json`: both complete Tacview recordings, both observation
streams, both reference metrics and Georgia/unobserved-Lebanon regression metrics
match the supplied baseline **byte for byte** (8/8). The complete reference files
also match accepted D1 sizes/SHA-256. All three scenario roots still match repository
input manifests after completion; actual Python comparisons exited0.

Original raw data remains outside Git under
`R:/d2-evidence/candidate-pack-los-61e856645`. These are readable text copies;
Git may normalize line endings. Preservation hashes identify the named original
external files, not the normalized copies. Private retail inputs/binaries and full
raw recordings were not uploaded. No accepted baseline or tolerance changed.

Recording equality is not visual inspection. Existing EVIDENCE ONLY/NOT COVERED
claims and known scenario blind spots remain unchanged. Current CI, normal merge
and Jira/Confluence delivery records are separate remaining delivery gates.
