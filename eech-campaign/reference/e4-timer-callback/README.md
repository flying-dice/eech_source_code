# E4: the timer.c callback, red/green on newer GCC and the generated code

The evidence for patch E4 (`crates/eech-engine-sys/build/patches.rs`; [`docs/corrections.md`](../../docs/corrections.md),
E4) and for the `[elidable]` inventory rule ([`docs/global-state.md`](../../docs/global-state.md)). It was requested
in the technical-lead review of #76 (2026-10-20, review 5587346957). Recorded in the form of
[`BASELINE_RECORD.md`](../../../docs/governance/BASELINE_RECORD.md).

## Identity and scope

- **Task:** #75; **Epic:** #35 (M5); **PR:** #76.
- **Question:**
  - Does `timer.c:274` fail on a newer GCC without E4 and build with it?
  - Is E4 behaviour-neutral for the headless product?
- **Red revision:** `944323911adbb4d7188abe2809d8b68b9314ac52`, PR #76 before E4. It is the integration head `d2d4792a` plus the inventory tightening.
- **Green revision:** `303103d26e7107b7cf28339c7f65311b7e05b2b4`, E4. The later commits on #76 change documentation and this record only.
- **Platform:** Linux x86_64 containers under Docker Desktop 29.8.0, on Windows 11.
- **Images,** pinned by digest in [`reproduce.sh`](reproduce.sh):

  | Tag pulled | Digest | GCC | Rust |
  |---|---|---|---|
  | `rust:trixie` | `rust@sha256:e259494b…e4a` | Debian 14.2.0-19 | 1.99.1 |
  | `gcc:15` | `gcc@sha256:1fdcd041…26a` | 15.3.0 | 1.99.1 (rustup) |
  | `rust:bookworm` | `rust@sha256:93ce27a8…71e` | Debian 12.2.0-14+deb12u1 (CI's pinned `gcc-12`, tools/Dockerfile's base) | 1.98.1 |

- **Not in scope:**
  - **The campaign runtime.** No retail run was made, and none was requested for this patch (see "Validation decision" in the review).
  - **CI without the GCC 12 pin.** The pin stays, as the lead decided; unpinning is separate work.

## Reproduce

```sh
# from this directory, with git and Docker (Git Bash on Windows works)
sh reproduce.sh            # red 94432391, green 303103d2, results in ./out
```

`reproduce.sh` exports both revisions with `git archive` (`eech-campaign`, `aphavoc`, `modules`). It then runs
[`inner.sh`](inner.sh) in each pinned image, mounting the exports read-only. Every log starts with its time, GCC,
Rust and revision.

## Results

