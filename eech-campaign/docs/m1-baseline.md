# M1 baseline: the Windows `eech_dc` candidate

The evidence record for M1 ([#19](https://github.com/flying-dice/eech_source_code/issues/19)).
It identifies the candidate, shows what it does through the public Lua path,
and states what that evidence establishes and what it does not. Its structure
follows [`BASELINE_RECORD.md`](../../docs/governance/BASELINE_RECORD.md).

M1 prepares a candidate. The candidate is the whole engine as a Lua module,
which #19 allows as migration scaffolding. It is not a campaign-only module.

## The candidate

| | |
|---|---|
| Source | commit `a8661eaa`. That is the integration branch at `5f07697b` plus this record's checks (`lua/lifecycle.lua`, `tools/lifecycle-windows.ps1`) and the build record in `tools/build-windows.sh`. No engine or host code differs from `5f07697b`. |
| Build | `tools/build-windows.sh target/m1-candidate` (Git Bash, Docker): `x86_64-pc-windows-gnu`, release |
| Build image | `eech-build` from `tools/Dockerfile`, image `sha256:0c60c75db7b035e830c4834b1e15e096483631915bafc22c1af6aae02032bab3` |
| Toolchain | rustc 1.98.1 (48a229cea 2026-09-01); cargo 1.98.1; MinGW-w64 GCC 12 (`gcc-mingw-w64-x86-64-posix` 12.2.0-14+deb12u1+25.2+b1); Lua 5.1.5 as `lua.dll` |

The build writes `BUILD-INFO.txt` beside the binaries. At `a8661eaa` it reports no uncommitted changes and these hashes:

| File | SHA-256 |
|---|---|
| `eech_dc.dll` | `dd8c65687cde48f49fff49685cd09c480bae0b481f0b1af81bd1bb4d19f3a0e4` |
| `eech-world.exe` | `314fde3d4f7959bbf4bc27d5055c8198f8c59ef6c40f8f3cdd66684daa3eda35` |
| `lua.dll` | `d0c799fe4e1e615d335d18504d28a7b0d7c78d7f97d1754bb885b1097ac60350` |
| `lua/campaign.lua` | `3835b94fe0353e4658f6d5d7d5dcc2ffa7e8291eaee7663df59288a7c180fd03` |
| `lua/metrics.lua` | `dd28274e374e2698a84c0725e1e5ab3d5a564f13ecc0457bfce765df634d3053` |
| `lua/lifecycle.lua` | `dbce9000c176e8563c0443d4b82942b4aef332393fb95da4c1fab738f3c3f30c` |

**Rebuild check.** A second build of `a8661eaa`, from scratch in a fresh cargo target volume (`EECH_WIN_TARGET=eech-win-target-m1-clean tools/build-windows.sh target/m1-clean`), gave:

- different SHA-256 for `eech_dc.dll` (`82403592…`) and `eech-world.exe` (`e1e1e630…`), with the same sizes;
- identical `lua.dll` and scripts.

A byte comparison locates every difference:

- the link timestamps, in the PE header and in `eech_dc.dll`'s export directory;
- the header checksum;
- the COFF symbol table after the last section, which the loader doesn't map.

The code, data, read-only data and relocations are byte-identical. So the hashes above identify these particular files, not every build of the commit. A bit-reproducible build would need the timestamps and symbols removed; that isn't done here.

`tools/Dockerfile` starts from the floating tag `rust:bookworm`. Rebuilding the
image later can therefore change the toolchain. The image ID above identifies
the one used here.

### Candidate update: S2 removed (`0288ece2`)

The M1 review of the corrections ([`corrections.md`](corrections.md), #62) rejected S2, and `0288ece2` removes it. The same build (`tools/build-windows.sh target/m1-no-s2`, same image and toolchain, no uncommitted changes) now gives:

| File | SHA-256 |
|---|---|
| `eech_dc.dll` | `da72f128376e52a160e780365b235720d3b12a390e62d576e11c7d646889a975` |
| `eech-world.exe`, `lua.dll`, `lua/*.lua` | unchanged (the hashes above) |

Rerun at `0288ece2`:
- **Lifecycle checks:** unchanged, 5 PASS and 1 KNOWN DEFECT.
- **Campaign regression:** Georgia is IDENTICAL to its baseline. Lebanon's baseline is re-recorded, and 39/39 expectations hold. The before/after is in `corrections.md` under "S2 removal".

Everything else in this record was produced at `a8661eaa`, with S2 still in: the regression figures below, the rebuild check and the kernel.

## The public path

```
eech-world.exe            host process: owns the Lua 5.1 state (lua.dll)
  lua/campaign.lua        advances time, observes, records metrics; makes no campaign decisions
    require ("eech_dc")   eech_dc.dll: boot {...}, engine:frame (ms), engine:objects (), engine:clock ()
      original EECH C     campaign, AI, tasking, logistics, and the physical world
```

The script only chooses how far to advance time and when to stop. Every
tasking, supply, combat and capture decision is made inside `eech_dc.dll`.
The physical world is inside the same DLL (see "What this does not establish").

## Evidence produced for this record

Every command was run on Windows 11 (x64), except the kernel tests. The retail
installations are local and not in the repository: `tools/retail-cvh.sh` builds
Lebanon and `tools/retail-map3-installs.sh` builds Georgia.

| Check | Command | Result |
|---|---|---|
| Campaign regression | `tools\regress-windows.ps1 -Georgia <root> -Lebanon <root> -Bin target\m1-candidate -Exact` | PASS for both. Campaign expectations: Georgia 30/30, Lebanon 39/39. Both runs IDENTICAL to `regression/windows/`. The two runs took 1,140 s in parallel. |
| Lifecycle and failure | `tools\lifecycle-windows.ps1 -Root <Lebanon root> -Bin target\m1-candidate` | 5 cases PASS; 1 KNOWN DEFECT unchanged (`missing_map`) |
| Lifecycle check sensitivity | the same with a root that has no Lebanon map | 4 cases FAIL (the boots crash), so the check detects changed behaviour |
| Kernel (supporting; Linux x86-64, in the `eech-build` image) | `cargo test --locked -p eech-campaign -p eech-harness -p eech-sys` | 20 tests pass (see below) |

### The campaign regression

Retail Georgia and Lebanon ran for 3 simulated hours each (seed 1, 100 ms frames, no recording), in parallel, on 2026-09-25, at `a8661eaa` (with S2). The table shows the figures the runner reported, which matched `regression/windows/*.json` at the time. Lebanon's figures without S2, and its new baseline, are in `corrections.md` under "S2 removal".

| After 3 h | Georgia blue | Georgia red | Lebanon blue | Lebanon red |
|---|---|---|---|---|
| Task sorties (task types) | 120 (15) | 227 (18) | 163 (18) | 227 (19) |
| Aircraft sorties | 147 | 271 | 247 | 281 |
| Weapons launched | 1,179 | 526 | 3,370 | 1,917 |
| Units lost | 196 | 198 | 293 | 254 |
| Units regenerated or spawned | 46 (infantry) | 96 (infantry) | 108 | 148 |
| Keysites captured | 1 FARP | 1 FARP | 1 FARP | none |
| Airbase ammo / fuel (from 100) | 64 / 28 | 55.3 / 28.2 | 49.9 / 31.1 | 45.5 / 38.3 |

What the expectations check (`tools/campaign-expectations.py`):

- **Both campaigns:** the task types flown by each side; combat and attrition.
- **Lebanon:** regen from reserves, production at factories and refineries, and airbases resupplied with ammo and fuel.
- **Georgia:** keysite captures. Its regen interval (16.7 h) is longer than the run, and it has no producers.

The Windows baselines (`regression/windows/`) and the expectations were last
written at `12e8f692`. The candidate's identical result shows that `5f07697b`
(N3, H1, U1) left these 3-hour results unchanged.

Both the baselines and the expectations were recorded from runs of this code
base and checked against the known EECH mechanics. They were not independently
derived. An identical result shows that this build repeats that behaviour. It
does not prove fidelity to historical EECH.

## Lifecycle and failure behaviour observed

Observed through `require ("eech_dc")`, one fresh `eech-world.exe` process per
case, on retail Lebanon (`lua/lifecycle.lua`).

| Behaviour | Observed |
|---|---|
| **One boot per process** | Enforced. A second `boot` raises `eech: not valid in the engine's state`, and the first engine keeps running. |
| **Shutdown, reset, reuse** | Unsupported: no such operation exists. Setting the engine variable to nil and collecting garbage, or clearing `package.loaded` and requiring the module again, doesn't allow another boot (the same error). The limit is process-wide. |
| **A failed boot uses up the process's boot** | After an engine-rejected install root or a fatal missing campaign file, every later `boot` raises the same state error. The host has to restart the process. |
| **Argument errors in the binding** | An unknown gunship (`eech: bad argument: unknown gunship "tiger"`), a missing `install_root`, `map` or `campaign`, or a non-table config are Lua errors raised before the engine is touched. A valid `boot` afterwards succeeds. Missing fields give mlua's generic conversion message (`error converting Lua nil to String`), not an `eech:` message. |
| **Input the engine rejects** | A missing install root: `eech: bad argument: rejected by the engine`. The reason ("has no cohokum directory") goes only to stderr. |
| **An EECH fatal error** | A missing campaign file: `eech: EECH fatal error: Error opening file for reading: …`. `debug_fatal` comes back as a Lua error, not a crash. |
| **`frame` arguments** | A negative or non-numeric step is rejected by the binding, `frame (0)` is accepted, and the engine keeps running afterwards. |
| **Known defect: a missing map directory crashes the host** | An access violation in `msvcrt.dll` (reading address `0x134`) during `boot`. The process dies with `0xC0000005`, and no Lua error is raised. Reproduced with two installations. The crash reporter's call chain is invalid here: it can't walk frames through `msvcrt.dll`. |

**Not exercised:**

- A fatal error during `frame`. The engine would then be poisoned (`EngineError::Poisoned`: every later call fails), but no public input that triggers one was found.
- Threading and reentrancy: unsupported and unverified. The Rust `Engine` type isn't `Send`, which constrains Rust callers only.
- Host callbacks: none exist on this path, so there is no callback contract to characterise.

**Risks for running in-process in DCS** (inferred, not tested):

- The module imports `lua.dll` by name.
- A crash in the module ends the host process, as the missing-map case shows.
- With one boot per process, a mission restart can't boot the campaign again in the same process.
- The MinGW build uses `msvcrt.dll`, not the C runtime DCS uses.

## Kernel and reference evidence (supporting only)

This evidence belongs to the smaller campaign kernel: `crates/eech-campaign`, `eech-sys` and `eech-harness`. It exercises the kernel's Rust API, not `eech_dc` or the Lua path, so it corroborates the original C running natively but does not stand in for the candidate's evidence.

- **What it covers:** the supply-chain slice (keysite supply, the low-on-supplies response, supply task creation and assignment, up to `assign_primary_task_to_group`), replayed against the eech-core-ts corpus: 1,401 scenarios and the float fixtures (`docs/conformance.md`). The corpus's expected output comes from the 32-bit C reference, with the TSTL port's verdict recorded alongside.
- **At this candidate:** the kernel's 20 tests pass on Linux x86-64 (GCC 12.2.0, `eech-build` image): public-API behaviour (11), the corpus (3), semantic scenarios (1), findings (2), smoke (2) and a doctest. The kernel's code last changed at `f6ecff99`, so this repeats the spike's result rather than covering anything new.
  - The recorded corpus: all 1,401 scenarios give output identical to the C reference in every family: assess-group-supplies 291, entity-lifecycle 194, force-low-on-supplies 187, keysite-cargo 182, supply-task-assignment 186, supply-task-construction 189, update-timeline 172.
  - The float fixtures (float32-rtz, double-sum-rtz, crate-row, aircraft-database) also match.
  - The fresh-corpus test passes without checking anything: no fresh corpus was generated.
- **Not rerun here:** the kernel's i686 and Windows (Wine) runs, and fresh random corpora (`README.md`, "Commands").

## What this establishes

- **Identity:** the candidate is an exact commit, build configuration and set of binary hashes. A from-scratch rebuild reproduces its code and data byte for byte. The file hashes differ (timestamps, symbol table), so they identify these particular files.
- **Campaign behaviour:** through the public Lua path on Windows, the original EECH campaign ran retail Lebanon and Georgia for 3 simulated hours each. Every mechanic the expectations check occurred:
  - in both campaigns: task types flown, combat and attrition;
  - in Lebanon: regen, production, and airbase ammo and fuel resupply;
  - in Georgia: captures.

  The results were identical to the retained Windows baselines.
- **Lifecycle:** the module's lifecycle and failure behaviour as it exists, including one crash on bad input.
- **Kernel:** its conformance still holds at this commit, on Linux x86-64.

## What this does not establish

- **A campaign-only module.** The whole engine, physical world included, runs inside `eech_dc.dll`, and `eech-world.exe` only drives time and observes. That hosts can replace the world is unproven (M5).
- **Fidelity to historical EECH.** The expectations and baselines come from this code base's own runs (see above).
- **Longer runs.** Only 3 simulated hours per campaign ran at this candidate. The following were reported in the commit messages of `12e8f692` and `5f07697b` and in issue comments. They were not reproduced here and have no retained artifact in the repository:
  - retail Lebanon to a conclusion;
  - 195- and 14-hour Lebanon runs;
  - 24-hour AddressSanitizer runs.
- **Review of the corrections.** The corrections to original EECH behaviour are part of the candidate: S2 (since removed, `0288ece2`), S3, N1–N3, H1 and U1, and earlier X1, S1, C1, T1, T2, B2, W1 and E1–E3 (`docs/engine.md`). They are not classified or reviewed here.
- **Tacview.** No recording was made at this candidate. The recordings in `recordings/` come from earlier commits.
- **Linux.** The Linux baselines (`regression/*.json`) predate S2–U1 and weren't run.
- **CI.** The repository workflow currently fails before any test runs: `eech-world` can't link `lua5.1` on the runner. It gives no evidence for this candidate. Campaign runs stay local (retail data, and cost).

## Remaining M1 work and known limitations

This work is classified against #19's exit question: *can we use this candidate to develop a host while knowing exactly what it does and does not establish?* M1 can keep unsupported or unverified behaviour, as long as it is explicit.

**M1 blocker:**

1. Review and classify the behavioural corrections already in the candidate. These are S2 (since removed, `0288ece2`), S3, N1–N3, H1 and U1, and the earlier X1, S1, C1, T1, T2, B2, W1 and E1–E3. Each is either a defect correction or an intentional semantic change. #19 requires them to be visible and reviewed rather than silently inherited.

**A blocker only where an older claim contradicts this baseline:**

2. Correct claims in the older documents (`feasibility.md`, `engine.md`, `handover.md`, `recordings/README.md`) only where they materially conflict with this record. Wholesale documentation cleanup is not M1 work.

**M1 acceptance step:**

3. The explicit milestone acceptance review on #19.

**Known limitations and follow-up candidates** (recorded above, and not required for M1 to exit unless the acceptance review finds that one stops a host from safely respecting the candidate):

- **Bad installation input:** a missing map directory crashes the host, and a failed boot uses up the process's one boot.
- **Error messages:** Lua-binding errors aren't classified as `eech:` messages, and the engine's rejection reason reaches only stderr.
- **Poisoning:** after a fatal error during `frame`, it stays unverified. No such fatal was manufactured just to test it.
- **Kernel on Windows:** the supporting kernel tests weren't run on Windows.
- **CI:** the repository workflow fails before any test runs.
- **Bit-reproducibility:** a rebuild reproduces the code and data but not the file hashes (see "Rebuild check").

## Reproduce

```sh
# Git Bash, with Docker
tools/build-windows.sh target/m1-candidate          # compare target/m1-candidate/BUILD-INFO.txt with the hashes above
```
```powershell
tools\lifecycle-windows.ps1 -Root <Lebanon root> -Bin target\m1-candidate
tools\regress-windows.ps1 -Georgia <Georgia root> -Lebanon <Lebanon root> -Bin target\m1-candidate -Exact
```
