# M5: host-owned AI helicopter rotor and dying/crash motion (EEDCS-36)

**Status: implemented locally; native build, witness and regression pack results are
recorded under Evidence only once they exist. Nothing here is accepted.** It awaits
the parent's independent review and the technical lead's approval. **D1 and the
aircraft work are not complete.** The parent owns push, PR, Jira/Confluence,
approval and merge.

**Scope.** One physical extraction under D1 (EEDCS-15), on base `f70f30c3` (the PR #94
merge), following [`m5-helicopter-movement.md`](m5-helicopter-movement.md). The rotor
state of an AI helicopter and its dying, fall, contact, settle, tumble and crash
movement are stepped by the host, per host ID, through the same `move_vehicle` seam.
The original C arithmetic and its order are kept. The campaign still decides when a
helicopter dies and what its death means; the host's retained physical state drives
the DLL's response. A live DCS run is M6 and is not claimed.

## What moved

`move_vehicle` gains twelve lifecycle modes (values 10 to 21; Lua snake_case names)
for `movement_kind` 5. Every result carries `rotor[3]` (main rpm, tail rpm, spin-up
timer). `pending` is true only for `crash_begin`.

| Mode | Host does | `moved` |
|---|---|---|
| `rotor_set` | adopts the campaign's rotor (explicit adoption, like Place) | false |
| `rotor_alive` | original alive-rotor integration, spin-up gated | spin-up gated |
| `rotor_down` | original dying-rotor spin-down (takes `delta_seconds`) | false |
| `rotor_idle` | original idle spin-down of a follower-less helicopter (takes `delta_seconds`) | false |
| `fall` | original dying fall step; clears the retained contact altitude | false |
| `fall_gravity` | original gravity step of the fall | false |
| `contact` | ground contact test; also returns `altitude`; a false flag clears the retained altitude | touched |
| `settle` | settle into the ground; consumes the retained contact altitude | buried |
| `tumble` | original tumble; consumes the crash candidate | false |
| `crash_begin` | starts a crash slide (`pending = true`) | false |
| `crash_end` | finishes the slide; consumes the candidate | stopped |
| `impact` | original impact step | false |

While a crash slide is pending every mode except `crash_end` is refused.

- **Original arithmetic, copied.** `crates/eech-world/csrc/hc_step.c` copies the
  rotor and dying-motion expressions; `movement.rs` holds the records and guards.
- **Host-retained state with mirror validation.** The host keeps the rotor, the
  contact altitude and the crash candidate per host ID. Place and `rotor_set` are the
  explicit adoptions; every other step compares the DLL's value bit-exactly with the
  host record and refuses a difference before mutating anything.
- **ABI layers change in lockstep:** C `campaign_movement_*` (`rv_hostmv.h/.c`),
  `eech-engine-sys`, `eech-engine`, `eech-dc` `call_move_vehicle`, and `eech-world`
  (`lifecycle_from`, `movement.rs`, `hc_step.c`).
- **DLL side.** `hc_move.c`, `hc_float.c`, `hc_pack.c` and `hc_updt.c` call
  `campaign_movement_*` under `EECH_CAMPAIGN_HOST_MOVEMENT`; the original bodies stay
  in `#else`/`#ifndef`, so builds without the flag are unchanged. A hosted helicopter
  without a host ID fatals rather than falling back to DLL physics.
- **Death protocol order is kept:** `rotor_down`; on land with `contact(FALSE)` the
  crash path (`crash_begin` then `crash_end`); otherwise `fall`, the DLL's terrain
  sample, `fall_gravity`, `contact(TRUE)`, and if in contact the splash Place, a
  fresh CG and `settle` (with a clearing Place if buried); then the two `frand1`
  draws and `tumble`.

## Evidence (local; this stage)

Linux container (pinned image `eech-build`, sha256:bb10e44a…562be, Rust 1.98.1;
ephemeral clippy): `cargo fmt --all -- --check` clean; `cargo clippy --locked
--all-targets -p eech-world -p eech-engine -p eech-dc -- -D warnings` exit 0;
`cargo test`: `eech-world` movement 55 unit tests (15 new lifecycle tests), supply 17,
`host_callbacks` 2, `load_dc` 2; `eech-engine` 2 new transport tests; `eech-dc` 5.
Logs are in `E:/eech-runs/m5-d1/helicopter-lifecycle-20261022/`; failed attempts are
kept (`clippy.log`, no clippy in the image; `test-world.log`, `load_dc` needed the
built `libeech_dc.so`). The C half was not compiled until the native build below.

The new tests characterise the ported maths and the guards; bit-identity is the
regression pack's job.

## Public witness

`lua/helicopter-lifecycle.lua` through `tools/helicopter-lifecycle-windows.ps1`
(Lebanon retail root). Subject: the retained natural loss of the Lebanon reference
run, the Mi-17 Hip (legacy entity 87568, group 87567) that dies at about 1584.66 s
([`m3-supply-loss.md`](m3-supply-loss.md)); the host ID comes from the campaign's
binding, never from the legacy index.

- **Causal claim: rotor hold and release only.** From the subject's first
  `rotor_down`, the host declines to spin the rotor down for `hold_seconds` (it
  answers the rotor and motion vector it already holds). The campaign's rotor, handed
  to the host on each held step, must not change while held, and after the host
  answers again it must change, the first change since the release coming from the
  host's `rotor_down` answer and not from another phase.
- **Coverage, not claims.** The control run reports which lifecycle modes the
  subject passed through, in what order, and what the host answered for the phases
  that report an outcome (contact touched, settle buried, crash_end stopped). These
  phases are not held or delayed, and no terminal state is claimed.
- The script targets the embedded Lua 5.1. It was syntax-checked with Lua 5.1.5
  `luac -p` (rough check) before the native runs.

`lua/helicopter-movement.lua` covers only live, alive-state integration; this witness
covers the migrated rotor and dying state.

## Remaining coupling

D1/aircraft is **not** finished. These are retained physical dependencies in the DLL,
not campaign policy, and each needs its own boundary decision:

- Terrain sampling and its cache (D3), including the sample the fall uses.
- Smoke, effects and sounds raised during dying and crash; the RNG draws
  (`frand1`) around tumble.
- Pilot control, avoidance, targeting and guidance; bounding-box sizes; the DEAD
  state clears and `kill_client_server_entity`.
- The DLL's rotor copy remains because the entity still lives in the DLL.
- Other direct writers announced to the host only through the setters' `place`.
- Client/multiplayer helicopters are unsupported in a host build.

The campaign keeps deciding when a helicopter dies and the death consequences (group
state, removal, logistics).

## Author handoff

One inline author pass; not an independent review, which is the parent's. No pushes,
posts or identity changes. Next bounded action: the parent's review; if C or Rust
source changes, the settled build, witness and pack must be rerun on the new commit
and no golden or tolerance may be re-recorded.

## Commands

```
cargo fmt --all -- --check
cargo clippy --locked --all-targets -p eech-world -p eech-engine -p eech-dc -- -D warnings
cargo test --offline -p eech-world        # pinned image, fresh target volume
EECH_WIN_TARGET=<fresh volume> tools/build-windows.sh target/lifecycle-settled
tools/helicopter-lifecycle-windows.ps1 -Root <lebanon root> -Bin target/lifecycle-settled -Out <out>
python tools/regression-pack.py --georgia <georgia> --lebanon <lebanon> --lebanon-repeat <lebanon copy> --bin target/lifecycle-settled --out <out>
```
