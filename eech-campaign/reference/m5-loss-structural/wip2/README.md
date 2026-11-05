# The superseded working candidate `wip2`

This is history only, not the evidence for the committed change.

- **Source:** built from `f7a3b756` plus 13 uncommitted files, 11 of them build inputs (`prepack-provenance/manifest.json`, hashed by the lead before the pack).
- **Binaries:** `eech_dc.dll` `daea0ddf…`, `eech-world.exe` `ae0b3bff…` (`BUILD-INFO.txt`).
- **Results:**
  - the M4 pack, run unchanged by the lead: PASS, 40 of 40 checks (`report.md`, `report.json`);
  - a public-path probe with campaign diagnostics equal to the retained diag-A hashes (`probe-*`);
  - the person `kill_local` codegen comparison (`ps-kill_local*`).
- **Why it was revised:** the pre-commit audits moved the `MOVEMENT_DEPENDENT` unlinks back inline, renamed a helper and edited comments. So these results do not identify the committed source.
