# M5: host-owned AI helicopter rotor and dying/crash motion (EEDCS-36)

**Status: implemented at runtime commit `68a16fc1` (draft PR95), natively validated,
and awaiting the parent's independent review and the technical lead's approval.
Nothing here is accepted. D1, the aircraft work and M5 are not complete.** The
parent owns push, PR, Jira/Confluence, approval and merge.

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

## Draw precision

The tumble draws are `frand1 ()` doubles, and the original forms
`PI * 0.2 * frand1 () * dt` in double, narrowing only the float heading or roll it
assigns. The first transport carried the draws as floats. The first native pack (at
`3ace3841`) then failed one of 40 checks, `retained.observations`: first difference at
line 62489 (t = 1559.67, snapshot 83324, a dead AH-1T), height 0.75041 in the baseline
against 0.75040. Both complete Tacviews still equalled the baseline.

`68a16fc1` carries the draws as doubles end to end: C `hc_draws[2]` (`hc_move.c`,
`rv_hostmv.h/.c`), `eech-engine-sys`, `eech-engine`, `movement.rs`, `hc_step.c`, and an
exact `%.17g` f64 Lua read in `main.rs`. Real float fields stay float; the RNG order and
count are unchanged. `hc_draws` is the step's last field so the double's alignment moves
no other field, and `eech-engine-sys` asserts that at compile time. This corrects the
candidate. It is not an accepted difference, and no golden, tolerance or scenario changed.

Mutation check: with the step's call narrowed back to `f64::from(draw as f32)`,
`the_tumble_step_hands_the_helper_the_draws_it_received_in_double` fails (red); restored,
both precision tests pass (green). The unit tests characterise the ported maths and the
guards; bit-identity is the regression pack's job.

## Evidence

All for runtime commit `68a16fc1a31715d5f884344d9e9ae06a980004b1` (tree `4c6904a7`,
0 dirty inputs). The compact portable record is
[`../reference/m5-helicopter-lifecycle/`](../reference/m5-helicopter-lifecycle/)
(`reference-summary.txt`, `identities.txt`, `BUILD-INFO.txt`, pack report, witness
outputs, container log). The Tacview recording and observations are identified by hash
there, not copied.

| Evidence | Result | Source |
|---|---|---|
| Linux container (pinned image `sha256:bb10e44a…562be`, Rust 1.98.1): `fmt` clean; `clippy --locked --offline --all-targets -D warnings`, no warnings; `cargo test` | `eech-dc` 2; `eech-engine` 5; `eech-world` library 57, binary 18, `host_callbacks` 2, `load_dc` 2; 0 failed | `container-clippy-and-test-final.log` |
| Precision mutation | red (1 failed), green (2 passed) | `precision-mutation-red2.log`, `-green2.log` |
| Retail MinGW build, fresh volume, pinned image | exit 0, 0 dirty inputs | `BUILD-INFO.txt` |
| Focused 0.5 h Lebanon replay | 72,971 observation lines equal the accepted prefix, 0 differences (prefix only, not full acceptance) | `replay-0.5h-prefix-comparison.json` |
| Public witness, control and 1 s hold | PASS | `witness-*.out`, `witness-driver.log` |
| Full regression pack | exit 0, PASS, 40 of 40 checks, `windows-retail v1`, Georgia and Lebanon exact IDENTICAL | `pack-report.json`, `pack-report.md` |
| Reference runs (the parent's independent hashing) | Tacview `3bd17b64…2d19b` and observations `da26bab0…77cf` equal the accepted baseline in both runs; metrics identical; 39 of 39 campaign expectations | `identities.txt`, `parent-independent-reference-hashes.json` |

The Linux build compiles the host C (`hc_step.c`) and the Rust; the `aphavoc` call sites
(`hc_move.c`, `hc_float.c`, `hc_pack.c`, `hc_updt.c`, `rv_hostmv.c`) were compiled by the
retail MinGW build. The pack reports the build as differing from the accepted one in
`eech_dc.dll`, `eech-world.exe`, `lua.dll`, `lua/campaign.lua` and `lua/lifecycle.lua`;
there is no output difference to classify. The Tacview recording was checked by hash and
was not visually reviewed. The pack's blind spots stand (`coupled-engine`, `three-hours`,
`one-war`, `captures`, `tacview-visual`, `linux`).

