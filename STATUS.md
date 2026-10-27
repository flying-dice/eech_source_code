# D2 — physical aiming/readiness implementation and validation

Authority: live M5 goal11534337v3 and migration ledger14548994v37. D2 owns physical
sensing/visibility, aiming/weapon execution and combat resolution/reports. Campaign
choices and consequences stay in EECH_DC. D3 has not started.

## Delivered and accepted

D1 a7934aa7 is accepted; all15 tasks Done and40/40/full-byte evidence reused.
The supplied-binary destination baseline87915 completed actual0,40/40, complete
accepted recordings equal and all3 post-input manifests match. Do not repeat it
merely for handover.

EEDCS-42/PR101 delivered LOS at3ce17432e; its final report-display recovery and
original exit distinction are retained in the LOS dossier.

EEDCS-43/PR102 merged normally22:28:45UTC at04bf645a96f8277ca54d4a575b77f35b5ee68052.
All9CI passed atd5c080883; merged tree equals that validated head. Runtimea2a041fec
moves physical sensing range/envelope/tolerance/emitter factors to the host;
campaign filtering/ranking/engagement/awareness stay core. All4 native campaigns
completed. Original full runner89376 EXIT1 from child PowerShell's WindowsStore
Python alias is preserved; separate unchanged checker/reporting recovery EXIT0,
same40 checks allPASS. All8 COMPLETE artifacts equal the destination baseline;
all3 post-input processes exit0. Same independent reviewer verified the recovery,
24 public copies/72 original hashes and unchanged check definitions/limits.
Jira43 Done (finalcomment10790); ledger37 verified with all5 report macros unchanged.
Evidence: reference/d2-sensor-detections/extraction/final-pack and
../docs/handover/D2-SENSOR-FINAL-VALIDATION.md from eech-campaign.

## Active EEDCS-44 / draft PR103

Branch codex/d2-weapon-aiming; physical commitsf749878ce and a0d3b6f9c (the latter
routes HUD/player offsets through the host too). Candidate moves readiness
advancement, device slew, weapon/target/intercept vectors, lead/ballistic lookup,
measured angles and person-facing. Package/target/weapon choice and firing policy
stay core. Ballistic-table generation still uses core projectile simulation:
this is remaining D2 weapon-execution coupling, not D3.

Original characterization/evidence is committed in reference/d2-aim-characterization.
Canonical0ee readiness oracle:1962events, physical8d5afe5ea932338f,
decision4e542d029ba7362a; rawSHAc4e9c2b108184243b2bb48034216cad80a7bc9e6225b6229ef8148fe0823ce62.
Clean candidatef749-lf (0dirty,29 BUILDINFO entries independently verified): c-a
and c-b each completed native/driver0, all1962events and COMPLETE raw bytes equal
the original and each other (root comparatoractual0). This is fixture conformance,
not full shipping acceptance. Both a0d3 c2-a/c2-b completed native/driver0;
root compared all1962events/rawbytes exact to the original and each other. Latest results live in external
R:/d2-evidence/aim-extraction-20261026. Development builds/known-different partial
x1 and exact x2 are retained with explicit working-tree provenance limits.

Current lead checkpoint, 27 Oct 19:12 UTC: author source52b6804d3 is committed,
clean and pushed; Claude exec69430 actually exited0 and PID14560 is gone. Reusing
relevant local context6f for one bounded comparator correction, not a duplicate
implementation. Ordinary d3c9a5c5e900s completed native0; the author's complete
three-file comparison matches ab-base. Causal/refusal results below await lead
verification. No full shipping pack or aiming delivery is claimed.

Independent NEW physical source pass at d3c9a5c5e found no confirmed runtime or
boundary defect. P2: compare-person.py accepts incomplete traces/arbitrary true
check renames; the retained actual p2 traces independently have complete passing
verdicts. Correct the checker and recompute existing evidence; no person rerun is
needed for this checker-only change. Do not call the untriggered ballistic bad-draw
mode a native refusal. Clarify the off-map previously-unwritten output instead of
asserting an unreviewed semantic difference is accepted.

Accepted sensing04bf645a is being normally merged here; only docs/evidence differ
from the already-composed sensing runtime. Remaining gates: bounded correction,
lead evidence verification, frozen ordinary snapshot/build/full40checks/all8full
files/post3inputs, finish independent review of NEW evidence, CI/normalmerge and
Jira/Confluence. Then continue remaining D2 weapon execution/combat resolution.

