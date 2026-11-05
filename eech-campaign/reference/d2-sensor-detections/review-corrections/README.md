# EEDCS-43 review corrections: evidence (26 Oct 2026)

Corrections to the findings of the independent source review of `37989fa5f` and of its follow-up on `5c00ab4fa`; the findings themselves, with the lead's dispositions, are in
[`docs/handover/D2-SENSOR-DELIVERY-REVIEW.md`](../../../../docs/handover/D2-SENSOR-DELIVERY-REVIEW.md), the account in
[`docs/m5-sensor-characterization.md`](../../../docs/m5-sensor-characterization.md) ("Review corrections"). Development stacking on the unaccepted EEDCS-42; nothing here is acceptance.
No retail asset, installation root or binary is in Git; the roots and binaries named here are private copies under `R:/d2-evidence/sensor-characterization-baseline-20261026/`.

| Commit | What | Evidence |
|---|---|---|
| `647558f8b` | P1: `modules/` and `aphavoc/source` registered as build inputs of `eech-engine-sys` | `p1-incremental-check.{sh,log}` |
| `5c00ab4fa` | P2: generic `not_perceived` rejection reason, accepted for every scan; the four EECH predicates stay as reference diagnostics; comments describe transitional mirrors | `p2-providers/`, `checks/5c00ab4fa-*`, `builds/*-5c00ab4fa-*` |
| `a2a041fec` | follow-up P2: the sensor fixture observes `not_perceived` instead of stopping on it | `p2-fixture/`, `checks/a2a041fec-*`, `builds/*-a2a041fec-*` |