Retained outside the repository and not passes: the original 39/40 pack at `3ace3841`
and the stopped 20 s hold attempt (`native/` in the external output directory).

## Public witness

`crates/eech-world/lua/helicopter-lifecycle.lua` through `tools/helicopter-lifecycle-windows.ps1`
(Lebanon retail root; embedded Lua 5.1). Subject: the retained natural loss of the
Lebanon reference run, the Mi-17 Hip `eech-world:3069` (legacy entity 87568, group
87567) that dies at 1584.66 s ([`m3-supply-loss.md`](m3-supply-loss.md)); the host ID
comes from the campaign's binding, never from the legacy index.

- **Causal claim: rotor hold and release only.** From the subject's first `rotor_down`
  (1585.00 s) the host declines to spin the rotor down for `hold_seconds` (default 1 s)
  and answers the rotor it already holds. Result: 11 held steps over 1.1 s with 0 rotor
  changes; 23 released steps; the first change after release was `rotor_down` at
  1586.10 s. Host answers therefore drive the campaign's rotor, through the public
  Lua -> module -> world path. The 1 s window comes from the control run: the settle
  burial at about 1587.7 s clears the rotor independently of `rotor_down`, so a
  longer hold (a 20 s attempt was stopped) cannot isolate `rotor_down`.
- **Coverage, not claims (control).** `rotor_down`, `fall`, `fall_gravity`, `contact`
  and `tumble` 34 times each (every `rotor_down` changed the rotor), `settle` 7 times
  (buried once), `impact` once. These phases are not held or delayed, and no terminal
  state is claimed.
- **Limits.** One subject, on the fall path. The crash slide (`crash_begin`/`crash_end`)
  and `rotor_idle` appear in the fleet-wide counts and the regression pack, but this
  witness does not hold or change them. Its `exit.txt` is empty because PowerShell's
  `$LASTEXITCODE` was null on completion; the verdict is the driver log's `PASS` and
  the hold output's `result ok`.

`crates/eech-world/lua/helicopter-movement.lua` covers only live, alive-state integration; this witness
covers the migrated rotor and dying state.

## Remaining coupling

D1/aircraft is **not** finished. These are retained physical dependencies in the DLL,
not campaign policy, and each needs its own boundary decision:

- **D3, terrain:** sampling and its cache, including the sample the fall uses.
- **D2, effects:** smoke, effects and sounds raised during dying and crash.
- **D1, other aircraft physics:** pilot control, avoidance, targeting and guidance;
  bounding-box sizes; the DEAD state clears and `kill_client_server_entity`; the
  `frand1` draws stay in the DLL (only their values cross the seam).
- The DLL's rotor copy remains because the entity still lives in the DLL, and the
  campaign and the physical world still share `eech_dc.dll` (the `coupled-engine`
  blind spot).
- Other direct writers announced to the host only through the setters' `place`.
- Client/multiplayer helicopters are unsupported in a host build.

The campaign keeps deciding when a helicopter dies and the death consequences (group
state, removal, logistics).

## Commands

```
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world -- -D warnings
cargo test -p eech-world -p eech-engine -p eech-dc        # pinned image
EECH_WIN_TARGET=<fresh volume> tools/build-windows.sh target/lifecycle-precision
tools/helicopter-lifecycle-windows.ps1 -Root <lebanon root> -Bin target/lifecycle-precision -Out <out>
python tools/regression-pack.py --georgia <georgia> --lebanon <lebanon> --lebanon-repeat <lebanon copy> --bin target/lifecycle-precision --out <out>
```