Lane report (author, 27 Oct, source d3c9a5c5e = a0d3b6f9c engine/kernel/module sources + provider driver script + docs; evidence in
`eech-campaign/reference/d2-aim-extraction/`, account in `eech-campaign/docs/m5-aim-extraction.md`): every gate below ran in this lane, native exits the process's own.
Final-head fixture a0d3b6f9c: c2-a and c2-b each native/driver0, 1,962 events, **raw bytes equal the original r-a/r-b** (physical 8d5afe5ea932338f, decision
4e542d029ba7362a). Controlled person case (a0d3b6f9c) target/control/repeat native0, 0 failed checks; equal to the 74ae traces under the declared entry-point mapping
(`place` -> `person_face`), repeat byte-identical. Ordinary 900 s with the ordinary d3c9a5c5e binaries: native0 (1,260 s), recording 11,363,845 / observations 6,629,273 /
metrics 11,777 byte-identical to ab-base, observation check PASS, post-input manifest IDENTICAL. Host authority (provider, 300 s): the reference host's report equals the
original's checkpoint (196 lines); a quarter-rate ready keyframe, tenth-rate device steps, all-invalid vectors and a non-EECH "own model" host each change births/losses/rejections
(185/6 -> 182/6, 177/9, 0/0, 5/1) and the fixture records their answers; the ORIGINAL build is unmoved by the same hosts (equal, 0 requests). Shipping refusals: missing provider
fails at boot; nil, unknown state, ready state 5, phase 7, NaN vector, second draw, negative range and a Lua error stop with `host aim:` and the reason. One mode
(`bad_draw_for_ballistic`) was UNTRIGGERED (no standalone ballistic request occurs): its native exit1 is the driver's own end-of-run assertion, not an engine refusal, and it is not
counted among the refusals; that draw refusal has only an engine unit test behind it.
fmt/clippy(default and aim-fixture)/tests: cargo `CARGO_EXIT=0` for all five steps. Failed experiments kept (x-dev1 physical-hash mismatch from a missing lead-result echo, fixed;
first ordinary-900 attempt stopped in the runner script before any native; first provider runs one frame short). Remaining coupling: device viewpoint/model lookup, target point,
ballistic-table generation (core projectile sim), the HUD sight's shared `get_angle_of_projection*`, rocket-tube roll and package-arc bearing, guided flight intercept
(`wn_move.c`), the refused triangulated range. Unexercised by the natural witnesses: standalone ballistic/intercept, off-map/tiny-range lead, point-blank, player tracking
and HUD offsets (standalone lead at launch WAS reached: 143, all after 420 s, inside the equal 600 s trace). Off-map lead success returns time of flight 0 where the original left
the caller's uninitialised local unwritten (wn_creat decoy/fuze timer): an observed difference, acceptance PENDING with lead/reviewer, runtime unchanged. Person comparison
checker corrected (revision 2, fail-closed, 35-case helper self-test; revision 1 and its records superseded, kept); no native rerun. Cost: +17-19% wall time (about 12.9 M extra crossings per 600 s). Not done here: root's final ordinary build/full pack, independent review, merge, integration of
accepted04bf645a. Next bounded action: independent review of the new physical source and evidence; then root's gates.

## Operator/workflow checkpoint

One implementation lane: Claude CLI-p, exact claude-sonnet-5-5 High, local context
6f729579-d2da-46ae-af38-6e163c42dce4; exec69430 completed0/PID14560 gone.
Same independent Codex reviewer d2_los_review; no nested agents/review fleets.
Check real process state after timeouts. No GitHub auth change/credential copying;
use the logged-in fd-starscream-bot identity and autonomous normal merges as the
human directed. Do not fabricate GitHub self-approval or use an admin bypass.

New reusable skill installed: ~/.codex/skills/eech-regression/SKILL.md; portable
workspace eech-regression-skill-20261026.zip. It covers full retail setup, repo
runner/environment preflight, monitoring, exact comparison, recovery and handover.
Real preflight/independent forward tests passed; no campaign was run for skill tests.
Use its run_regression.py for future full packs: child PowerShell Python is verified,
bytecode disabled before repo imports, busy roots refused, actual native cwd/exits
and post-manifests recorded. Run --preflight-only first with a fresh output, then
another fresh full-run output. Source/bin/root/output argv must use short paths.

R: maps this workspace; keep it while natives use it. Existing WSL2 EECH-D2-Build
and local Docker daemon exec37778 remain; inspect before restart/cleanup. Use
physical C: cwd for WSL and preserve exported tested binaries before cache removal.
Private retail assets/binaries/raw recordings remain outside Git.

After44, complete remaining D2 weapon execution/combat resolution before D3.
No speculative tickets or historical TypeScript/call-map work. External detailed
checkpoint: R:/d2-evidence/ACTIVE-RUNS.json (latest_checkpoint overrides older fields).

