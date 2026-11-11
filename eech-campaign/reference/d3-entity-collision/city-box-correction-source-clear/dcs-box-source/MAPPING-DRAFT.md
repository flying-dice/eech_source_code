# CITY box-construction mapping: verified shapes, open capability

Source-only draft for the required D3 mapping. This does not change migration scope, close the mapping gate, implement a TypeScript port or claim live DCS behavior.

Pinned author-maintained declarations: flying-dice/tslua-dcs at `1179554fcafcef92d71313336cc09500febf8245`. The new literal [StaticObject.ts](https://github.com/flying-dice/tslua-dcs/blob/1179554fcafcef92d71313336cc09500febf8245/packages/tslua-dcs-mission-types/src/StaticObject.ts) is Git-blob-exact (`0378b1f371fc57bc2d4f2b22dd4ed64653e92cad`). Existing verified Unit/Object/world declarations remain in the earlier DCS evidence pack.

| Needed fact for a possible DCS-facing box service | Source fact | Limit |
| --- | --- | --- |
| Unit descriptor bounds | Unit.ts:110-115 declares `UnitDesc.box.min/max`; getDesc/getDescByName are declared | Bounds only, not a collision mesh, point/normal oracle or changing sub-object state. Coordinate/origin and supported-type behavior still require validation. |
| Static descriptor bounds | StaticObject.ts:12-23 includes a box in an example; its `StaticObjectDesc` type at26-54 omits the box; getDesc/getDescByName return that type at140/301 | An example is not a declared universal field guarantee. The omission is not proof DCS never returns it. Do not silently reuse the Unit contract for every static/scenery object. |
| Live pose | StaticObject.ts:255/271 declares getPoint/getPosition; position carries point and orientation axes | This alone does not validate descriptor-axis conventions, EECH-to-DCS conversion, city membership or per-child geometry. |
| Type identity | StaticObject.ts:96 declares getTypeName | A type name is not automatically the campaign/host registry identity or a versioned physical descriptor. |

The new EECH reference constructor needs model bounds from the existing Host ModelStore, plus canonical city/child state. A box can be suitable input to a city broadphase aggregate; it must not be substituted for the separate ENTITY_COLLISION mesh service. Preserve this distinction in the eventual adapter mapping.

For a required DCS-facing constructor, the proposed bounded capability is: use the Host's explicit parent/member registry; resolve each supported member's type and pose; obtain and validate its finite descriptor bounds and documented coordinate/origin/scale conversion. If a required member has no proven bounds contract, use only a named versioned per-type bounds dataset with actual acquisition provenance and explicit supported types, or refuse the capability by name. No guessed dimensions, zero-box fallback, callback-order-as-list-order or invented complete catalogue.

Dataset/probe evidence still needed: DCS/module version, type and supplier, acquisition method/source, actual finite min/max, frame/origin/scale conventions, transform validation, supported-type inventory, retained raw capture hash and unsupported disposition. No such new DCS dataset is claimed here.

Exact follow-up DCS check (M6 environment, not executed here): for each needed mapped static/unit type, create/resolve a named object; record getDesc/getDescByName field presence and finite bounds alongside getPosition/getTypeName, DCS/module versions and the Host's axis conversion; verify a controlled translated/rotated instance against the intended box frame. Unsupported or absent fields remain explicit; do not infer a mesh result.

The final active-caller census must also decide whether this low-level constructor is internal to the alternative EECH reference physics backend. Existing LOS/weapon dialogues can accept a Host-completed result without requesting legacy collision subqueries, but that does not prove every remaining Core consumer is unnecessary. Name the actual DCS-facing contract only after that census; this draft cannot waive required physical services or all-ten D3 acceptance.
