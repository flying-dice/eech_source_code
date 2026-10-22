# M5: host-issued identities and a durable member roster

**Decision.** The technical lead authorised this prerequisite on 2026-10-21 from
the reviewed integration revision `0ba0384c`. It advances the six outcomes in
the [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337),
version 1, but completes none of them. The legacy physical world, entity heap,
creation, movement, damage and campaign consequences remain inside
`eech_dc.dll`.

The selected host must supply a `bind_mobile` callable at boot. The native
campaign calls it once when a physical group member first joins the campaign
roster. The callback receives copied diagnostic facts (`member_key`,
`group_key`, reusable EECH entity index, type, subtype and side) and returns an
opaque UTF-8 identity of 1–128 bytes. The campaign copies that identity and
never parses it. A missing callback, callback error, invalid UTF-8, NUL, excess
length or identity reused by another physical incarnation fails loudly. A
callback failure poisons the native engine and no later boot can reuse its
partially initialised state.

This is a breaking public API change: Lua `dc.boot` now requires
`bind_mobile`, and the Rust/C boot and frame calls require the corresponding
host callback argument. Every maintained caller and runnable repository script
passes it explicitly; there is no hostless compatibility fallback.

`eech-world.exe` is the first actual issuer: its Rust host creates the monotonic
`eech-world:<serial>` value and exposes the callable to host-owned Lua. The
value does not derive from the reusable EECH entity index. This demonstrates
that the selected process participates through an invoked boundary; it does not
make physical creation host-owned yet.

The roster source is registered only in the host-capable headless module build:
`sources.txt` now has 1,219 entries (the 1,218 unchanged legacy sources plus
`gp_roster.c`). Legacy Windows targets compile the guarded call sites as no-ops,
so this prerequisite does not silently impose a callback on the retail
executable.

## Identity lifetime

The roster separates physical lifetime from list membership.

- `LIST_TYPE_MEMBER` unlink only marks a member detached. Relinking the same
  live entity restores membership with the same member key and host identity.
- `amalgamate_groups` moves live landed members with
  `set_client_server_entity_parent` (`group.c`), which unlinks and relinks the
  same entity. The roster updates its group key without rebinding it.
- Selecting a group leader deletes and reinserts the member in the same list
  (`gp_ptr.c`). This also retains the identity.
- The generic entity-heap release (`set_free_entity`) ends the physical
  incarnation. Its append-only record remains observable with
  `physical_present=false`; the entity-index map is cleared. Reusing that index
  therefore creates a new member key and requires a new host identity.
  Under the host-movement build (EEDCS-38) the release also retires the host
  identity (`campaign_movement_retire`); the host then refuses any later step
  for that id and any reuse of it. See `m5-force-creation.md`.

The public `engine:members()` snapshot exposes copied values only. Its
`initial_alive` field is explicitly the internal birth fact read at first link;
there is no host loss report in this slice. It is not current state and is not
evidence of host-owned loss causality. `objects().id`, diagnostics, metrics,
structured observations and `Recording::record` remain unchanged, including
their existing index-reuse handling and ACMI identifiers.

## Callback and entry safety

The C entry layer rejects same-thread re-entry across boot, frame, observation
and installation operations. This non-atomic guard does not synchronise
concurrent C calls; the Rust `Engine` is `!Send`. Re-entry returns
`EECH_ENGINE_STATE` before changing the active host
or fatal-jump target. Public Lua lifecycle checks exercise nested boot and,
during a real frame-time bind, nested `members`, `objects`, `frame` and
`prepare_installation` calls. Rust catches callback panics; Lua errors become a
Rust error result; Rust returns a bounded error buffer to C; only after the
callback has returned does C enter its existing fatal/poison path. No Rust or
Lua unwind crosses C or `setjmp`.

The roster currently checks a new identity against the append-only roster with
a linear scan. `engine:roster_stats()` reports the real comparison count and
CPU time accumulated in that C scan so evidence can bound its cost at normal
campaign size. This is transitional instrumentation, not a frame-budget claim.

## Mapping to DCS

A future DCS adapter must implement the same callable in DCS-owned Lua and
resolve a real DCS object using the APIs proved by the DCS probe. The descriptor
keys remain diagnostic correlation values; they are not DCS identities. The
adapter will need an explicit uniqueness/lifetime decision using the relevant
DCS unit name and/or object ID evidence. No DCS adapter, DCS spawn request or
actual DCS identity binding exists in this slice.

## Remaining work

In code (not yet validated by the native pack), the host computes the physical
initial height and attitude from the campaign's input facts, owns the wreck
lifetime gate and timer, and refuses a retired identity, as of EEDCS-38
(`m5-force-creation.md`, which also records what remains coupled and unverified).
The campaign keeps formation, route, heading and the terrain base sample. The next cuts must publish campaign
requests, accept host observations, route the natural loss and arrival paths through
those observations, and remove pointer and hidden-world reads before the
campaign/world link split. The existing loss structural proof remains the
starting campaign-consequence implementation. Identity alone neither moves
loss authority nor proves the required causal difference when a host report is
withheld.