## Historical characterization checkpoint (superseded by the current status above)
# D2 — EEDCS-44 physical aiming and readiness

Current task: EEDCS-44, In Progress. Branch `codex/d2-weapon-aiming` was created from
verified accepted integration `3ce17432e4026f85b19ebf06cc582314817ddac2`.
Characterization comes first: no aiming responsibility has moved. Draft PR103
contains the observer and evidence work. Corrected f9f15f52f original600s traces
match all1740 events and raw bytes; its900s control completed native0 with all3
complete artifacts exact against the retained supplied-build baseline. B7 evidence
and its completed exact900s control remain preserved.

Oracle0ee1fa46b additionally measures readiness producers. Both600s runs
completed native/driver0; all1962 events and raw bytes match. Physical signature
8d5afe5ea932338f includes readiness; decision4e542d029ba7362a is unchanged.
Its900s hooks-on control completed native0 (1,194 s) with all3 complete files equal
to the retained supplied-build baseline; the same holds for the b7 and f9 controls.
Root verified all27 readiness-build entries, all28 person-build entries and strict feature-off equivalence for all8
instrumented sources.

Characterization source74ae6a96d, final evidence c2554db2d (pushed). Actual Claude
exec43568 completed0 and PID22636 is gone. Fmt/default-Clippy/aim-Clippy and Docker
all exited0. Independent characterization review found no blocking issue; two P3 reporting
corrections are committed. Root is launching physical extraction in the same relevant
Sonnet5.5High context; no source cut is claimed at launch. See
docs/handover/D2-AIM-CHARACTERIZATION-REVIEW.md.
Some intermediate person-driver traces were overwritten and are explicitly
disclosed; the final target/control/repeat and last continuity failure are retained.

Lane report (author, 74ae6a96d, no cut begun): both Lebanon and Georgia natural
600s witnesses recorded0 person-branch entries, so one bounded controlled case closes
it through the real host path (`lua/aim-person-case.lua`, fixture-only entries that
take and return host ids only; build 74ae6a96d, 0 uncommitted inputs). A real
host-identified person was given a real host-identified target by the campaign's own
target-link call (a precondition, not target-selection policy) and stepped normally,
native0 after21 s: all11 checks pass (branch entered20/20 with a link, facing heading
equals the independently computed bearing, a 1.65 rad turn, the host placed the person
20/20 and adopted exactly the pose sent, the mirror equals the host answer, and after
the link was released all40 later person steps arrived with the attitude the host had
last answered). The control without a link enters the branch0 times. The target case
repeated byte-identically. Remaining unexercised, stated not covered: player/cockpit and
HUD paths; the lead solver's tiny-range, off-map, triangulated and failure exits;
fixed-pitch ballistic lookups; the guided static-point branch; other seeds/durations;
person aiming in natural runs. Also open and recorded in
`eech-campaign/docs/m5-aim-characterization.md`: the lead solver draws the shared
random stream once per moving-target solve (7,676 in 600 s), the weapon-system update
slews on the previous update's vectors, and ballistic-table generation is weapon
simulation. Evidence: `eech-campaign/reference/d2-aim-characterization/`. No physical
cut before the independent review of this characterization. D2 is the only active
delivery epic; D3 has not started.

Authority: live M5 goal11534337v3 and migration ledger14548994v36 (reread20:32UTC). D2 covers physical
sensing/visibility, physical aiming/weapon execution, and combat resolution/reports.
Campaign target and weapon choice, priorities, engagement, awareness and consequences
remain in EECH_DC. Historical TypeScript and old call-map inventory are not scope.

## Accepted and frozen predecessors

- D1 is accepted at a7934aa7; all15 tasks Done,40/40/full-byte evidence reused.
- EEDCS-42 is Done: PR101 merged normally at3ce17432e after all9 CI passes at8bd28294b.
  All40 checks passed; all8 complete artifacts and all3 post-run input manifests match.
  Original runner exit1 was only a final cross-drive report display; unchanged report-only
  recovery exited0, independently checked, originals preserved. Ledgerv34 keeps all5 reports.
