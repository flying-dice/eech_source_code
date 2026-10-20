# #59 candidate identity and validation: integration `65892de1` against windows-retail v1

This record identifies the build of the integration candidate that #59 would promote to `master`. It answers item 2 of the lead's #59 review
(5587358297) and the decision in EEDCS-5 comment 10423: "establish final integration build identity against #76's validated DLL/host/Lua/scripts/configuration. Reuse the accepted runtime evidence only when that identity is demonstrated; otherwise rerun the applicable M4/runtime exact checks."

It follows the form of [`BASELINE_RECORD.md`](../../../docs/governance/BASELINE_RECORD.md). The #76 record it compares against is [`../pr76-candidate-validation/`](../pr76-candidate-validation/).

## Result

- **Source and configuration: identical.** The candidate `65892de1` differs from #76's validated commit `d38b9b2d` only by #76's validation record (18 files under `reference/pr76-candidate-validation/`), which no build or runner reads.
- **Toolchain: identical.** The image, rustc, cargo, MinGW GCC and Lua are the same.
- **Lua runtime and scripts: identical.** `lua.dll` and the four `lua/*.lua` files have the same SHA-256.
- **`eech_dc.dll` and `eech-world.exe`: different file hashes, same code and data.** A from-scratch rebuild changes the link timestamps, the header checksum and the unmapped COFF symbol table, as `docs/m1-baseline.md` ("Rebuild check") already records. Every section the loader maps is byte-identical, except `.edata`, where only the export directory's timestamp differs.
- **So #76's runtime evidence was not reused.** #76's record says a rebuild "must be repeated" when the hashes differ. The file hashes differ, so the M4 pack runtime tier, including Georgia and Lebanon `-Exact` against windows-retail v1, was **rerun** on this candidate's build: **PASS**, with Georgia and Lebanon IDENTICAL to windows-retail v1 and every claim and check the same as #76's run. See "Runtime validation".

## Identity, side by side

