# M5: host-observed SUPPLY arrivals

**Decision.** The technical lead authorised this arrival-first slice on
2026-10-21 from frozen identity revision `ce5ab4cd`. It advances criteria 2 and
3 of the [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337),
version 1. It is partial engineering, not completion of either criterion.

Lua `dc.boot` now requires both `bind_mobile` and `supply_reached`. During a
frame, the campaign calls `supply_reached` only for PICK_UP,
PREPARE_FOR_DROP_OFF, DROP_OFF and FINISH_DROP_OFF on a SUPPLY task. Missing
callables, callback errors and non-boolean results fail loudly. The C operation
guard rejects same-thread re-entry while the callback is active; it is not
thread synchronisation (`Engine` remains `!Send` and the C flag is non-atomic).

## Boundary

The campaign still owns the current waypoint, radius criterion, task leader,
valid-member mask and answer combination. For each member it selected, it
copies these facts to the host:

- the opaque host-issued mobile identity and durable member key;
- session-local task and waypoint incarnation keys used only for correlation;
- fixed-wing/helicopter kind, AI and task-leader roles, landed and update
  flags, rotor spin-up time, current distance and velocity, frame delta,
  radius, and copied position, motion, guide-position and leader-position
  vectors.

The host returns one boolean physical observation. No EECH pointer or mutable
record crosses the C API. Task and waypoint keys combine entity index and a
session-local generation incremented at the central heap release; they are not
host identities and are never presented as DCS identities.

`eech-world` computes the old fixed-wing and helicopter predicates from those
raw facts outside `eech_dc.dll`. It preserves the fixed-wing landed, early
range, direct-radius and pass-through-segment checks, and the helicopter AI
landed/rotor, leader-radius and nonleader-cohesion checks. The last nonleader
case is implemented at the callback call site. As in the original C, radius
and cohesion checks accept unordered floating-point comparisons. The four
maintained waypoint rows currently set LAST_TO_REACH false for both aircraft
types, so boot also validates that exact scoped invariant rather than claiming
a natural runtime case that cannot occur.

Those four rows also have no action message and disable action checking. Boot
validates both facts, and the guide action predicate explicitly refuses the
four scoped waypoints. Therefore withholding the reached observation cannot
perform pick-up, drop-off or completion through the separate action route.

## Causal observation

`engine:supply_tasks()` is a read-only snapshot of extant SUPPLY task state.
It reads the actual task's current waypoint and completion field, the actual
supplier and receiver supply fields, and cargo of the task's type on the
current leader. It also reports the copied leader host identity and the
session-local task/waypoint correlation keys. It does not maintain a shadow
arrival state or make a campaign decision.

`tools/supply-arrivals-windows.ps1` drives three public Lua cases on retail
Lebanon: the reference host sees all four waypoint kinds; a withheld PICK_UP
leaves the task at that waypoint without debiting the supplier or attaching
cargo while physical position reports continue, then a true observation
advances the task, debits the supplier and attaches cargo; and a delayed loaded
DROP_OFF leaves cargo and receiver stock unchanged until a later true physical
observation, then advances the task and credits the receiver. The retained
legacy group-receiver path credits supplies but leaves its cargo attached at
DROP_OFF, so the causal check records that state without claiming a removal.

`tools/supply-arrivals-lifecycle-windows.ps1` checks a missing callable before
boot, nil and thrown callback failures at a natural SUPPLY arrival, poison
behaviour after failure, and nested public operations from inside the callback.
Portable unit tests call the actual reference-host predicate with Lua
descriptors at the fixed-wing landed, early, radius, forward/reverse segment
and degenerate-segment boundaries, and at the helicopter AI landed, rotor,
leader-radius, AI-leader cohesion and human-leader boundaries.
Hosted CI cannot run these cases because the retail campaign and map data are
not distributable; the build, lint and source-level tests remain mandatory CI.

## Compatibility and remaining work

This is a breaking host API change. Every maintained caller passes the second
callable; there is no hostless or embedded-answer fallback. Legacy Tacview IDs,
`engine:objects()`, recordings, metrics, random stream, frame timing and all
non-SUPPLY arrival handlers remain unchanged. The reference predicate is
synchronous in the same frame, preserving the existing consequence ordering.

The physical aircraft and movement model still live inside the DLL. All
non-SUPPLY arrivals remain coupled. A DCS adapter must supply the callback from
real DCS object facts; this repository does not claim that adapter exists. The
next loss slice must let physical destruction occur independently while host
loss reporting controls only campaign consequences through the durable roster.
It must not gate all of `kill_local`.
