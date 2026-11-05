# #76 candidate validation: windows-retail v1 `-Exact` and the M4 runtime pack

The rebuilt candidate of PR #76 run through the whole M4 regression pack
([`docs/m4-regression-pack.md`](../../docs/m4-regression-pack.md)), on the retail roots, against the accepted baseline
**windows-retail v1**. Recorded in the form of [`BASELINE_RECORD.md`](../../../docs/governance/BASELINE_RECORD.md).

**It is intended to serve #59 item 2** (lead review 5587358297): "Rebuild the resulting integration candidate and run
Georgia and Lebanon against windows-retail v1 with -Exact, plus the applicable M4 runtime pack/public-path scenarios."
- It qualifies only if #59's merged head builds to the same binaries. The hashes are below.
- If the candidate changes before #59 merges, rebuild with `tools/build-windows.sh` and compare `BUILD-INFO.txt`. Identical `eech_dc.dll` and `eech-world.exe` hashes mean this run applies; different ones mean it must be repeated.

## Identity

| | |
|---|---|
| Task / Epic / PR | #75 / #35 (M5) / #76; for #59 item 2 |
| Candidate commit | `d38b9b2d30a25e063e8b48230120020b7184327b` (`ci/integration-green`). It merges the integration head `74aaa168` (with #73) in `433871fa`, and #76's own changes |
| Build inputs | E4 (`303103d2`) is the only build-input change on #76 relative to the integration head `74aaa168`. The inventory change is test-only (`crates/eech-sys/tests`, `global-state.txt`). `433871fa` and `d38b9b2d` add governance and documentation files, plus this record and the E4 record, which no build reads. |
| Build | `tools/build-windows.sh target/pr76-candidate`, with `EECH_WIN_TARGET=eech-win-target-pr76` (a fresh cargo target volume). [`BUILD-INFO.txt`](BUILD-INFO.txt): 0 uncommitted build inputs; image `sha256:0c60c75d…bab3`, the same `eech-build` image as the accepted M1 build (`target/m1-no-s2/BUILD-INFO.txt`); rustc 1.98.1; MinGW GCC 12.2.0 (posix); release, x86_64-pc-windows-gnu |
| Binaries (SHA-256) | `eech_dc.dll` `186857dfaf280c653d0a537cdc44ad74c41c2c254ad17b83175f7d2a743a8763` · `eech-world.exe` `56f08086f5c371bf626902eba8147cc471b46b07af064c4194755541ceb69471` · `lua.dll` `d0c799fe…0350` and the four `lua/*.lua` scripts are the same as the accepted build's |
| Accepted build compared | `target/m3-reference-bin` (candidate `0288ece2`, scripts `0ea49a4a`): `eech_dc.dll` `da72f128…`, `eech-world.exe` `314fde3d…`. Both differ, as expected: the engine and host code changed since `0288ece2` (#62–#69, `b4d43ea6`, E4). |
| Baseline | windows-retail v1 (`regression/baselines/lineages.json`, established `e0801614`; unchanged) |
| Retail inputs | Georgia `E:/eech-runs/roots/georgia`: 1,174 files, combined SHA-256 `d373bc42…50de2`. Lebanon `E:/eech-runs/m2/lebanon-root` and `E:/eech-runs/m2/lebanon-m1-copy`: 1,179 files each, `121a7890…59c09`. Each matched its accepted manifest (`logs/inputs.*.txt`). |
| Configuration | seed 1, 3 simulated hours, 100 ms frames (`regression/pack.json`) |
| Host | Windows 11 Pro 10.0.26200, the user's retail PC. No other EECH process was running at the start (checked with `Get-Process`). |
| Time | 2026-10-20 16:30:24 – 16:49:08 UTC. The report's "run 17:49:08" is local time (BST). |

## Command

```powershell
# eech-campaign\, at d38b9b2d, clean
python tools\regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root `
  --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target\pr76-candidate --out target\regression-pack\pr76-candidate
```

## Result: **REGRESSION PACK: PASS** (exit 0)

**Claims:**

| Status | Claims |
|---|---|
| PASS - directly protected | 17 |
| PASS - baseline-sensitive | 12 |
| EVIDENCE ONLY | 8 |
| NOT COVERED | 8 |
| NOT RUN | 0 |

**Checks:**

| | |
|---|---|
| **Retained tier** | All 15 checks pass, including the lineage and the governance fixture |
| **Inputs** | All three roots are IDENTICAL to their manifests |
| **Lifecycle** (public Lua path, `tools/lifecycle-windows.ps1`) | boot_twice, bad_arguments, missing_root, missing_campaign and no_reuse PASS. missing_map is the known characterised defect, unchanged |
| **Georgia `-Exact`** | **IDENTICAL** to windows-retail v1; expectations 30 of 30 |
| **Lebanon `-Exact`** | **IDENTICAL** to windows-retail v1; mechanics 37 of 37, resupply 2 of 2 |
| **Reference** (`tools/reference-windows.ps1 -ObserveSupply`, two runs) | Observation check PASS in both. Recording, observations and metrics are byte-identical between the runs. Metrics are IDENTICAL to the baseline, with 39 of 39 expectations. Observed = unobserved. Supply chains: 31 complete of 32 deliveries in both runs. Loss chain: Mi-17 87568 at 1584.66 s, PASS |
| **Against the retained evidence** | `recording.acmi` SHA-256 `3bd17b64…d2d4` equals `m2.recording`'s content hash. `observations.jsonl` `da26bab0…22e77cf` equals `m3.observations`' content hash. The check, chain and loss reports equal the retained ones. |

**The metrics files.** `regress/georgia_retail.json` (`74276605…148d`) and `lebanon_retail.json` (`fa09d562…2587`) do not hash like the committed baseline files. Parsed, they are equal to `regression/windows/*.json` in every field, so only the serialisation differs. `regress-compare.py --exact` compares content and reports IDENTICAL.

## Differences and classification

**None.** Every exact comparison is IDENTICAL, and every check passes, so there is nothing to classify and no change record. No baseline was re-recorded, and no tolerance or expectation was changed.

As expected from the code:
- E4 is unreachable in the headless product ([`../e4-timer-callback/`](../e4-timer-callback/)).
- The other changes since `0288ece2` were accepted as behaviour-preserving, or are the accepted corrections windows-retail v1 already contains.

## Retained here

- `report.md` and `report.json`: every claim's status and every check's detail.
- `output.txt`: the console output.
- `logs/`: the existing runners' output: inputs, lifecycle, regress, reference, loss chain, tolerance comparisons, observed vs unobserved.
- `chains.json` (reference run 1) and `loss-chain.json`.
- `BUILD-INFO.txt`.

The full outputs stay local, under `target/regression-pack/pr76-candidate/`, as in M4: the recordings, observations and roots (about 750 MB), which include retail data.

## Limitations

- **One seed, 3 simulated hours.** These are M4's residual risks, unchanged: 8 evidence-only and 8 uncovered claims (N1–N3, U1, longer runs, callbacks and reentrancy, adaptive compensation, visual Tacview review), and the blind spots in `report.md`.
- **No visual Tacview review.** The recording was compared by hash with the retained one, not visually inspected.
- **Local retail data.** The runtime tier depends on this PC's retail roots; it is not hosted-CI evidence.
- **#59 applicability.** The run serves #59 item 2 only while #59's head builds to the hashes above. The decision is the lead's.
