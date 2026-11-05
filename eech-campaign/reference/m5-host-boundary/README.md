# M5 host boundary: retained characterisation evidence and report

Proposal: [`docs/m5-host-boundary.md`](../../docs/m5-host-boundary.md). Branch `codex/m5-host-boundary` from `163e9158` (the PR #79 merge). The product tree is `65892de1`. #79 changed only retained evidence and the CI workflow, neither of which is a product build input.

## Report of the proposal pass (`f7a3b756`)

This is the report of the proposal pass. Slice 1a, authorised later, is in [`docs/m5-loss-structural.md`](../../docs/m5-loss-structural.md).

- **STATUS:** proposal and characterisation are complete for the lead's decision. No product, build, baseline or expectation change, and nothing pushed.
- **DID:**
  - read the Confluence goal 11534337 (live, v1) and the working plan 12484650 v3, the call map at `a4cd7eea`, and #77 and #78;
  - read the arrival and loss source;
  - ran two public-path characterisation runs on accepted inputs.
- **EVIDENCE:** the files in this directory.
  - The natural M3 loss of Mi-17 87568 was reproduced at 1,584.66 s.
  - The campaign books it inside the DLL: red ARMED_HELICOPTER 92 → 91, REGEN_HELICOPTER 3 → 4, Wolfpack members 2 → 1.
  - The campaign diagnostics are **byte-identical** whether or not the host observes anything. This is the red for criteria 2 and 3.
- **DECISIONS NEEDED:** proposal §6.
- **OUT OF SCOPE NOTICED:** the uneven per-type loss bookkeeping. Regen applies only to helicopters, fixed wing and routed vehicles, and the objectives check only to helicopters. It is preserved, not fixed.

## Runs

| | |
|---|---|
| Build | #59 candidate, a copy of `.claude/worktrees/agent-a9904988ee5ea2a0f/eech-campaign/target/pr59-candidate`, checked against `reference/pr59-candidate-identity/BUILD-INFO.txt`: `eech_dc.dll` `a99d7a1118008f73a995fd7e4701a04bddb5fa51d1bf3b5a487297d78c383f41`, `eech-world.exe` `13cc2a6f…`, `lua.dll` `d0c799fe…` |
| Root | A copy of `E:/eech-runs/m2/lebanon-root` at `E:/eech-runs/m5-host-boundary/root-lebanon`. `python -I tools/input-manifest.py <copy> --compare reference/lebanon-3h/inputs.json` gives IDENTICAL (1,179 files, `121a7890…`), before run A and between the runs. |
| Scenario | Retail Lebanon map5 `LEBANON.CHC`, seed 1, gunship apache, 100 ms frames, 18,000 frames |
| A | `eech-world.exe probe.lua root=… seconds=1800 observe=1 out=A.jsonl diag_at=1580,1590,1795`: 3 min 12 s, exit 0, 2026-10-20 23:06 UTC |
| B | The same with `observe=0` (the host never calls `objects`): 2 min 03 s, exit 0 |

## Files

- `probe.lua`: the host script. It supplies only boot configuration and milliseconds.
- `A.jsonl`, `B.jsonl`: the surface probe, the changes to unit 87568 and group 87567, and the diagnostics points.
- `diag-{A,B}-{1580,1590,1795}.txt`: the campaign force-state diagnostics (`engine:diagnostics ()`), without their header line.
- `loss-delta.txt`: the lines of `diag-A-1580` against `diag-A-1590` for red's force, the regen queues and Wolfpack.

## Check

```sh
cmp diag-A-1580.txt diag-B-1580.txt && cmp diag-A-1590.txt diag-B-1590.txt && cmp diag-A-1795.txt diag-B-1795.txt
sha256sum diag-A-*.txt   # 70b59f20… (1580), 5a4c40aa… (1590), 65f82373… (1795)
```

Today the `cmp` succeeds, and that is the defect for criterion 3. After the proposed slice 1c, a host that withholds 87568's report must make the 1,590 s diagnostics differ in exactly the loss lines.
