# Feasibility: the original EECH campaign C as an embeddable native engine

This is the report of the spike. It asked whether the original EECH campaign C
can become an embeddable native campaign engine, with Rust providing the
architectural boundary, instead of porting the campaign C → TypeScript → Lua.

**Verdict: yes.** The spike did not find anything that blocks the approach. It
found one actual 64-bit blocker, and fixed it with a two-line patch. It found
one Windows toolchain issue, and resolved it with a compile flag. It found and
fenced several global-state facts. The first vertical slice runs behind a safe
Rust API. It reproduces the original 32-bit C reference byte for byte on the
whole eech-core-ts corpus, and on 7,000 fresh random scenarios, on three
platforms.

## What was built

| Deliverable | Where |
|---|---|
| Native build of the original campaign C: `cargo build` compiles it and needs no separate C project | `crates/eech-sys/build` |
| Private FFI boundary (no EECH type crosses it) | `crates/eech-sys/csrc/eech_kernel.h`, `crates/eech-sys/src` |
| Public, safe API: `Campaign`, `CampaignConfig`, `World`, `CampaignEvent`, `CampaignSnapshot`, `EntityId`, `CampaignError` | `crates/eech-campaign` (`#![forbid(unsafe_code)]`) |
| Headless harness: JSON scenario → `HeadlessWorld` → `Campaign` → JSON result | `crates/eech-harness`, `scenarios/` |
| Corpus: the eech-core-ts scenarios, with the original C's output and TSTL's verdict | `corpus/`, `tools/corpus` |
| Differential, behavioural, platform and findings tests | `crates/*/tests`, CI `.github/workflows/eech-campaign.yml` |

## The milestone, point by point

1. **A Rust `Campaign` facade exists.** See `crates/eech-campaign/src/campaign.rs`.
2. **Rust owns the lifecycle.** `Campaign::new` opens the kernel, which fully resets the C state. `Drop` closes it and frees everything EECH allocated: the kernel's allocator is an arena. One campaign runs at a time, enforced with a process token (`docs/global-state.md`).
3. **Original EECH C executes behind it.** The kernel compiles 48 original translation units unchanged, roughly 40k lines, plus about 4k lines of verbatim extracts carrying `#line` provenance. Two units are patched with exact-text patches (`docs/patches.md`). The native host layer adds about 3k lines of C and about 2k lines of runtime Rust. The legacy replay adds another 2k lines of C, but it is test tooling. So the campaign logic that runs is original code.
4. **Environmental dependencies go through the new host boundary.**
   - `World::position` answers EECH's `VEC3D_TYPE_POSITION` reads of aircraft.
   - `World::object_bounds` answers `get_object_3d_bounding_box`.
   - The frame delta is an argument of `step`.
   - The transport and the campaign screen come back as `CampaignEvent`s.
5. **A real campaign behaviour executes.** `Campaign::step` runs EECH's own update loop, and the supply chain emerges from it:
   - keysites consume supplies;
   - a FARP runs low;
   - its force finds a factory crate;
   - a supply mission is constructed and waits at the airbase;
   - three minutes later the airbase's assignment pass selects the idle, registered medium lift group.

   The step then stops at the slice boundary, `assign_primary_task_to_group`, with the group's and the task's ids (`CampaignError::Boundary`). See `tests/campaign.rs` and `scenarios/supply-chain.expected.json`.
6. **The harness observes the result.** The output is stable JSON: events per frame, the outcome, the snapshot, and the world queries.
7. **The result agrees with the C reference.** The recorded corpus has 1,401 scenarios and 5,000 float operations; the fresh random corpus has 7,000 scenarios and 28,000 float operations. On every one of them, the native module's output is byte-identical to the canonical 32-bit C reference harness's output. That holds on x86-64 Linux, i686 Linux and x86-64 Windows, run under Wine (`docs/conformance.md`).
8. **The result also agrees with TSTL.** The TSTL port agrees with the C on every corpus scenario, so all three implementations agree. The exporter records that verdict per scenario.

One gap remains. The `step` composition itself (`ks_updt.c`, `ts_updt.c`) has no counterpart in the C reference or in TSTL, because neither ports the keysite update yet. It is verified in three ways instead:
- it is original code, compiled unchanged except for patch P2;
- every operation it composes is corpus-verified;
- its recorded results are identical on i686, x86-64 and Win64, and with the original i386 va_list idiom.

## The questions the spike was asked

### Is EECH `observe → step → commands`, or does it need callbacks?

**It needs callbacks, and it needs them for queries only.** EECH pulls physical state at the moment its logic needs it. For example:
- `assess_group_task_locality_factor` reads the first member's position in the middle of a task assignment;
- `update_keysite_cargo` reads the crate model's bounds in the middle of laying out a crate row.

These are synchronous, answer-now questions. So `World` is a query port that the campaign calls during `step`. Everything the campaign does outwards is a push, and never needs an answer:
- replication messages;
- the campaign screen;
- the force's low-on-supplies notification.

Those pushes are collected and returned as events. The resulting API shape is therefore:

```
step(&mut world, dt) -> events          world: synchronous queries (&self)
```

It is not `observe → step → commands`. No `WorldCommand` exists yet, because no C call site in the slice produces one. The first will be the hand-over of a mission to a physical group, which happens inside `assign_primary_task_to_group` (the slice boundary, eech-core-ts 6b). `docs/ports.md` derives the boundary call site by call site.

