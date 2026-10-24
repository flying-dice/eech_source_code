# EEDCS-41 all-family host loss reports: retained evidence

Records how host loss reports (`engine:report_loss(host_id)`) were exercised for
the six force families. Behaviour and contract: [`docs/m5-physical-loss.md`](../../docs/m5-physical-loss.md).
Source of this record: the author's evidence map `PROVENANCE-AND-SELF-REVIEW-41.md`
and the parent's `INDEPENDENT-REVIEW.md`, kept in the (non-repository) run root
`E:/eech-runs/m5-d1/all-family-loss-20261023`.

## Status: final shipping native pack PENDING

The M4 regression pack on the shipping `final-default22` build was running when this
record was written (started 2026-10-24T01:24:34+01:00). **No pass is claimed.** No
live or incomplete pack output is copied here. The pack result, full-file
comparison to the accepted baseline, independent approval and D1 scope
reconciliation are all outstanding. This record is evidence for the extraction, not
acceptance.

## Source state

- Runtime source of the final builds: `9b4a668b4e608ee9b092193a8b2f2a1930187f46`
  (`builds/final21.src`, clean tree).
- Checkpoint `eefd0bb0` adds only the fixture Lua driver change and docs over
  `9b4a668b`; no `aphavoc/` or crate source changed, so it cannot affect the DLLs.
- Accepted PR99 (EEDCS-40, `2fadceb2`) was merged into the branch as `e618c6d1`, before
  `9b4a668b`, so the final builds include it.

## Build identities