| | #76 validated build | #59 candidate build | Same? |
|---|---|---|---|
| Commit | `d38b9b2d30a25e063e8b48230120020b7184327b` (#76 head before its record) | `65892de17bf5c4bb9ab6e9f8af70779e35b0106b` (integration head, #76's merge; #59's head) | No: differs only by `reference/pr76-candidate-validation/` |
| Build-input trees (`git ls-tree`) | `aphavoc` `d3309ea5`, `modules` `cb40c456`, `eech-campaign/crates` `99463f74`, `tools` `5d2c4211`, `regression` `090bd08a`, `Cargo.lock` `5d0b008a`, `Cargo.toml` `b1d15e2c` | the same IDs | Yes |
| Uncommitted build inputs | 0 | 0 | Yes |
| Build command | `EECH_WIN_TARGET=eech-win-target-pr76 tools/build-windows.sh target/pr76-candidate` | `EECH_WIN_TARGET=eech-win-target-pr59 tools/build-windows.sh target/pr59-candidate` (a new, empty cargo target volume) | Same procedure |
| Image | `sha256:0c60c75db7b035e830c4834b1e15e096483631915bafc22c1af6aae02032bab3` | the same | Yes |
| rustc / cargo | 1.98.1 (48a229cea 2026-09-01) / 1.98.1 (797e8a9bc 2026-08-05) | the same | Yes |
| MinGW | gcc 12-posix, gcc-mingw-w64-x86-64-posix 12.2.0-14+deb12u1+25.2+b1 | the same | Yes |
| Target | x86_64-pc-windows-gnu, release | the same | Yes |
| `eech_dc.dll` SHA-256 | `186857dfaf280c653d0a537cdc44ad74c41c2c254ad17b83175f7d2a743a8763` | `a99d7a1118008f73a995fd7e4701a04bddb5fa51d1bf3b5a487297d78c383f41` | **No** (code and data the same; see below) |
| `eech-world.exe` SHA-256 | `56f08086f5c371bf626902eba8147cc471b46b07af064c4194755541ceb69471` | `13cc2a6f00ec570a907c701bf2f8711429934b7f833c01bd847a1f492988a228` | **No** (code and data the same; see below) |
| `lua.dll` | `d0c799fe4e1e615d335d18504d28a7b0d7c78d7f97d1754bb885b1097ac60350` | the same | Yes |
| `lua/campaign.lua` | `4b1fa47d8426e9b9a1806c2bffb746c9843c9cf70ab46bc205f76ddc803ecbe2` | the same | Yes |
| `lua/lifecycle.lua` | `dbce9000c176e8563c0443d4b82942b4aef332393fb95da4c1fab738f3c3f30c` | the same | Yes |
| `lua/metrics.lua` | `dd28274e374e2698a84c0725e1e5ab3d5a564f13ecc0457bfce765df634d3053` | the same | Yes |
| `lua/observations.lua` | `68527ae645ad156de3b2887b8cb905156f7ad470da7735e2db96489fef6bd701` | the same | Yes |
| Configuration | `regression/pack.json` and `regression/baselines/lineages.json` in tree `090bd08a`: seed 1, 3 simulated hours, 100 ms frames; windows-retail v1 | the same tree | Yes |
| Georgia root `E:/eech-runs/roots/georgia` | 1,174 files, `d373bc42…b2b50de2` | 1,174 files, `d373bc4262464da87532b8128d396358f0003bc2e9e51e8fa92cf0a6e2b50de2`, IDENTICAL to `reference/m4-s3-mutation/inputs-georgia.json` | Yes |
| Lebanon roots `E:/eech-runs/m2/lebanon-root`, `lebanon-m1-copy` | 1,179 files each, `121a7890…4259c09` | 1,179 files each, `121a789003b68f9ec4d312f82afafdbb6c22be4a6c71720d66d4aeeaa4259c09`, IDENTICAL to `reference/lebanon-3h/inputs.json` | Yes |

### Where the binaries differ

[`compare-pe.py`](compare-pe.py) compares the two builds region by region ([`section-comparison.txt`](section-comparison.txt)). The #76 binaries are still on this PC, at the hashes above.

| | `eech_dc.dll` | `eech-world.exe` |
|---|---|---|
| Size | 16,657,810 bytes, both | 1,902,960 bytes, both |
| `.text`, `.data`, `.rdata`, `.pdata`, `.xdata`, `.idata`, `.CRT`, `.tls`, `.reloc` | byte-identical | byte-identical |
| `.edata` | differs only in the export directory's timestamp (2 bytes) | (none) |
| COFF `TimeDateStamp` | 2 bytes | 2 bytes |
| Optional-header `CheckSum` | 2 bytes | 2 bytes |
| COFF symbol/string table after the last section (not mapped by the loader) | 115,428 bytes | 22,472 bytes |

This is the same pattern as the M1 rebuild check (`docs/m1-baseline.md`), which the #60 review accepted as not an M1 blocker. It is strong evidence that the two builds execute the same code. It does not make the file hashes equal, and the reuse condition in #76's record is stated on the file hashes. So the runtime checks were repeated rather than inferred.

## Runtime validation (rerun)

The whole M4 regression pack ([`docs/m4-regression-pack.md`](../../docs/m4-regression-pack.md)) was run on the rebuilt candidate, on the retail roots, against **windows-retail v1**. The command and roots are the ones #76 used; only the build and output directories differ.

```powershell
# eech-campaign\, at 65892de1, clean
python tools\regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root `
  --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target\pr59-candidate --out target\regression-pack\pr59-candidate
```

| | |
|---|---|
| Host | Windows 11 Pro 10.0.26200, the user's retail PC |
| Other EECH processes | Another session's 8 `eech-world.exe` runs (its own roots and binaries, under `.claude/worktrees/agent-a20573899785375f2/`) were running at 19:37 UTC. The pack was started only after none was running: 0 at the start (20:10:36 UTC), and 0 foreign processes when it ended. |
| Time | 2026-10-20 20:10:36 – 20:33:33 UTC. The report's "run 21:33:32" is local time (BST). |
| Configuration | seed 1, 3 simulated hours, 100 ms frames (`regression/pack.json`) |

### Result: **REGRESSION PACK: PASS** (exit 0)

| Status | Claims |
|---|---|
| PASS - directly protected | 17 |
| PASS - baseline-sensitive | 12 |
| EVIDENCE ONLY | 8 |
| NOT COVERED | 8 |
| NOT RUN | 0 |

| Check | Result |
|---|---|
| Retained tier | All checks pass, including the lineage and the governance fixture |
| Inputs | All three roots IDENTICAL to their manifests (also checked before the run: [`logs/inputs-before-run.txt`](logs/inputs-before-run.txt)) |
| Lifecycle (public Lua path) | boot_twice, bad_arguments, missing_root, missing_campaign, no_reuse PASS; missing_map is the known characterised defect, unchanged |
| **Georgia `-Exact`** | **IDENTICAL** to windows-retail v1; expectations 30 of 30 |
| **Lebanon `-Exact`** | **IDENTICAL** to windows-retail v1; mechanics 37 of 37, resupply 2 of 2 |
| Reference (`-ObserveSupply`, two runs on two roots) | Observation check PASS in both; recording, observations and metrics byte-identical between the runs; metrics IDENTICAL to the baseline, 39 of 39; observed = unobserved; supply chains 31 complete of 32 deliveries in both; loss chain Mi-17 87568 at 1584.66 s PASS |
| Against the retained evidence | `recording.acmi` `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` = `m2.recording`'s content hash; `observations.jsonl` `da26bab01163aa4ab725f9f2ecb516d37a4558777fe3684222b8d885822e77cf` = `m3.observations`' content hash; the check, chain and loss reports equal the retained ones |
| Metrics files | `regress/georgia_retail.json` `74276605…9148d`, `lebanon_retail.json` `fa09d562…62587`: the same file hashes as #76's run (equal in content to `regression/windows/*.json`, as #76 records) |

(#76's README abbreviates the recording hash as `3bd17b64…d2d4`; the full hash in its `logs/reference.txt` ends `…d2d19b`, the same as here.)

### Against #76's run

Every claim status, every check status and every check detail are the same as #76's ([`comparison-report.txt`](comparison-report.txt), [`comparison-logs.txt`](comparison-logs.txt)). The only differences are identity and environment:
- the commit, the build path and the two binary hashes;
- the run time and the wall-clock durations (`runs took 1,286 s` and `1,232 s` here, against `1,025 s` and `971 s`);
- the worktree path in the manifest comparisons and in the chain and loss reports' input paths.

### Differences and classification

**None** against windows-retail v1. Every exact comparison is IDENTICAL and every check passes, so there is nothing to classify and no change record. No baseline was re-recorded, and no tolerance or expectation was changed.

### Retained here

- `report.md`, `report.json`, `output.txt` (the console output), `logs/`, `chains.json` (reference run 1) and `loss-chain.json`, as in #76's record.
- `BUILD-INFO.txt`, `section-comparison.txt` and the three comparison scripts with their outputs.

The full outputs stay local under `target/regression-pack/pr59-candidate/` (recordings, observations and roots, about 750 MB, including retail data), as in M4.

## What the candidate contains, and what it does not

- `65892de1` is #76's merge into the integration branch. It contains M1 to M4 (accepted), #73 (governance, `74aaa168`) and #76 (E4 and the CI repair, `65892de1`).
- **#74** (the SUPPLY ownership map, EEDCS-10) is approved at `44347ade` but **not merged**. It changes only `eech-campaign/README.md` and `docs/m5-supply-ownership.md`, which no build reads.
- **#77** (the EEDCS-11 arrival characterisation) is approved at `ab279d44` but **not merged**. Its files are documentation, retained evidence and test tools (`tools/arrival-*`, `tools/build-observed-windows.sh`). `tools/build-windows.sh` builds only `eech-dc` and `eech-world`, and no crate build script reads `tools/`. So merging it would not change the product build, though it would change the head and its `tools` tree ID.
- **`master` has moved** since the merge base `81ed32e8`: #72 adds `.claude/hooks/session-start.sh`, `.claude/settings.json`, `.gitattributes` (an eol rule for `.claude/hooks/*.sh` only) and `.github/workflows/claude-hooks.yml`. None of these is a build input. `git merge-tree --write-tree origin/master 65892de1` merges cleanly, to tree `4fb0dfa3`.

## Applicability

The identity covers `65892de1`. It extends to a later #59 head only if `git diff --quiet 65892de1 <head> -- aphavoc modules eech-campaign/crates eech-campaign/tools eech-campaign/regression eech-campaign/Cargo.lock eech-campaign/Cargo.toml` holds. Merging this record into the integration branch changes #59's head, but only under `reference/pr59-candidate-identity/`, so that check holds. Merging #77 changes `eech-campaign/tools/`, so the check would need the narrower reading above (no product build input changed); the lead decides whether that suffices.

## Limitations

These are the residuals listed in EEDCS-5 comment 10423, unchanged:
- one seed and three simulated hours; 8 evidence-only and 8 uncovered M4 claims;
- no visual Tacview review and no human-player verification (the recording is compared by hash);
- the runtime tier is local retail evidence on this PC, not hosted CI;
- the world and campaign are still coupled in one DLL;
- timing, identity and sampling limits of the observations;
- the known supply defects, and the corrected M3 attribution;
- GCC 12 stays pinned; the Wine/nm and cross-drive report limitations remain.
