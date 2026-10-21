# M5: host-observed SUPPLY arrivals

**Decision.** The technical lead authorised this arrival-first slice on
2026-10-21 from frozen identity revision `ce5ab4cd`. It advances criteria 2 and
3 of the [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337),
version 1. It is partial engineering, not completion of either criterion.

Lua `dc.boot` now requires `bind_mobile`, `observe_loss` and `supply_reached`. During a
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

The public C boundary accepts only finite copied floating-point facts; a NaN or
infinity is a malformed descriptor and fails loudly before the host callback.
The reference helper's unordered-comparison tests protect its legacy predicate
when called directly, but do not claim that non-finite facts can pass through
the public C boundary.

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
causal run's first post-transition snapshot observes receiver fuel `60 -> 100`,
waypoint `DROP_OFF -> FINISH_DROP_OFF`, and cargo `1 -> 1`. It records that
instant without claiming the cargo is retained permanently or removed later.

`tools/supply-arrivals-lifecycle-windows.ps1` checks a missing callable before
boot, nil and thrown callback failures at a natural SUPPLY arrival, poison
behaviour after failure, and nested public operations from inside the callback.
Portable unit tests call the actual reference-host predicate with Lua
descriptors at the fixed-wing landed, early, radius, forward/reverse segment
and degenerate-segment boundaries, and at the helicopter AI landed, rotor,
leader-radius, AI-leader cohesion and human-leader boundaries.
Hosted CI cannot run these cases because the retail campaign and map data are
not distributable; the build, lint and source-level tests remain mandatory CI.
The lifecycle script rejects an unknown `case=` before loading `eech_dc` or touching
the installation, and both Windows runners build the child command line with
`tools/native-arguments.ps1` so a root containing spaces reaches the host intact.
Both are checked in hosted CI by `tools/supply-arrivals-tools-selftest.py` (a Lua 5.1
driver with a stub module, and a real child process receiving spaces, quotes,
backslashes and an empty argument); neither changes campaign behaviour.

## Compatibility and remaining work

This is a breaking host API change. Every maintained EECH World runtime and
lifecycle caller passes the `supply_reached` callable; there is no hostless or
embedded-answer fallback. The staged DCS probe still boots without the
mandatory host callbacks and therefore fails until a real DCS adapter supplies
`bind_mobile`, `observe_loss` and `supply_reached`. Legacy Tacview IDs,
`engine:objects()`, recordings, metrics, random stream and frame timing remain
unchanged, as do the arrival handlers for every non-SUPPLY case this slice did not
route: ALTITUDE, HEADING and WEAPON_VECTOR criteria, ground vehicles and members
without a host identity. The one later routing change, for non-SUPPLY
radius-only fixed-wing and helicopter arrival, is described below. The reference predicate is
synchronous in the same frame, preserving the existing consequence ordering.

The physical aircraft and movement model still live inside the DLL. Non-SUPPLY
radius-only fixed-wing and helicopter arrival now uses this seam
([`m5-aircraft-arrivals.md`](m5-aircraft-arrivals.md)); all other non-SUPPLY
arrivals remain coupled. A DCS adapter must supply the callback from
real DCS object facts; this repository does not claim that adapter exists. The
bounded loss slice (PR #88, [`m5-physical-loss.md`](m5-physical-loss.md)) is
delivered: physical destruction occurs independently while host loss reporting
controls campaign consequences through the durable roster, without gating all of
`kill_local`.
