# D3 target-point services: planned DCS calls and remaining coupling

This is a provider plan and an explicit gap record for EEDCS-52, not a native DCS
result or Task52/D3 acceptance. The production source is `8118389a96`; the finite
reference-host query review is `a1d6b37c`. The live M5 goal and migration ledger
remain authoritative. No campaign rule, runtime, baseline or tolerance changes.

The pinned `flying-dice/tslua-dcs` declarations at
`1179554fcafcef92d71313336cc09500febf8245` are reference material. Reading them does
not resume the historical TypeScript port. The exact Object, Unit and coordinate
file blobs were retrieved and verified; their provenance is in
`reference/d3-target-point-characterization/dcs-model-reference-20261103.json`.
These declarations establish the named API shapes below, not measured DCS
behavior, an exhaustive native API inventory or a dynamic sub-object oracle.

| Required physical fact | Planned DCS call or host data | Boundary and limitation |
| --- | --- | --- |
| Object origin for an explicitly absent target point or other defined position fallback | [`Object.getPoint()`](https://github.com/flying-dice/tslua-dcs/blob/1179554fcafcef92d71313336cc09500febf8245/packages/tslua-dcs-mission-types/src/Object.ts) | Resolve the host's object identity in its registry. An unknown object/provider is an error; it is not a legitimate absence. The campaign continues to choose its target. |
| Current root placement | `Object.getPosition()` returns the declared `Position3` fields `p`, `x`, `y`, `z` | This supplies a root pose. It does not establish the current joint pose of a named target-point sub-object. Coordinate/basis conversion needs an actual DCS check. |
| Model identity and available bounds | `Object.getTypeName()`, [`Unit.getDesc()` / `Unit.getDescByName(typeName)`](https://github.com/flying-dice/tslua-dcs/blob/1179554fcafcef92d71313336cc09500febf8245/packages/tslua-dcs-mission-types/src/Unit.ts); `UnitDesc.box.min/max` are declared vectors | A DCS type needs an explicit mapping to the campaign scene name. Bounds do not prove an authentic aim point, animation chain or collision mesh. Do not silently substitute their centre. |
| Per-model LOCAL target-point offset (`TARGET_POINT_OFFSET`, including all three coordinates used by guidance) | A versioned host-owned per-type target-point catalogue, keyed by the explicit model mapping | This is the bounded data workaround where no verified call supplies the semantic point. It must state the configured offset or a defined absence for every required scene and retain provenance. Unknown/missing data refuses initialization. Seeker limits, guidance and engagement policy remain Core. |
| Target-point selection and its CURRENT world position (`TARGET_POINT_CHAIN` / `TARGET_POINT_WORLD`); fixed-point rotation (`TARGET_POINT_FIXED`) | The host's configured semantic point/transform service, root pose and any explicitly supported live joint data | The host performs the physical transform. A root pose plus a configured rigid local point can serve that declared rigid-model contract; it does not reproduce an animated point. Required unsupported joint behavior must be resolved explicitly rather than answered with fabricated data. |

The reference service currently consumes a host-selected chain of child ordinals
and CURRENT pose/scale words marshalled from Core's legacy instance storage.
Its reply checks the scene name, operation, finite words and chain bound. That
storage and its declared counts remain coupled, as the finite review states.
The query also does not itself establish a DCS object-identity lookup. Neither
`getPosition()` nor these declarations supplies that legacy chain automatically.

Before whole D3 closure, the required hierarchy/instance extraction must supply
the remaining semantic instance services and resolve how host identities and live
placements reach these queries. Audit the active rotor-damage and smoke-attachment
consumers as well as device/muzzle/world-viewpoint callers; their physical work
cannot be dismissed as rendering or deferred merely because this finite getter
slice passed. The DCS provider must not require private C pointers or invent host
identities from an EECH entity index. This document does not close that work.

The later native DCS check must resolve a configured host object, compare the
reported root basis and configured point with that object's observed position,
exercise movement/rotation and each supported animated point, and demonstrate
clear errors for a lost identity, missing catalogue entry and unsupported joint
capability. It must distinguish intentional physical-world variation from a
campaign regression. Reference-host complete byte-identical conformance remains
a separate required gate; this plan is no replacement for it. Live DCS campaign
operation remains M6, as the M5 goal specifies.