### How much must change in the C?

**Almost nothing.**

| Change | Why | Kind |
|---|---|---|
| P1 `en_creat.c`: marshal the variadic attribute list | 64-bit blocker B1 | patch (4 lines) |
| P2 `ks_updt.c`: `static float task_timer` becomes a named global | leaks between campaign instances (G1) | patch (2 lines) |
| `-Dcreate_supply_task=…` on `fc_msgs.c` | observation point (replaces the C reference's `ld --wrap`, which does not port) | compile definition |
| `-Dget_free_entity=…` on `en_heap.c` | entity identity: generations for `EntityId` | compile definition |
| `-UWIN32` for GCC-family Windows builds | `#define ai_log();` only works with MSVC's preprocessor (T1) | compile flag |
| `-ftrivial-auto-var-init=zero` on the `taskgen.c` extract | F1, the compatibility decision the C reference and TSTL share | compile flag |

The rest of the work is environment: tables, allocator, transport, host calls, and fail-loud stubs. All of it lives in `csrc/`, not in EECH files.

### 64-bit

See `docs/64-bit.md`.
- **One actual blocker (B1).** `en_creat.c` reinterprets a `va_list` as the i386 argument stack. On x86-64 System V this reads the va_list descriptor (probed: the attribute list reads back as 16 and 48, the descriptor's offsets). On Win64 it reads 8-byte slots as 4-byte items.
- **The closure is otherwise clean.** It compiles on LP64 and LLP64 with no pointer/int conversions, and pointer/int casts are build errors.

### Floating point

See `docs/fpu.md`.
- The campaign runs rounding toward zero, as EECH does. Each entry sets that mode and restores the host's mode afterwards, and host callbacks run under the host's mode.
- The results do not depend on the C optimisation level (`-O0` to `-O3`), on the compiler (gcc or clang), or on the platform (i686, x86-64, Win64).
- The corpus does detect the rounding mode: under round-to-nearest, 200 recorded and 1,198 fresh scenarios differ from the C reference.

### Global state

See `docs/global-state.md`.
- One campaign per process at a time is enforced (`CampaignError::AlreadyRunning`).
- Sequential campaigns are supported and proven identical.
- Concurrent campaigns are not supported, and the API does not pretend otherwise.
- Every writable global of the kernel is classified in a checked-in inventory. A test compares the inventory with `nm`, so a new global fails the build's tests until someone decides what it is.

### Vendoring

See `docs/closure.md`. The kernel's closure is 565 original files: 75 `.c`, of which 48 are compiled whole and the others only have verbatim extracts, and 490 headers, mostly whole original headers. The build generates the list (`closure.txt`, the compiler's own dependency scan).

The spike builds from the monorepo's original tree rather than copying it. That keeps a single source of truth, and it keeps the C reference and the native module on the same bytes. Vendoring for a standalone crate means copying `closure.txt`'s files, which is a mechanical step.

### DCS

See `docs/dcs.md`. The key facts:
- DCS is Win64 Lua 5.1, and Rust native Lua modules are established practice there (for example mlua with the `module` feature).
- The module keeps the host's floating-point environment intact.
- Some EECH paths are unguarded NULL dereferences. In process, one of those would take DCS down with it, so the integration needs either preconditions validated in the facade or an out-of-process campaign. The API allows both.

## Risks and open questions

- **Unguarded NULL dereferences.** EECH's release code dereferences NULL on some inputs. The corpus reaches 24 of them in slice 1: a group without members asks for its position. In process, such a path is a crash. The facade must make these inputs unrepresentable (`CampaignConfig` validation) or run the campaign out of process. It must never catch the fault in a host process. The replay's NULL-page handler exists only in the dedicated replay process.
- **The slice boundary.** The next slices, 6b and 6c (assignment, route, guide, members), will bring the first `WorldCommand`s: the campaign handing routes to physical groups. They will also bring the `LandingObservation`-style observations eech-core-ts anticipates.
- **Hand-written rows.** The list storage of the session, guide and aircraft is still hand-written, exactly as in the C reference (`csrc/eech_tables.c`). Compiling `ss_list.c`, `gd_list.c` and `ac_list.c` retires them.
- **MSVC.** The build is written for MSVC (`cc` flags are conditional), but only MinGW was exercised. F1's zero-initialisation has no MSVC flag, so an MSVC build needs a source patch or a different decision.
- **Performance.** Not measured seriously. 60,000 frames of the supply chain take 34 ms in a release build with the C at `-O0`.

## Recommendation

Adopt the native module as the implementation strategy for the campaign core. Keep eech-core-ts's C reference and corpus as the behavioural authority's executable form.

The next steps, in order:
1. Grow the slice exactly as eech-core-ts planned: 6b (assignment transaction) introduces the first `WorldCommand`, and the landing handlers introduce observations. Each slice adds original translation units to `spec.rs`, and the corpus and the global-state inventory keep it honest.
2. Retire the hand-written list rows (`ss_list.c`, `gd_list.c`, `ac_list.c`).
3. Build and run with MSVC, and decide F1 there.
4. Prototype the DCS adapter: a Lua module (mlua) or a sidecar process, driving `step` from the simulation frame and implementing `World` over DCS unit positions.

The TSTL port stops being the delivery vehicle. It remains useful as an independent implementation for the slices it already covers.
