# #59 candidate identity and validation: integration `65892de1` against windows-retail v1

This record identifies the build of the integration candidate that #59 would promote to `master`. It answers item 2 of the lead's #59 review
(5587358297) and the decision in EEDCS-5 comment 10423: "establish final integration build identity against #76's validated DLL/host/Lua/scripts/configuration. Reuse the accepted runtime evidence only when that identity is demonstrated; otherwise rerun the applicable M4/runtime exact checks."

It follows the form of [`BASELINE_RECORD.md`](../../../docs/governance/BASELINE_RECORD.md) (phase table, reproduction and rollback, next question: see the end). The #76 record it compares against is [`../pr76-candidate-validation/`](../pr76-candidate-validation/).

## Result

- **Source and configuration: identical.** The candidate `65892de1` differs from #76's validated commit `d38b9b2d` only by #76's validation record (18 files under `reference/pr76-candidate-validation/`), which no build or runner reads.
- **Toolchain: identical.** The image, rustc, cargo, MinGW GCC and Lua are the same.
- **Lua runtime and scripts: identical.** `lua.dll` and the four `lua/*.lua` files of the build output (copied from `eech-campaign/crates/eech-world/lua/`) have the same SHA-256.
- **`eech_dc.dll` and `eech-world.exe`: different file hashes; code and data the same apart from link metadata.** A from-scratch rebuild changes the link timestamps (COFF header, and the DLL's export directory), the header checksum and the unmapped COFF symbol table, as `eech-campaign/docs/m1-baseline.md` ("Rebuild check") already records. Every mapped section is byte-identical except the DLL's `.edata`, whose only differing bytes are the export directory's timestamp. (This record's first commit message said every mapped section was identical; that was wrong, and is corrected here.)
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

[`compare-pe.py`](compare-pe.py) compares the two builds region by region. [`section-comparison.txt`](section-comparison.txt) holds the commands, the full SHA-256 of every section of both builds, the differing bytes by region and the exit status (0 for both files). The tool first rejects a file whose layout is ambiguous (an optional header too short for its checksum or its declared data directories, a section table or section past the end of the file, raw data overlapping the headers or another section, sections overlapping in memory), then checks that the two layouts agree, with section names compared as raw bytes, and that each expected region is what it claims to be: the export directory lies in the raw data of a section without code (`.edata` here), and the COFF symbol and string table holds symbols, starts where the last section ends and runs to the end of the file.

| | `eech_dc.dll` | `eech-world.exe` |
|---|---|---|
| Size | 16,657,810 bytes, both | 1,902,960 bytes, both |
| `.text`, `.data`, `.rdata`, `.pdata`, `.xdata`, `.idata`, `.CRT`, `.tls`, `.reloc` | byte-identical | byte-identical |
| `.edata` | hash differs; the only differing bytes are the export directory's timestamp (2 bytes) | (none) |
| COFF `TimeDateStamp` | 2 bytes | 2 bytes |
| Optional-header `CheckSum` | 2 bytes | 2 bytes |
| COFF symbol/string table after the last section (not mapped by the loader) | 115,428 bytes | 22,472 bytes |

This is the same pattern as the M1 rebuild check (`eech-campaign/docs/m1-baseline.md`), which the #60 review accepted as not an M1 blocker. It is strong evidence that the two builds execute the same code. It does not make the file hashes equal, and the reuse condition in #76's record is stated on the file hashes. So the runtime checks were repeated rather than inferred.

