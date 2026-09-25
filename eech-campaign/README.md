# eech-campaign

Enemy Engaged: Comanche vs Hokum's dynamic campaign, and the whole engine
behind it, as an embeddable native module.

## The full engine: EECH headless, as a Lua module

All of EECH (the maintained Windows build's 1,225 C files: dynamic campaign,
AI, pathfinding, flight, weapons, damage, comms, entities) compiles headless
for x86-64 Linux into a Lua 5.1 module. A Rust host process embeds Lua,
loads the module with `require ("eech_dc")` and records Tacview.

```
   eech-world (host binary)     Rust; embeds Lua 5.1; Tacview ACMI 2.2
        │  require ("eech_dc") → boot { ... } · engine:frame (ms) · engine:objects ()
        ▼
   eech-dc (libeech_dc.so)      the Lua module (mlua, lua51 + module)
        ▼
   eech-engine / -sys           safe API · native build, headless platform layer, synthetic data
        ▼
   original EECH C              ../aphavoc/source, ../modules (1,217 files as written)

   eech-map                     OpenStreetMap + SRTM → an EECH map and dynamic campaign
```

```sh
cargo build --release -p eech-map -p eech-dc -p eech-world
# a map and campaign from OSM and SRTM (Georgia: SRTM N41-N43 x E040-E046), into an installation root
target/release/eech-map georgia-latest.osm.pbf srtm-georgia/ /tmp/georgia
# run the dynamic campaign for 6 simulated hours, recording Tacview
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/georgia scenario=georgia hours=6 acmi=georgia.acmi
```

A twelve-hour Georgia campaign recorded this way is in
[`recordings/georgia-12h.zip.acmi`](recordings/) (Tacview opens it directly).

Details, findings and limits: [`docs/engine.md`](docs/engine.md). The M1
candidate (Windows), what its evidence establishes and what it does not:
[`docs/m1-baseline.md`](docs/m1-baseline.md).

## The campaign kernel spike

The Enemy Engaged: Comanche vs Hokum dynamic campaign as an **embeddable native
module**: the original EECH campaign C, compiled for the host target, behind a
Rust API that owns the architecture.

```
                     host (DCS integration, tests, tools)
                          │  Campaign · CampaignConfig · World · CampaignEvent · CampaignSnapshot
                          ▼
   crates/eech-campaign   public, safe API            #![forbid(unsafe_code)]
                          │  private FFI
                          ▼
   crates/eech-sys        native build + raw FFI      all unsafe lives here
                          │  csrc/eech_kernel.h (entity refs, ints, floats: no EECH types)
                          ▼
   original EECH C        ../aphavoc, ../modules      ~92% of the running code
```

This is the result of a feasibility spike. **Verdict: the original campaign C
works as an embeddable native engine.** The spike's first vertical slice (the
supply chain) runs behind the Rust API on x86-64 Linux, i686 Linux and x86-64
Windows, and reproduces the original 32-bit C reference byte for byte on every
scenario of the eech-core-ts corpus. The evidence and the constraints found are
in [`docs/feasibility.md`](docs/feasibility.md).

## Quick start

```sh
cargo build
cargo test                          # module tests, corpus differential, findings
cargo run -p eech-harness -- run scenarios/supply-chain.json
```

Requirements: a Rust toolchain (1.80+) and a C compiler (gcc or clang; the
build fails loudly without one). Nothing else: the build compiles the original
C from `../aphavoc` and `../modules` itself (set `EECH_SOURCE_ROOT` to build
from another checkout).

```rust
use eech_campaign::*;
use std::time::Duration;

let mut campaign = Campaign::new(config)?;             // a campaign as a saved game holds it
let report = campaign.step(&mut world, Duration::from_millis(500))?;
for event in report.events { /* LowOnSupplies, MissionCreated, ... */ }
let state = campaign.snapshot();                       // forces, keysites, groups, missions
```

`world` implements [`World`](crates/eech-campaign/src/world.rs): the physical
positions of campaign aircraft and the 3D object database, which EECH asks for
synchronously in the middle of its update.

## What runs

`Campaign::step` is EECH's host frame: the frame delta, then
`update_client_server_entities ()` over the original update list. The slice
implemented today:

```
keysite update (ks_updt.c)            supply usage, cargo crates (keysite.c), assignment timer
  └─ FORCE_LOW_ON_SUPPLIES            fc_msgs.c response: duplicate guard, supplier, cargo
       └─ create_supply_task          taskgen.c: create_task, route, start keysite, MISSION_CREATED
            └─ assign_keysite_tasks   assign.c: task order, suitable registered group, locality ETA
                 └─ assign_primary_task_to_group   ← boundary: CampaignError::Boundary
group update (gp_updt.c), task update (ts_updt.c: expiry)
```

Anything outside the slice fails loudly with the EECH function or dispatch row
it reached (`CampaignError::Unported`), never with an invented default.

## Layout

| Path | What |
|---|---|
| `crates/eech-sys/build/` | the native build: source closure (`spec.rs`, forked from eech-core-ts's `extract.mjs`), verbatim extraction with `#line` provenance, the two source patches (`patches.rs`), name tables |
| `crates/eech-sys/csrc/` | the native host layer (C): dispatch tables, environment, entries and aborts, entity identity, restore, legacy replay |
| `crates/eech-sys/src/` | raw FFI and the `Kernel` singleton; trampolines (panic-safe) |
| `crates/eech-sys/global-state.txt` | every writable global of the kernel, classified (checked against `nm`) |
| `crates/eech-campaign/` | the public API; `conformance` feature: replay of the eech-core-ts scenario language |
| `crates/eech-harness/` | headless harness: JSON scenario → `HeadlessWorld` → `Campaign` → JSON result; `replay` for the corpus |
| `scenarios/` | semantic harness scenarios and their recorded results |
| `corpus/` | the eech-core-ts corpus with the original C's output and the TSTL verdict (`tools/corpus/export.sh`) |
| `docs/` | findings: feasibility, 64-bit, floating point, global state, patches, ports, closure, conformance, DCS |

## Commands

| Command | Proves |
|---|---|
| `cargo test` | public-API behaviour; corpus: native == 32-bit C reference (and TSTL); B1 probe; global-state inventory; database immutability |
| `cargo test --target i686-unknown-linux-gnu` | the same module on EECH's platform |
| `EECH_ORIGINAL_STACK_ATTRIBUTES=1 cargo test --target i686-unknown-linux-gnu` | patch P1 is behaviour-neutral where the original idiom works |
| `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine cargo test --target x86_64-pc-windows-gnu` | Windows x64 (LLP64) |
| `EECH_C_OPT=2 cargo test`, `CC=clang cargo test` | results do not depend on optimisation or compiler |
| `EECH_FPU_ROUNDING=nearest cargo test -p eech-harness --test corpus` | the corpus detects a wrong rounding mode (fails, by design) |
| `EECH_CORPUS_FRESH=1000 tools/corpus/export.sh && cargo test` | 7000 fresh random scenarios (needs Node and gcc-multilib) |
| `EECH_RECORD=1 cargo test -p eech-harness --test scenarios` | re-record the semantic scenarios' results |

CI: `.github/workflows/eech-campaign.yml`.
