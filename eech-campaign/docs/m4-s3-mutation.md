# M4: does the accepted regression detect a reintroduced campaign defect?

This record is part of M4 ([#31](https://github.com/flying-dice/eech_source_code/issues/31)). It answers one question: **if a structural change reintroduces a meaningful campaign defect, does our accepted public-path evidence detect it and tell us what behaviour changed?**

**Answer: yes, for the S3 defect.** The accepted Windows regression was used unchanged: `tools/regress-windows.ps1 -Exact`, running retail Georgia and Lebanon through `eech-world.exe` → `campaign.lua` → `require ("eech_dc")`. The control passes it; the build with S3 removed fails it.
- **Lebanon:** the campaign expectation "airbases resupplied with ammo" fails, and the metrics differ from the baseline.
- **Georgia:** the metrics differ from the baseline.

The metrics that differ name the behaviour: ammo stops reaching airbases and military bases, while fuel deliveries rise. That is the known S3 defect, where a supply task picks up the wrong crate type.

## The controlled negative

S3 (`S3-pick-up-the-tasks-cargo`, reviewed in [`corrections.md`](corrections.md) and accepted as a defect correction in #62) makes a SUPPLY transport take a crate of its task's cargo type. Without it, the transport takes the supplier's first crate whatever its type. So an ammo task can load fuel and deliver fuel.

The mutant is **test-only** and never touches the product:

- `tools/build-mutant-windows.sh <patch id | control> target/mutants/<dir>` builds from a **temporary git worktree** of the clean HEAD, under `target/mutants/`, removed afterwards:
  - it removes that one patch entry from `build/patches.rs` in the worktree only, and checks the change is that entry and nothing else;
  - it builds in a **cargo target volume of its own**, so the normal volume never compiles mutant code;
  - it writes only under `target/mutants/`, stamped **MUTANT** or **CONTROL**, with the source diff and the patch markers found in the sources it compiled.
- The normal build (`tools/build-windows.sh`, `target/windows`), the working tree and the committed sources are never changed. Nothing in this PR lets a normal build leave a patch out.
- `tools/mutation-windows.ps1` builds a control and a mutant the same way, checks they are a clean pair, gives each mutant run identical copies of the inputs, and runs the accepted regression on all three side by side.

## Evidence (Windows 11, local, 2026-09-26)

```powershell
tools\mutation-windows.ps1 -Georgia <Georgia root> -Lebanon <Lebanon root> -Patch S3-pick-up-the-tasks-cargo -RepeatMutant
```

**Why the failure is S3 alone** ([`reference/m4-s3-mutation/`](../reference/m4-s3-mutation/)):

| Check | Result |
|---|---|
| Source revision | Control and mutant are built from the same commit, `56b164d7` (`control-BUILD-INFO.txt`, `mutant-BUILD-INFO.txt`) |
| Build | The same image (`sha256:0c60c75d…`), rustc 1.98.1, MinGW GCC 12, `lua.dll` and Lua scripts |
| Source difference | `mutation.diff`: in `patches.rs`, the `S3-pick-up-the-tasks-cargo` entry removed (8 lines), nothing added |
| What was compiled | Patch markers in the staged sources: the control's 17, the mutant's 16. Only `EECH headless (S3)` is missing. |
| Inputs | Each mutant run used copies of the roots, identical in all input files: Lebanon 1,179 files `121a7890…`, Georgia 1,174 files `d373bc42…` |
| Scenario and time | The same scripts, seed 1, 3 simulated hours, 100 ms frames |
| The difference is not noise | Two mutant runs, on two copies of the inputs, gave **byte-identical** metrics for both scenarios. The runs are deterministic (M2). |

**Result** (`summary.txt`, `regress-*.txt`):

| Run | Georgia | Lebanon |
|---|---|---|
| **Control** | **PASS**: 30/30 expectations, IDENTICAL | **PASS**: 39/39 expectations, IDENTICAL |
| **Mutant** (and its repeat) | **FAIL**: 30/30 expectations, but **not identical** to the baseline | **FAIL**: **38/39**, with `EXPECTATION FAILED airbases resupplied with ammo (0)`, and **not identical** to the baseline |

The control's metrics equal the committed baselines byte for byte (Lebanon's once line endings are normalised). So the separate worktree build reproduces the accepted behaviour exactly.

The tolerance comparison flags the mutant as well, without `-Exact`: 11 Georgia metrics and 13 Lebanon metrics fall outside tolerance (`tolerance-mode.txt`). Detection does not depend on exact identity.

## What behaviour changed

From `difference.txt` (control → mutant, after 3 simulated hours):

| | Lebanon | Georgia |
|---|---|---|
| Ammo deliveries to red airbases | 3 → **0** | 6 → **1** |
| Ammo deliveries to red military bases | 6 → **0** | — |
| Ammo deliveries to blue military bases | 1 → **0** | — |
| Fuel deliveries to red airbases | 8 → **12** | 5 → **11** |
| Fuel deliveries to red military bases | 1 → **5** | — |
| Red airbase ammo (mean) | 43.3 → 37 | 55.3 → 46.8 |
| Red airbase fuel (mean) | 32.3 → 44.9 | 28.2 → 43.3 |

Ammo deliveries vanish, and fuel deliveries and fuel levels rise. That is the signature of ammo tasks picking up fuel crates: the drop-off refills by the carried crate's type (`ks_msgs.c`), which is the defect S3 corrects.

The rest of each war diverges from there: 148 of 272 Lebanon metrics and 116 of 180 Georgia metrics differ. For example, blue helicopters alive in Georgia go from 31 to 19. That divergence follows from the supply change; it is not a separate defect.

## What this protects, and what it does not

**Protected**, by a demonstrated sensitivity:
- **The keysite resupply behaviour S3 establishes:** SUPPLY tasks deliver the resource they were created for, and airbases and military bases get ammo back.
  - In Lebanon both checks catch its loss: the "airbases resupplied with ammo" expectation and the baseline comparison.
  - In Georgia only the baseline comparison catches it. Georgia's expectations don't check resupply, since the retail map has no producers. This blind spot in the expectations is covered by the baseline.

**Outside this slice:**
- **Other defects.** Only S3 was removed; the sensitivity to other defects, or to other kinds of change, is not shown here.
- **Changes smaller than a whole mechanic.** S3 changes a whole supply mechanic within 3 hours. A regression that shows only after 3 hours, or only in a rare case, might pass.
- **Linux, CI and other scenarios.** The regression needs local retail data, so it runs locally, not in hosted CI.
- **Automatic judgement of a change.** A red result says *that* behaviour changed and lists *which* metrics moved. Classifying the change (regression, correction or intended change) remains a review decision.

## Incident during this slice

The first version of `build-mutant-windows.sh` cleaned up with `git worktree prune`. That also deleted the record of another session's worktree (`.claude/worktrees/governance-project-review-f9dd93`), because Windows git took its recorded path as stale.
- **Damage:** its files and branch were untouched.
- **Repair:** the record was restored by hand, and its index rebuilt from its branch. Its status is clean, so nothing was lost.
- **Prevention:** the builder now removes only the worktree it created.

## Reproduce

```powershell
tools\mutation-windows.ps1 -Georgia <root> -Lebanon <root> -Patch S3-pick-up-the-tasks-cargo -RepeatMutant
# the outputs, under target\mutants\s3\: summary.txt, difference.txt, the builds, and regress-*.txt
```