**Reproducing it.** The binaries are local build outputs and are not committed. The #59 candidate's are under `target/pr59-candidate/` of this record's worktree, and #76's under `target/pr76-candidate/` of #76's worktree on the same Windows PC (the path is in `section-comparison.txt`). Elsewhere, rebuild both commits with `tools/build-windows.sh` and run the commands in `section-comparison.txt`. A rebuild reproduces neither the file hashes nor every section hash: the DLL's `.edata` hash changes with the export timestamp inside it. What a correct rebuild reproduces is the tool's verdict, exit 0: every differing byte lies in the documented metadata fields or the validated symbol table, and every other section hash is the same.

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
| Other EECH processes | Reported by the session that ran the pack: it waited until another session's `eech-world.exe` runs (own roots and binaries) had ended. That check was not recorded, so this record does not evidence it. |
| Time | 2026-10-20, first pack output 20:10:37 UTC, last 20:33:32 UTC (the report's "run 21:33:32" is local BST), from the output files' times in [`provenance-checks.txt`](provenance-checks.txt). |
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
| Metrics files | `regress/georgia_retail.json` `74276605…9148d`, `lebanon_retail.json` `fa09d562…62587`: the same file hashes as #76's run; parsed, equal to `regression/windows/*.json` (the committed files differ only in serialisation). Commands and outputs in [`provenance-checks.txt`](provenance-checks.txt). |

(#76's README abbreviates the recording hash as `3bd17b64…d2d4`; the full hash in its `logs/reference.txt` ends `…d2d19b`, the same as here.)

### Against #76's run

Every claim status and every check status are the same as #76's. [`comparison-report.txt`](comparison-report.txt) and [`comparison-logs.txt`](comparison-logs.txt) list every value and file that differs, in full; both tools exit 1 because differences exist. Each difference is classified as identity or environment, none as a result:

| Where | Differs | Class |
|---|---|---|
| `report.json` `head.pack_revision`, `head.build.build_info[0]` | `d38b9b2d` against `65892de1` | identity: the commit |
| `report.json` `head.build.files` (2) | the two binary hashes | identity: the build (see above) |
| `report.json` `head.build.path`, `head.command` (`--bin`, `--out`) | `pr76-candidate` against `pr59-candidate`, and the worktree | environment: output paths |
| `report.json` `head.when` | 17:49:08 against 21:33:32 (local) | environment: run time |
| `report.json` `checks` `inputs.*.detail` (3) | only the worktree path of the manifest compared against; root, file count, bytes and combined SHA-256 equal | environment: path |
| `chains.json` `metrics`, `observations`; `loss-chain.json` `observations` | the worktree path of the input files | environment: path |
| `logs/reference.txt` | the commit line, and `runs took` 1,286 s against 1,025 s | identity and wall time |
| `logs/regress.txt` | `runs took` 1,232 s against 971 s | wall time |
| `logs/inputs-before-run.txt` | only in this record (the pre-run input check #76 did not keep) | additional evidence |

`comparison-logs.txt` masks the two runs' worktree (`agent-a…`) and candidate (`pr59-candidate`, `pr76-candidate`) names before comparing; the masks are printed in it.

### Differences and classification

**None** against windows-retail v1. Every exact comparison is IDENTICAL and every check passes, so there is nothing to classify and no change record. No baseline was re-recorded, and no tolerance or expectation was changed.

### Retained here

- `report.md`, `report.json`, `output.txt` (the console output), `logs/`, `chains.json` (reference run 1) and `loss-chain.json`, as in #76's record.
- `BUILD-INFO.txt`, `section-comparison.txt`, `provenance-checks.txt`, and the three comparison scripts with their outputs, the rerun of those outputs with the current helpers (`helper-rerun.txt`) and their selftest (`compare-selftest.py`, with `selftest.txt`, `selftest-red-1783ae3b.txt` and `selftest-red-f3f43b89.txt`).

The full outputs stay local under `target/regression-pack/pr59-candidate/` (recordings, observations and roots, 750 MB by `du -sh`, including retail data), as in M4. They are not committed.

## What the candidate contains, and what it does not

- `65892de1` is #76's merge into the integration branch. It contains M1 to M4 (accepted), #73 (governance, `74aaa168`) and #76 (E4 and the CI repair, `65892de1`).
- **#74** (the SUPPLY ownership map, EEDCS-10) is approved at `44347ade` but **not merged**. It changes only `eech-campaign/README.md` and adds `eech-campaign/docs/m5-supply-ownership.md` (in #74, not at this head), which no build reads.
- **#77** (the EEDCS-11 arrival characterisation) is approved at `ab279d44` but **not merged**. Its files are documentation, retained evidence and test tools (`eech-campaign/tools/arrival-*`, `eech-campaign/tools/build-observed-windows.sh`, in #77, not at this head). `tools/build-windows.sh` builds only `eech-dc` and `eech-world`, and no crate build script reads `tools/`. So merging it would not change the product build, though it would change the head and its `tools` tree ID.
- **`master` has moved** since the merge base `81ed32e8`: #72 adds `.claude/hooks/session-start.sh`, `.claude/settings.json`, `.gitattributes` (an eol rule for `.claude/hooks/*.sh` only) and `.github/workflows/claude-hooks.yml`. None of these is a build input. `git merge-tree --write-tree origin/master 65892de1` merges cleanly, to tree `4fb0dfa3` (output in [`provenance-checks.txt`](provenance-checks.txt)).

## Applicability

The identity covers `65892de1`. It extends to a later #59 head only if `git diff --quiet 65892de1 <head> -- aphavoc modules eech-campaign/crates eech-campaign/tools eech-campaign/regression eech-campaign/Cargo.lock eech-campaign/Cargo.toml` holds. Merging this record into the integration branch changes #59's head, but only under `reference/pr59-candidate-identity/` and in `.github/workflows/eech-campaign.yml` (the `evidence-selftest` job below), so that check holds. Merging #77 changes `eech-campaign/tools/`, so the check would need the narrower reading above (no product build input changed); the lead decides whether that suffices.

## Limitations

These are the residuals listed in EEDCS-5 comment 10423, unchanged:
- one seed and three simulated hours; 8 evidence-only and 8 uncovered M4 claims;
- no visual Tacview review and no human-player verification (the recording is compared by hash);
- the runtime tier is local retail evidence on this PC, not hosted CI;
- the world and campaign are still coupled in one DLL;
- timing, identity and sampling limits of the observations;
- the known supply defects, and the corrected M3 attribution;
- GCC 12 stays pinned; the Wine/nm and cross-drive report limitations remain.

## Comparison helpers and their selftest

`compare-pe.py`, `compare-report.py` and `compare-logs.py` exit 0 when the inputs are the same (or, for `compare-pe.py`, differ only in validated link metadata and the symbol table), 1 when they differ, and 2 on bad arguments, when an input cannot be read (for `compare-pe.py`, also a PE file whose layout is ambiguous) or, for `compare-logs.py`, when there is nothing to compare. `compare-report.py` prints every differing value in full and `compare-logs.py` every differing text line (a non-UTF-8 file only as the same bytes or differing); `compare-pe.py` prints every section's full hash and the count of differing bytes per region, not the bytes themselves. `compare-logs.py` reports a symbolic link or a Windows junction and never enters it; it recognises a junction by `os.lstat`'s `st_reparse_tag` (Python 3.8 and later). The helpers and selftest are run with Python 3.10 (WSL), 3.12 (hosted CI, `ubuntu-24.04`) and 3.13 (Windows); older versions are untested.

[`compare-selftest.py`](compare-selftest.py) builds small synthetic inputs (minimal PE32+ and PE32 files, JSON files and log trees), runs each helper under `python -I -X dev` (each `compare-pe.py` case under `-O` as well) and checks each case's exit status, the verdict or diagnostic it prints, and that stderr holds no traceback or warning, only the expected usage or argument message where there is one. It needs no retail data or binaries. A case that cannot run on a platform is counted and named, not dropped: the two junction cases are Windows-only, and the unreadable-directory and unreadable-file cases need POSIX permissions, so each platform skips those by design; a symbolic-link case on Windows without the privilege is "unavailable", and `--strict` fails the run on any unavailable case. Hosted CI runs it with `--strict` in the `evidence-selftest` job of `.github/workflows/eech-campaign.yml` (Ubuntu 24.04), so the symbolic-link and permission cases are required there; the junction cases run only on Windows, on this PC. [`selftest.txt`](selftest.txt) holds the runs: Windows, Python 3.13.6, 115 of 115 attempted cases as expected, 2 skipped by design and the 2 symbolic-link cases unavailable (so `--strict` exits 1 there); WSL Ubuntu, Python 3.10.6, `--strict`, 117 of 117, the 2 junction cases skipped by design.

**Red, against the helpers of `1783ae3b`.** This selftest was written first and, before any helper changed, run against the helpers of `1783ae3b`; the retained run is its final version against a copy of those helpers: 27 of 115 attempted cases as expected, 88 not ([`selftest-red-1783ae3b.txt`](selftest-red-1783ae3b.txt)). The defects it shows, each with its own case: `compare-pe.py` and `compare-logs.py` left files unclosed (a `ResourceWarning` on every run that read a file); `compare-pe.py` excused a changed `.text` byte as the export timestamp when two sections' raw data overlapped, excused a section-name byte as the checksum when the optional header was too short to hold it, compared section names after a lossy decoding, and accepted section data inside the headers, sections overlapping in memory and more data directories than the optional header holds; `compare-report.py` took 121.7 s in the retained run on 100,000 distinct keys (the case allows 10 s), as its duplicate check was quadratic; and `compare-logs.py` entered a Windows junction and compared the target's files although it printed "LINK, not followed or compared". Rerun with the corrected helpers, the retained comparisons of the real binaries, reports and logs are unchanged byte for byte, with nothing on stderr under `-X dev` ([`helper-rerun.txt`](helper-rerun.txt)). One defect of the repair itself was caught by the selftest on Linux: the first junction check used a `stat` constant that Linux Python lacks, and every `compare-logs.py` case failed with a traceback until it was replaced.

This record's first versions of the helpers (commit `f3f43b89`) failed open, and the previous selftest (at `1783ae3b`) fails 64 of its 85 cases against them ([`selftest-red-f3f43b89.txt`](selftest-red-f3f43b89.txt)): `compare-report.py` exited 0 whenever both files were readable JSON, whatever they contained, cut values at 110 characters and treated null, absent and empty values alike; `compare-logs.py` always exited 0, ignored files present only in its second directory and cut long diffs; and `compare-pe.py` accepted any bytes after the last section as the symbol table and, under `python -O`, compared files without a PE header. Its earlier output gave the verdict "same code and data; only link metadata and the unmapped symbol table differ" without naming the export timestamp inside `.edata`. Rerun on the same inputs, the corrected helpers reach the same conclusions (above). The one difference the old helpers hid here is `logs/inputs-before-run.txt`, which exists only in this record; the rest of their findings are unchanged.

**Corrections to `1783ae3b`.** Its commit message and this README said that removing any new `compare-pe.py` guard made the selftest fail 2 cases. No per-guard command or output was retained, so that claim is withdrawn; the red run above is the evidence instead. The same commit message said the helpers "print differences in full": that holds for `compare-report.py` and `compare-logs.py`, but `compare-pe.py` prints a count of differing bytes per region, not each byte.

## Execution evidence

| Phase | Exact command and revision | Actual result | Retained evidence |
|---|---|---|---|
| Baseline characterisation | #76's run at `d38b9b2d` | PASS | `../pr76-candidate-validation/` |
| Meaningful red check, or justified not applicable | not applicable to the campaign: no behaviour changes. For the helpers: `python -I compare-selftest.py` against the `f3f43b89` helpers, and the current selftest against the `1783ae3b` helpers | 64 of 85 cases fail; 88 of 115 attempted cases fail | [`selftest-red-f3f43b89.txt`](selftest-red-f3f43b89.txt), [`selftest-red-1783ae3b.txt`](selftest-red-1783ae3b.txt) |
| Green candidate / sociable tests | the pack command above, at `65892de1`; `python -I compare-selftest.py` (with `--strict` on Linux); the retained comparisons rerun with the current helpers | REGRESSION PACK: PASS; 115 of 115 on Windows, 117 of 117 on Linux; reruns identical | `output.txt`, `report.md`, `report.json`, `logs/`, [`selftest.txt`](selftest.txt), [`helper-rerun.txt`](helper-rerun.txt) |
| Post-change boundary assertion | not applicable: no boundary change | | |
| Applicable regression/conformance | Georgia and Lebanon `-Exact` against windows-retail v1 | IDENTICAL | `logs/regress.txt`, `report.json` |
| Integrated public Lua/module path, if affected | the pack's lifecycle and reference tiers | PASS; missing_map the known defect | `logs/lifecycle.txt`, `logs/reference.txt` |

## Reproduction and rollback

Rebuild with the build command above at `65892de1`, run the pack command, then the comparison commands recorded at the top of `section-comparison.txt`, `comparison-report.txt` and `comparison-logs.txt`. Reproducing it needs the retail roots, whose manifests are in `reference/m4-s3-mutation/` and `reference/lebanon-3h/`; retail data and the built binaries are not committed. Rollback: revert this record's commits (files under this directory, and the `evidence-selftest` job).

## Next bounded question

Does this record, with its corrected helpers, establish #59's candidate identity at `65892de1`? #59's final head after the landings it lists needs its own provenance.