All builds: the repository `tools/Dockerfile` image in WSL2 `EECH-D2-Build`, rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`, the unchanged `tools/build-windows.sh` and `tools/build-windows-sensor-fixture.sh`, each in a fresh private cargo volume
(`eech-win-target-d2-sensor-{shipping,fixture}-4` for `5c00ab4fa`, `-5` for `a2a041fec`), 0 uncommitted build inputs. (The `image:` id in BUILD-INFO differs between builds because the image is rebuilt from stdin each time; the toolchain versions it prints are the same.)
Two earlier builds of `5c00ab4fa` are not evidence: one reported 1 uncommitted input (a log file written into the tree) and the other was refused for its output directory; their logs are kept outside Git.

## P1: original sources are build inputs (`647558f8b`)

`p1-incremental-check.log`: a never-cleaned cargo target and a scratch copy of the original tree. With the reviewed `build/main.rs`, editing the original `mb_hostsense.c` does not rerun the build script and the staged copy keeps the old SHA (RED).
With the corrected script an edit of that `.c`, of its `.h` and of `modules/3d/textname.h` each rerun it and the staged file equals the edit; an unchanged build does not rerun it (GREEN). A source-watch change alone alters no runtime.

## P2: a host can say "not perceived" (`5c00ab4fa`)

Public Lua -> `eech_dc` -> host path, ordinary-feature shipping build (`eech_dc.dll` `88a862ea...ac2e`, `eech-world.exe` `7fec4c99...962e`), Lebanon, 300 s, seed 1, fresh private root per mode (manifest-identical source), one native process at a time while the lead's work ran
(`p2-providers/`, exits in `run-exits.log`: 0 after 254 s and 260 s). One run each: a demonstration, not statistics.

| Provider | begin | candidates | draws | signatures | accepts | weapon births | losses |
|---|---|---|---|---|---|---|---|
| reference host (retained, `extraction/providers/`) | 16,840 | 277,652 | 174 | 5,341 | 356 | 185 | 6 |
| every candidate `reject` with a scan-specific EECH reason (`blocked`, retained) | 17,047 | 324,946 | 0 | 0 | 0 | 176 | 1 |
| **every candidate `reject` with `not_perceived`** | 17,047 | 324,946 | 0 | 0 | 0 | 176 | 1 |
| **a host model of its own** (`own_model`: perceived inside a plain 4,000 m horizontal distance from the positions it was given, that distance as the measured range) | 16,914 | 285,097 | 0 | 0 | 199 | 180 | 9 |

* `not_perceived` was accepted for all four scan kinds (air-to-air 6,245, air-to-surface 312,160, surface-to-air 520, surface-to-surface 6,021 consultations, every one rejected with it) and produces the same campaign behaviour as the scan-specific rejections: the campaign acts on the verdict, not its reason.
* `own_model` is never asked for a draw or a signature, uses no EECH predicate, receives no seed and no entity index, and changes what the campaign does (199 detections, 180 births, 9 losses against 185/6 and 176/1): the host's observations are used. Which targets are candidates, their order, ranking and engagement are still the campaign's.
* Unchanged and not re-run: the reference-host behaviour (the retained 900 s whole-campaign Tacview control of `379`, raw-equal to the supplied baseline), the 600 s witness, and the retained reference/blocked/far/malformed runs. The malformed-reason control (`bad_reason`) still refuses an unknown reason; only its message text now lists `not_perceived` as well (the retained record keeps the older text).
* Checks at `5c00ab4fa` (cargo's own exits, fresh clippy volume): `cargo fmt --all --check` 0; clippy `-D warnings` 0 for default and for sensor-fixture; `cargo test --locked --workspace` 0 (3+2, 30, 165, 19, 6, 2, 1 tests pass).

## Follow-up P2: the fixture observes it (`a2a041fec`)

Red (`p2-fixture/red-5c00ab4fa-*`): the `5c00ab4fa` fixture build (`eech_dc.dll` `94d23b40...8376`, unchanged DLL) run with the new `sensor-characterization.lua sense=not_perceived`, 60 s: native exit 1 after 6 s with
`debug_fatal: host sense: the sensor fixture trace has no verdict for host reason 5` — a valid shipping provider poisoned the fixture build.

Green (fixture build `a2a041fec`, `eech_dc.dll` `e5f3674b...6ef3`; `p2-fixture/`): the host's verdict is counted apart (`sensor-host-verdicts` rows and a `<scan>/host_not_perceived` sample), folds into neither hash, and the reference-only hooks of that exchange are skipped.

| Case | Native exit | Result |
|---|---|---|
| `sense=not_perceived`, 60 s, probe at 20 s | 0 after 54 s | 1,339 surface-to-surface generic verdicts; the verdict table (accept, range, range_adjusted, altitude, tolerance, line_of_sight) all 0; probe and final report complete |
| `sense=own_model`, 60 s, probe at 20 s | 0 after 56 s | 1,193 generic verdicts and 40 accepts (surface-to-surface); no other verdict column moved |
| **`sense=not_perceived`, 300 s, probe at 150 s** | 0 after 268 s | generic verdicts air-to-air 6,245, air-to-surface 312,160, surface-to-air 520, surface-to-surface 6,021 — equal to the shipping run's consultations; every table verdict 0 |
| reference host (no `sense=`), 45 s, probe at 30 s | 0 after 41 s | **no** `sensor-host-verdicts` row and no `host_not_perceived` sample; the 83 probe and run-at-probe events at 30 s are byte-identical to the retained original's (`out/lebanon-probe`), sorted-text SHA-256 `0744d56d...fb27` for both |

The reference comparison is an ad-hoc text comparison of the events at `at = 30` (the comparator has no time filter), not a run of `tools/sensor-trace-compare.py`; the strict 443-event oracle, its hashes (`d60c462829c9714e`, `2611b04d99f5c689`) and its expected counters were not run again and not changed.
The reference path of the fixture differs from `5c00ab4fa` only by `if (!host_verdict (...))` guards that are true for every reference answer.

Shipping unchanged by it: `a2a041fec`'s shipping `ac_tgt.o` and `vh_tgt.o` are byte-identical to `5c00ab4fa`'s and `mb_hostsense.o` differs only in four `debug_fatal` line-number immediates (`0x148`..`0x14e` -> `0x14f`..`0x155`) from seven added fixture-only lines
(`checks/a2a041fec-*objects.log`, `*disassembly-diff.log`); the shipping `eech_dc.dll` is `ab42d053...968b`. Checks at `a2a041fec`: fmt 0; clippy `-D warnings` 0 for default, all fixtures, old-guard plus sensor, sensor alone; the symbol check shows 0 fixture symbols in the shipping objects.

## Not claimed

The corrected fixture was run on Lebanon for at most 300 s with host models that are mine (a rejecting one and a distance one): a boundary and fixture demonstration, not a DCS host. A full pack, the 900 s control, Georgia, player-controlled targets and visual Tacview inspection were not repeated.
The final shipping build, the full regression pack and the reviewer's follow-up on this delta are the lead's.

## Representation

Traces, summaries and logs written by native Windows processes are CRLF files; Git stores LF text, so the copies here are the LF representation of the same content (as for `extraction/traces/`). The raw files are in the private evidence tree
(`out/p2-providers`, `out/p2fix-*`, `logs/`); the byte comparison of the 83 events at the 30 s probe was made on the raw files.
