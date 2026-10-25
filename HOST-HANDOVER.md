# Host handover — 25 October 2026

## Resume point

D1 (EEDCS-15) is accepted and Done. All 15 child tasks are Done. D2 has not started.
Delivered integration branch: `claude/eech-campaign-native-spike-tjkj6g`, commit
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`. Do not start from the default branch.
This handover branch adds documentation only to that accepted tree.

Read live scope before further work:
- M5 authority: https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337
- Migration ledger and D1 acceptance: https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994 (acceptance v25)
- Epic: https://flying-dice.atlassian.net/browse/EEDCS-15 (acceptance comment 10714)
- Final delivery: https://github.com/flying-dice/eech_source_code/pull/100

The final pack passed 40/40; both full Tacview and observation recordings equal
accepted bytes and campaign metrics match. All nine CI checks and independent
exact-head approval passed. Full evidence and limits are committed under
`eech-campaign/reference/m5-all-family-loss/`. No need to repeat six-hour runs
merely to establish the handover. Validate copied input manifests first.

## Clone and regression data

Clone `git@github.com:flying-dice/eech_source_code.git`, selecting the integration
branch above. Fetch `codex/host-handover-20261025` for this note and the final
closure audit under `docs/handover/`. No Git repositories are in the network bundle.

Network bundle: `\\192.168.8.1\fs1\eech-regression-20261025`.
Copy it to local storage before running. `SHA256SUMS.json` lists every payload
file's SHA-256 and size; `verify-bundle.py` verifies the payload after copying.

Contents:
- `roots/georgia`: accepted Georgia inputs (1,174 manifest files).
- `roots/lebanon`: accepted Lebanon inputs (1,179 manifest files).
- `roots/lebanon-repeat`: independently assembled repeat root, same manifest.
- `bin`: tested Windows x64 default22 executable, campaign DLL, lua.dll and Lua scripts.
- `evidence`: final pack text/JSON reports and local independent review records.

Retail data is private runtime input, not source to upload to GitHub.
The bundle preserves engine-generated configuration too; do not arbitrarily
replace it when trying to reproduce the accepted run.

On Windows, with Python 3 and PowerShell installed, from `eech-campaign`:

```powershell
$bundle = 'D:/eech-regression-20261025' # your local copy
python "$bundle/verify-bundle.py"
python tools/input-manifest.py "$bundle/roots/georgia" --compare reference/m4-s3-mutation/inputs-georgia.json
python tools/input-manifest.py "$bundle/roots/lebanon" --compare reference/lebanon-3h/inputs.json
python tools/input-manifest.py "$bundle/roots/lebanon-repeat" --compare reference/lebanon-3h/inputs.json
# Run only when needed; use a fresh output directory, never the baseline directories:
python tools/regression-pack.py --georgia "$bundle/roots/georgia" --lebanon "$bundle/roots/lebanon" --lebanon-repeat "$bundle/roots/lebanon-repeat" --bin "$bundle/bin" --out D:/eech-new-run
```

The supplied binaries are Windows x64. Rebuilding uses Git Bash, Docker with
Linux containers and the repository's `eech-campaign/tools/build-windows.sh` /
Dockerfile. Build caches are deliberately excluded. Other operating systems
need a separate runtime validation plan; do not claim the Windows pack ran there.

## Work policy and unfinished branches

Use Claude CLI `-p`: compact Opus 5.5 Medium planning only when needed, Sonnet
High implementation. One implementation lane and one independent Codex reviewer;
no nested agents or review fleets. If quota expires, one GPT Sol High fallback,
preserving the checkpoint. User previously authorized bypass permissions, but
check supported CLI flags locally. Do not copy credentials or change authentication.
The destination needs its own existing GitHub, Atlassian and Claude access.
Fresh sessions can use this handover; old CLI resume IDs are machine-local and
their histories are not included in this regression bundle.

The DCS call-map branch `codex/m5-dcs-call-map` is unfinished PR83 work. Its
inventory describes an older pinned source and must be refreshed for current D1;
preserving/pushing it is not criterion-1 acceptance. Boundary-guard PR87 remains
paused; PR59/master promotion is not part of D1 acceptance. Read live PR state.

The separate `flying-dice/ee-dcs` repository contains historical TypeScript-port
work. Any handover checkpoint there is preservation only: it is not the current
M5 native campaign scope and must not restart the abandoned port.

## Evidence limits

Headless runtime coverage does not include live ship-route movement or player
recon. Source/dispatch/kernel evidence supports those extractions. Loss-report
withholding is bounded to characterized cases, not arbitrary delay, final-member
withholding or save/restore. Some reused family witnesses have partial-tree
provenance. Tacview comparison is full-file equality, not visual review.
Terrain, combat/perception, airfields/logistics and final world-code removal are
later D2–D5 work; live DCS operation is M6. D1 completion is not M5 completion.

## Opening prompt for the new session

Read HOST-HANDOVER.md, AGENT.md, the live M5 authority and migration ledger.
Verify the integration checkout and regression bundle hashes/manifests.
D1 is complete; D2 has not started. Report readiness and the next bounded
migration extraction before starting it. Preserve the one-author/one-reviewer
workflow. Do not repeat completed experiments without a change that warrants it.
