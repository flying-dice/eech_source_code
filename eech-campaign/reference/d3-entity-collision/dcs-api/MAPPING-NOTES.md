# Entity collision DCS mapping: verified shapes and remaining decision

Primary author-maintained declarations were fetched through the connected GitHub
service at commit `1179554fcafcef92d71313336cc09500febf8245`. Each literal file's
Git blob and SHA256 were verified locally; see PROVENANCE.json. These are declared
shapes, not actual DCS measurements or an exhaustive API absence proof.

| Needed fact | Verified declared call | What it does not establish |
| --- | --- | --- |
| Candidate discovery | `world.searchObjects(category, volume, handler, data)` in world.ts; handler can stop the search | No declared nearest-hit order, collision point, face normal or legacy tail-first sector ordering. Do not use callback order as EECH ordering. |
| Live root pose and velocity | `Object.getPosition()` gives `p/x/y/z`; `getPoint()`, `getVelocity()` | No current named-child pose, mesh, root-to-EECH mapping or sweep collision result. |
| Existence, type and object category | `Object.isExist()`, `getTypeName()`, `getCategory()` | A type or runtime reference does not supply the campaign's host registry identity automatically. |
| Available unit bounds | `Unit.getDesc()` / `Unit.getDescByName(typeName)`; `UnitDesc.box.min/max` | Bounds are static descriptor data; not a collision mesh, child state or physical hit/normal oracle. |
| Animation input | `Unit.getDrawArgumentValue(argument)` | A draw argument is not a sub-object transform or calibrated geometry definition. |

The new reference ENTITY_COLLISION narrow-phase callback needs a candidate model
at the supplied pose, the actual hit decision and final intercept/normal words.
None of the verified calls above alone proves that complete contract. A rigid
box made from UnitDesc.box would be an explicitly different physical model, not
an exact model-intersection service; silently substituting it is not approved.

Next required decision belongs with the full collision/instance extraction: identify
which remaining active Core callers require a DCS-facing primitive and which
queries are internal to the alternative EECH reference physics backend. The DCS
simulator may resolve its own weapon physics without asking the EECH narrow phase;
that does not prove every Core collision consumer is unnecessary. For any required
primitive, name a verified full API contract or a bounded versioned per-type host
geometry/transform workaround with actual provenance and supported types. Unknown
objects, unsupported child state and missing geometry refuse by name, never miss.

This note supplies primary API shapes for the required mapping work; it does not
close that work, prescribe a DCS world clone or resume the historical TypeScript
implementation. No live DCS campaign run or whole-D3 acceptance is claimed.
