# DCS integration: constraints (research, not implemented)

The spike's first objective was a headless native module, and that exists. This
note checks that the architecture does not rule out DCS. It does not implement
an adapter.

## How native code runs in DCS today

- DCS World is a 64-bit Windows program that embeds **Lua 5.1**. The module
  builds and passes its tests for `x86_64-pc-windows-gnu` (`docs/64-bit.md`).
  An MSVC build is the remaining step.
- Rust native Lua modules are established practice. DCS-gRPC
  (github.com/DCS-gRPC/rust-server) is a Rust `cdylib` that uses **mlua** with
  `default-features = false, features = ["lua51", "module", ...]`. It is loaded
  from a Lua script started by a line added to `Scripts/MissionScripting.lua`
  (`dofile (lfs.writedir () .. [[Scripts\DCS-gRPC\grpc-mission.lua]])`), plus a
  GUI hook in `Scripts/Hooks`. The mission scripting environment is sanitised:
  `require`, `package`, `loadlib`, `io`, `os` and `lfs` are removed. So a
  native module needs either that one-line change to `MissionScripting.lua` or
  a hook that runs in the GUI/server environment, where they are available.
- **mlua is therefore genuinely useful, in the adapter.** It is the
  established way to expose a Rust `cdylib` as a DCS Lua module. It is not
  needed to run the campaign, which is native C. The recommended shape is a
  separate `eech-dcs` crate: a `cdylib` using `mlua` (`lua51`, `module`), built
  on `eech-campaign`. It must not embed a second Lua VM, and `eech-campaign`
  must not depend on mlua.

## How the adapter maps onto the API

| DCS | eech-campaign |
|---|---|
| mission start: read the campaign (a save or a campaign file) | `Campaign::new (CampaignConfig)`. Config names are the DCS unit, group and airbase names; `Campaign::entity (name)` resolves them. |
| a timer (`timer.scheduleFunction`) or the simulation-frame hook | `campaign.step (&mut dcs_world, dt)` |
| `Unit.getByName (name):getPoint ()` | `World::position`. The adapter maps `EntityId` → DCS unit name with the config names. |
| the object database (crate models) | `World::object_bounds`, from a table shipped with the adapter |
| `CampaignEvent`s | Lua callbacks, messages, map markers; `Replicated` is unused (DCS replicates) |
| the first `WorldCommand` (eech-core-ts 6b: a group gets its route) | `group:getController ():setTask (...)` |

## Constraints the architecture must respect

1. **Threading.** DCS runs its Lua on the simulation thread, and `step` must be
   called from that thread. `World` queries then call back into DCS's Lua
   synchronously, from inside `step`. That is safe: `World` borrows only
   itself, and the campaign is not reentered. `Campaign` is `Send`, not
   `Sync`, which matches.
2. **The floating-point environment.** DCS has its own MXCSR and x87 control
   word. Every kernel entry saves the caller's environment, runs EECH rounding
   toward zero, and restores the caller's environment. Host callbacks run
   under DCS's environment (`docs/fpu.md`), so the campaign cannot perturb the
   simulation's numerics.
3. **Crash safety: the important one.** EECH's release code dereferences NULL
   on some inputs (the corpus reaches 24 such outcomes in slice 1). In process,
   that crashes DCS. The module never installs fault handlers in a library host
   (only the dedicated replay process does), and it should not. Two options:
   - **In process.** The facade makes such inputs unrepresentable. For example,
     `CampaignConfig` validation would refuse a memberless group whose position
     the campaign will ask for. Each slice's NULL outcomes (the corpus lists
     them) then become validation rules or `CampaignError`s. This is the right
     long-term direction, but it needs discipline.
   - **Out of process.** `eech-campaign` runs in a sidecar executable, and the
     DCS module is a thin mlua client over IPC, as DCS-gRPC does over gRPC.
     `World` queries become synchronous IPC round trips, and the simulation
     never shares an address space with legacy C. The API is transport-neutral,
     so the choice can be made late.
4. **One campaign per process.** This is natural for DCS, which has one
   mission per server process. In-process tests of the adapter must run
   campaigns one after another.
5. **Panics.** A panic in the adapter's `World` is resumed on the Rust side
   after the C call. mlua turns it into a Lua error, so a panic never unwinds
   through C or into DCS's Lua.
6. **Frame budget.** Not measured on a realistic campaign. The supply-chain
   scenario runs 60,000 frames in 34 ms, a release build with the C at `-O0`.
   EECH's own update splits a frame into `delta × rate + 1` passes, so the cost
   depends on the configured update rate as well as on the campaign's size.

Nothing found makes DCS integration impossible. The decision to make before
the adapter is option 3 (in process or out of process).