Run 2026-10-20 from 16:01:34 to about 16:25 UTC (each log's first line has its start time). All files are in [`out/`](out/).

| Step | Image (GCC) | Revision | Result | Log |
|---|---|---|---|---|
| Build `eech-engine-sys` | trixie (14.2.0) | red `94432391` | **exit 101.** `modules/system/timer.c:274:66: error: passing argument 3 of 'timeSetEvent' from incompatible pointer type [-Wincompatible-pointer-types]` is the only compile error; the two `cc: error: … '-?'` lines are cc-rs's compiler probes. The full failing command, with every flag, is in its `ToolExecError` line. | `gcc14-red-build.log.gz` |
| | trixie (14.2.0) | green `303103d2` | **exit 0.** The staged `timer.c` has 3 E4 sites; the `timer.c` command is at the end of the log | `gcc14-green-build.log` |
| | gcc:15 (15.3.0) | red | **exit 101**, the same single error at `timer.c:274:66` | `gcc15-red-build.log.gz` |
| | gcc:15 (15.3.0) | green | **exit 0**, 3 E4 sites | `gcc15-green-build.log` |
| `EECH_C_OPT=2` global-state inventory | trixie (14.2.0) | green | **2 of 2 passed**: only the two `[elidable]` globals are absent | `gcc14-inventory-O2.log` |
| | gcc:15 (15.3.0) | green | **2 of 2 passed** | `gcc15-inventory-O2.log` |
| Whole x86_64 `cargo test` (with `liblua5.1-0-dev`) | trixie (14.2.0) | green | **exit 0: 23 tests passed, 0 failed** | `gcc14-suite.log` |
| `timer.o` disassembly, i686 | bookworm (12.2.0) | red vs green | **IDENTICAL, 292 lines.** The compiler commands are identical apart from paths | `timer-i686.comparison.txt`, `timer-{red,green}-i686.{dis,norm,command.txt}` |
| `timer.o` disassembly, x86_64 | bookworm (12.2.0) | red vs green | **Differs** (87 lines before normalisation); see below. The compiler commands are identical apart from paths | `timer-x86_64.comparison.txt`, `timer-{red,green}-x86_64.*` |

**A correction to the earlier report on #76.** That report said "261 lines identical". It came from an ad-hoc compile with a hand-written subset of the flags (no `-g`, `-ffunction-sections` or `-fno-omit-frame-pointer`). This record replaces it: with cc-rs's exact commands, i686 is identical over 292 lines. An earlier run of this script failed the suite on a setup error: its export omitted `eech-campaign/scenarios/`. The export is now all of `eech-campaign`, and that run's output was discarded.

## Comparison method (generated code)

GCC 12, the CI compiler, builds `eech-engine-sys` from each revision for `i686-unknown-linux-gnu` and
`x86_64-unknown-linux-gnu`:
- **Build:** `cargo build -p eech-engine-sys --target …`, with `CC_ENABLE_DEBUG_OUTPUT=1`. This only makes cc-rs print its commands.
- **The compiler command:** the exact `timer.c` command is taken from the build script's output (`timer-<rev>-<arch>.command.txt`). Paths and hashes are normalised; red's and green's commands are otherwise identical.
- **The object:** the `timer.o` cc-rs produced is disassembled with `objdump -d --no-show-raw-insn` (`timer-<rev>-<arch>.dis`), so `.text` only, without debug information.
- **Address normalisation (`.norm`):** instruction addresses and symbol offsets are dropped, so that a shifted address does not show as a change.
- **The comparison:** `timer-<arch>.comparison.txt` compares `.dis` byte for byte, then diffs `.norm`.

## Differences and classification

**i686: none.** The generated code is identical, as expected: there `DWORD` and `DWORD_PTR` are both `unsigned int`.

**x86_64: the address-normalised diff (`timer-x86_64.comparison.txt`) has two semantic changes:**

| Function | Red (no E4) | Green (E4) | Meaning |
|---|---|---|---|
| `create_timer_event` | `mov %edx,%ecx` | `mov %rdx,%rcx` | The user data passed to `timeSetEvent` is the whole 64-bit `function` pointer, where it was truncated to 32 bits by `(DWORD) function` |
| `windows_timer_interrupt` | `mov %edx,%edx` | (removed) | `dwUser` is no longer zero-extended from 32 bits before the call through it |

**And one register-allocation change in `create_timer_event`, with no semantic effect.** The loop counter moves from `%edi` to `%r8d`. Red therefore saves `delay` in `%r9d` and restores it before the call (`mov %edi,%r9d` … `mov %r9d,%edi`), and green has no need to. The arguments passed to `timeSetEvent` are otherwise the same.

**Classification: no behavioural change, for the current headless product.**
- `timeSetEvent` resolves to the headless shim, which ignores its callback and user data and returns 0:
  - on Linux, `csrc/eech_win32.c`;
  - on Windows, the same function as `eech_w32_timeSetEvent`, through the force-included `csrc/eech_win32_names.h`.
- So neither changed instruction affects anything observable.
- Against a real Windows multimedia timer, the same two changes would be a **defect correction (portability)**. That is not claimed here (the lead's review of #76).

**The `[elidable]` inventory rule.** With E4, GCC 14 and 15 at `-O2` omit no global beyond the two `[elidable]` ones. The test is unchanged by E4, and the rule needs no new entry.

## Limitations

- **Linux only.** The Windows (MinGW) object was not disassembled. On Windows the call is renamed to the same no-op shim (`csrc/eech_win32_names.h`, force-included by `build/main.rs`).
- **Engine and x86_64 suite only.** The newer-GCC runs build `eech-engine-sys` and run the x86_64 suite and the `-O2` inventory. i686, windows-cross and clippy were not run on GCC 14 or 15.
- **No retail runtime in this record.** The rebuilt Windows candidate's retail run is a separate record: [`../pr76-candidate-validation/`](../pr76-candidate-validation/).