SHA-256 of the exported binaries (build tool: fresh volume, pinned image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`;
`builds/*.build.head`, `*.build.exit` all 0).

| build | features | `eech_dc.dll` | `eech-world.exe` | source |
|---|---|---|---|---|
| final-default22 (shipping) | default | `e0071743142692379b7ef4991f4ff395c8a815bcf8fa31ca96c4e62f0a7d92e2` | `c6a5bf51bc855fe145f60afd4867a395f4578e3663489bf73f6c1a30aced57e2` | clean `9b4a668b` |
| final-fixture21 | `eech-dc/loss-fixture` | `0c2f89635bf3a65257ef2a21265ad51e616ac024875b5eeb3cf9b149f5c4b6bb` | `6538b7f96709d33f96cb1613f9fdf84b3fce7fe237a23840e6c44168238f77cf` | clean `9b4a668b` |
| ring9 (r10, r11) | `loss-fixture` | `ed6d5e2755db64013dd9f0f6b9c20c4830b76d81cdfecef46b8d77267b230e4b` | not retained | **partial tree**, no snapshot |
| rel12 (r12) | `eech-dc/loss-fixture` | `8517c29d7096d91fa3aedeb7ab4992ca6902d1b82559ae08be58de2bd6688970` | not retained | **partial tree**, no snapshot |
| fix17 (fix17) | `eech-dc/loss-fixture` | `9d844dba3dae731d1f8ad60b8308b3ac6ce1b7a2e41ea5fddee0c5bad6de283b` | `0f2777008469c3ffc6d092c91dc66b498ec28834fac8113b8fe026ed81c8715f` | **partial tree**, no snapshot |

`lua.dll` is `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` in every
export listed (default22, fixture21, ring9, rel12, fix17). The ring9, rel12 and fix17 builds were made
from the author's uncommitted working tree, whose exact source was not archived, so
no source snapshot is given for them and no binary equality is claimed between them
and the final DLLs. They are retained as reasoned evidence for family paths that the
final fixed-wing-only production delta cannot reach (routed, anti-air, person and
ship cannot enter the new fixed-wing landing branch because their landing actions
stay refused). That is a reuse argument, not reproducibility.

Witness driver (`crates/eech-world/lua/loss-all-family.lua`) hashes, which differ by
run: r10 and r11 `2e349d88…9745`; r12 `b79207f7…ecc6`; fix17 `b4e2311f…ede8`; r13
`9df358e1…50c8` (equal to the committed file at `eefd0bb0`).

## Per-family evidence (`runs/`)

Each witness boots the real public `eech_dc` module in `eech-world.exe`, kills a
member through the host, checks the loss is withheld with force-info and
regeneration unchanged, then reports by host ID and checks the consequence applies
once.

| family | file | build | exit | result |
|---|---|---|---|---|
| routed, anti-air, person | `r10-routed-aa-person.out` | ring9 | 0 | `result ok`, 54 checks pass, 0 fail; duplicate report rejected ("was already reported") |
| ship | `fix17-ship-overall-FAILED.out` | fix17 | **1** | ship checks pass (same-stack unlink, withheld unchanged, report losses+1, hardware -1, regen unchanged, group losses 0->1, logical 2->1). The run FAILED overall because fixed-wing was UNAVAILABLE in it. Kept as-is; no FORWARD control or probe (group of 2) |
| fixed-wing | `r13-fixed-wing-final.out` | final-fixture21 | 0 | `result ok`, 15 checks pass. Reservation observe 2, report-time 2, 0 intervening frames, report 2->1 once; force +1, hardware -1, regen insertion, group losses 0->1, logical 2->1, primary-task losses 0->1 |
| final-member rejection | `r13-final-member-rejection.out` | final-fixture21 | 0 | `negative.fails_explicitly` pass. This is final-member rejection only, not duplicate-report evidence |
| unknown host id | `r12-unknown-id-fw-unavailable-FAILED.out` | rel12 | **1** | `reject.unknown.raises` pass; run FAILED overall (fixed-wing UNAVAILABLE) |
| forwarded-report rejection | `r11-ship-forwarded-reject-FAILED.out` | ring9 | **1** | `reject.forwarded.available` FAIL ("no such id in this run"); see limits |
| helicopter | not copied | | | Accepted helicopter proof is linked, not repeated: [`docs/m5-physical-loss.md`](../../docs/m5-physical-loss.md), [`../m5-loss-structural/`](../m5-loss-structural/), [`../m5-helicopter-lifecycle/`](../m5-helicopter-lifecycle/). No new helicopter run was made for EEDCS-41. The final pack (pending) is the remaining runtime check of the reordered helicopter path |
| shipping API | `api22-shipping-surface.out`, `api-surface.lua` | final-default22 | 0 | `frame`, `report_loss`, `members`, `objects` are functions; `loss_fixture`, `loss_state`, `loss_shape`, `loss_release`, `loss_group`, `dependent_fixture` are nil. Driver DLL hash equals the default22 export |

`runs/*.exit` hold each exit code. Verbose `.err` debug logs and failed
intermediate attempts (`wit-fw`, `wit-fw2`, `wit-fw3`, the green/g/p/w/shape runs, earlier
`api0`/`api13b` start failures) are not retained here.

## Checks (`linux8/`, final source, Linux container)

`cargo fmt --all -- --check` exit 0; `cargo clippy -p eech-dc -p eech-engine --all-targets`
with `-D warnings`, default features, exit 0; the same with `eech-dc/loss-fixture`
exit 0; `cargo test -p eech-engine -p eech-dc --features eech-dc/loss-fixture` exit 0
(eech-dc 3, lua_debug_layout 2, eech-engine 30 passed). Default-feature tests were not
run separately. `eefd0bb0` changes no Rust or C source, so these were not rerun.

## Limits and missing coverage (not passed checks)

- **Forwarded-report rejection has no passing runtime run.** r11 failed for lack of a
  forwarded id. Source dispatch rejects it; no runtime claim is made.
- **Invalid-landing shapes have no runtime run.** Landing/takeoff routes, other counts
  and other landing actions are refused in source and covered at the refusal level
  only; none was exercised through the host.
- **No arbitrary-delay proof.** The fixed-wing case reports after 0 frames; r10 reports
  after 0 extra frames. Delay robustness is listed as not covered in the doc.
- The fixed-wing group has two members, so there is no same-landing FORWARD control.
  The preservation check is a direct state read, not a comparison with a control.
- Fixed-wing setup (`loss_release`, `loss_group`) is explicitly artificial, and an
  observed census found no eligible member naturally.
- Ship, routed and person evidence is from partial-tree builds, as above.
- Duplicate and unknown-id dispatch were exercised only as listed, not on the final
  DLL; the final delta does not touch that dispatch.
- Tacview, baseline and full-file comparison are not part of this record.

`SHA256SUMS` covers every retained file (`sha256sum -c SHA256SUMS`).