- EEDCS-43 is not accepted. Reviewed runtimea2a041fec has resolved source findings,
  passing characterization/causal/refusal/fixture controls and unchanged900s records.
  Root's fresh ordinary build passed and all26 hashes match. Original pack89376
  remains active on Georgia; both full reference recordings and Lebanon regression
  completed. All6 complete reference files and completed Lebanon metrics match the
  baseline. PowerShell postchecks selected the Windows Store Python alias and failed;
  this is an environment failure, not a passed original pack. Failed evidence is
  untouched. A separate unchanged reference-postcheck invocation completed actual0
  with explicit Python3.12.14; complete40-check recovery, all8artifact comparison,
  post-input manifests, independent recovery review and normal merge remain pending.
  Original source/bin/roots stay immutable. Recovery lives under
  `R:/d2-evidence/candidate-pack-sensing-a2a041fec-postcheck-recovery`.
  Future root pack launches use a verified process-local Python command shim; no
  system configuration or authentication changed. All9 CI at the reviewed sensing
  head passed; final delivery checks remain. Sensing is not accepted yet.
- Reviewed sensing branche7e46dcc2 is composed before the first aiming build. This is
  development stacking of runtimea2a041fec, not accepted43 delivery. Delivery43 precedes44;
  any later source correction must be incorporated and its affected evidence renewed.

## Current bounded work

Characterize active physical aiming, intercept/vector production, readiness/device
orientation and person target-facing paths through the real host Lua -> native module.
Current source anchors: weapon.c around700/755/943/1102/1257/2213/2635, ac_wpn.c,
vh_wpn.c, ps_anim.c around311, en_maths.c target-point geometry, readiness producers
in en_anim/ac_anim/vh_anim, and all6 family updates.
Those are starting observations, not a predetermined ABI or proof of runtime coverage.
Observe existing computed values without changing calls, RNG/FPU order or campaign policy.
No physical cut before a passing named characterization and observational control.
Root identified fixture wrappers evaluating unwritten outputs on early exits
before the hook's internal guard. Those exits were absent from the completed600s
witnesses, which remain retained with their exact provenance. Sonnet corrected the
call boundaries at f9f15f52f; root checked the source and all27 fresh build hashes.
Root's stricter feature-off preprocessing check passes all8 sources with compiler
failures propagated and real numeric constants preserved. It supersedes the earlier
broad normalizer; its missing-relative-include setup failure is retained separately.
Person-facing count0 is a coverage
gap, not proof of that physical behavior. See
`../d2-evidence/AIM-LEAD-CHARACTERIZATION-FINDINGS.md`. The unstarted492 final-run
launcher remains lead-gated. Root also rejected an unstarted4500s natural person
search based only on an infantry-loss timestamp, gated its launcher and canceled
only its identified waiting PowerShell process with no child/native. The active
readiness witness and all4 frozen sensing natives were untouched. The controlled
person case replaced that search and ran (lane report above); its source remains
fixture-only. One additional native case ran at a time; no physical cut has begun.

One compact read-only Opus5.5Medium plan completed actual0, with source-grounded
lead corrections recorded in `../d2-evidence/AIM-COMPACT-PLAN-LEAD-DISPOSITION.md`.
It is advisory, not source approval. `../d2-aim-extraction-prompt.txt` is authorized after characterization and review;
the physical implementation stage begins now. The older DRAFT remains unlaunched.

Fresh eventual aiming full-pack roots at
`R:/d2-evidence/candidate-roots-d2-aiming` match all3 repository input manifests,
actualPython0. They have not run and no final aiming candidate exists yet.

One Sonnet5.5 High CLI `-p` implementation lane, one independent Codex reviewer.
No nested agents or review fleet. Root owns Jira/Confluence, final builds/full packs,
publication and normal merges. The user's logged-in identity direction authorizes
fd-starscream-bot publication/normal merging for this work; do not reopen that question,
change authentication, copy credentials, fabricate GitHub self-approval or use admin merge.

Use the unchanged Docker recipe in isolated WSL2 EECH-D2-Build/local Unix-socket daemon.
Current toolchain is recorded per build: Rust/cargo1.99.1, MinGW12-posix, Lua5.1.5.
Keep R: (workspace subst) live while native runs exist; native arguments must avoid spaces.
The final pack launcher explicitly establishes a native R: cwd for report formatting.
Private retail inputs/binaries stay outside Git. Preserve actual source/bin/script/input
identities and exits. No accepted baseline or tolerance changes and no D1 reruns for handover.

Next: characterize, then move the source-derived physical responsibility, run the same
scenario and compare complete Tacview/observations/metrics, investigate differences,
independent review, normal merge, and delivery records. Terrain/model lookup, entity
mirrors, shared RNG, effects and sounds remain honest transitional coupling; authoritative
aiming/ballistic calculations must not be left in core behind an echo callback. Remaining
weapon execution and combat resolution are still D2; do not start D3.

Exact live processes/checkpoints: `../d2-evidence/ACTIVE-RUNS.json`.
