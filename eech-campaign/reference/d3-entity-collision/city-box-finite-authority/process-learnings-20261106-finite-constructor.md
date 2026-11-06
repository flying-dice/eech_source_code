# D3 process review: verified controls and continuing lessons

Verified by Starscream, 3 November 2026. This is a learning record, not a second
live task checkpoint. Current state remains in D3-CURRENT.json.

## Five whys

1. D3 delivery took too long because required production responsibilities remained
   coupled while preparation and long regression runs consumed time.
2. Preparation expanded because captures, numerical prototypes and harness repairs
   were pursued serially without a firm next production increment.
3. Repairs repeated because the initial behavior contract missed actual replay
   inputs, initializer/flag effects, RNG/FPU state and ordering.
4. That did not force a smaller increment because there was no enforced preparation
   limit or clear distinction between a required blocker and optional hardening.
5. The pattern survived continuation because duplicated histories and stale recovery
   entry points obscured the current decision. The coordinator owns these failures.

Regression wall time is separately real. Byte-identical accepted outputs and an
independent review remain required; weakening either does not fix these causes.

## Implemented and verified

- PR113 merged as 581d32a716c46eb2b6999cac82d15284edbb5141. The source record is
  docs/governance/D3-FIVE-WHYS-20261102.md and the policy is EXECUTION.md beside it.
- Agent startup links the policy and active extraction card. The progress guard
  requires a concrete production action and named missing facts. Its two-batch
  preparation limit permits necessary correctness evidence, never unsafe cutover.
- Nine retained guard decisions matched expected exits. Independent review closed
  on 39ee5556333df04d23254da0d561f16bf02488aa. Original receipts:
  D3-execution-control-decisions-20261102.json and
  independent-D3-process-controls-final-39ee55563-20261102.json. Reused, not rerun.
- One author/reviewer, consolidated corrections, focused checks before freezing,
  equivalent-source evidence reuse and actual process-identity checks are policy.
- One compact runtime checkpoint (maximum 12 KiB), exact D: mirror, archived old
  state and pointer-only older entry names support compaction recovery. All ten
  whole-D3 requirements remain preserved in D3-REQUIREMENTS.json.
- Workspace AGENTS.md now directs recovery to that checkpoint before selecting a
  checkout. This addresses the observed stale main checkout, without moving or
  resetting anyone's worktree or interrupting a running check.

## Additional lessons from actual work

- Characterize the caller's branch prerequisites and require a positive measured
  call. Two TREE controls with zero calls remain UNREACHED. The subsequent finite
  authority report is independent-tree-production-authority-eb2-nativef0-20261103.json
  (SHA256 7ad1a698f7996012eda90584661aca717c6b34bf4817f48e2837f6c595fc1dca).
- Capture child-command exits directly. In entity a3/a5 logs the outer shell said
  actual_exit=0 despite a failed Cargo test piped into tail. That outer status is
  not a passing Cargo result. Preserve failures, inspect complete logs and require
  direct per-command exit evidence before clearing a gate. The a8 fingerprint also
  contains a Git worktree-resolution failure; do not call that a valid source
  fingerprint. New entity source/native acceptance remains open.
- Use rg -a or git grep -a for Core C: embedded NULs can hide real source matches.
  Use rg --files or -g globs; PowerShell does not expand wildcard file arguments.
  Never normalize accepted source or output to make searching easier.
- Update the tracked card/CURRENT summary at the sole author's bounded checkpoint.
  As observed at 04:24 UTC, those still described earlier TREE gates while the
  canonical checkpoint correctly recorded finite TREE closure and entity WIP.
  A stale card's successful guard is not proof that its facts are current.

Historical reconciliation above was completed on 5 November: Task50/52 are
normally delivered, TREE/source reports published, and the corrected stationary
entity authority closed in one combined independent pass. Current production
work and remaining gates are only in D3-CURRENT.json; this learning file is not
another live checkpoint. No whole-D3 acceptance follows from this record.

## Additional verified lessons — 5 November 2026

- Inspect normalized connector fields selectively. A GitHub PR response can
  contain the complete historical diff even when only metadata is needed. Print
  head/base/state and targeted body fields, never the whole structured payload.
- Use a fresh bounded Claude context when a long relevant context carries large
  historical transcripts. Supply the exact closed-proof/next-production contract,
  keep one live implementation lane, and verify the actual requested model. This
  does not authorize redoing closed work or guessing unavailable old histories.
- A test-only Lua addition need not rebuild unchanged native code. Prove the exact
  changed-path scope; carry the compiled snapshot identity separately from the
  literal diagnostic commit/blob/SHA. The b24/7ead authority controls exercised
  this distinction successfully; the ordinary accepted scenario was untouched.
- Run one real healthy control before preparing a matrix. After positive measured
  provider/caller/cohort evidence, two independent controls may run concurrently
  on separate fresh manifest-verified roots; keep one writer per log and no cache
  contention. Seven entity controls completed with14 full input checks0 and87
  unchanged payloads each. Do not repeat a war for each internal helper.
- Source review must follow the actual caller's initialization contract. The entity
  P1 was real: a caller leaves normal output storage uninitialized. The fix sends
  no caller output, validates a complete component-write mask and commits only
  original assignments. Preserve the rejected attempt and real mutant failures.
- Close and publish only the measured boundary. The stationary entity witness
  proves changed physical use/refusal; it does not establish moving-candidate
  native parity, per-weapon query attribution, an executed Original narrowphase,
  the remaining physical scan/pose/roster or whole Task49/D3. Those requirements
  survive its finite acceptance and immediately drive the next production change.

## Candidate source review lessons — 5 November 2026

- A synthetic kernel differential can pass while the real C offer and provider/store
  wrapper impose a new invalid precondition. Review420/runtime2d found a valid
  direct-city-building scene0 skip rejected, and geometry required before the
  original tallest-structure skip. Protect original skipped-branch reads through
  actual collaborators, and defer otherwise-unused facts. Do not supply a default
  model to hide missing geometry on a branch that really needs it.
- Source clearance before a fresh Windows/native launch avoided running that known
  bad candidate. Consolidate related preconditions, identity-label and raw-word
  refusal changes in one author batch, then review only changed material/new checks.
- Inspect every error across the complete failed gate before its final retry. Run
  the affected DC/Engine bridge compile and focused strict checks before a feature
  matrix; one package's fix does not imply all test-style failures are corrected.
- A new shipped diagnostic affects the existing script-profile inventory guard.
  Include that guard in focused Lua validation; b24 made61 scripts62, discovered
  later by the full suite. This inventory expectation is distinct from an accepted
  campaign baseline; never alter physical golden outputs to obtain green.
- Pin runtime, diagnostic blob and metadata head separately. The d39 healthy
  preflight correctly stopped before any engine/root write when its author finished
  metadata41e2. Inspect that exact metadata-only delta and the clean tree, then
  reuse the literal diagnostic and frozen binaries; do not rebuild or rerun a
  completed native control for metadata. Preserve the stopped preflight log.
- Read BUILD-INFO's header by lines or its exact standalone `sha256:` section.
  Splitting at the first substring `sha256:` truncates the image digest itself.
  The Candidate payload seal now preserves all eight actual header lines, including
  image ID and compiler versions; all payload hashes still come from exact rows.
- Feature-filter an API inventory before drawing reachability conclusions. The
  lexical `env_case` method is gated on `env-fixture`; its local-player mutation is
  temporary and directly calls rotor sounds, then restores state. Conversely,
  `set_raw_gunship_entity` is another positive writer through session unpacking.
  Bound fresh shipping, test fixtures and save/restore separately. Read exact Git
  blobs when another agent is editing the worktree; a mixed live read is not a
  coherent frozen source identity. Pinned follow-up: player-route facts4b3,11files.
- Separate tested raw bytes from committed bytes for every input, not only known
  C exceptions. The Scan gate's2918 raw/filtered maps matched before/after/commit;
  Root's all-input join additionally found3 unchanged testLua files with working
  CRLF and committed LF.2915 raw inputs and all18 changed files were Git-exact;
  the3 testLua qualifications are distinct from3 Core filtered-hash exceptions.
  Keep exact bytes/hashes and qualify evidence; never normalize a recording,
  accepted input or source file to make this assertion pass. Receipt8864 records it.
- Overlap the one source review with a final gate only after the code input set is
  frozen. Hold clearance until actual terminal exits, unchanged raw/filtered maps
  and committed source binding are joined. The Scan review used this overlap and
  held its report until all16 exits0/runtime8b852/meta40ef/qualifications were
  verified. Windows native execution still waits for explicit source clearance.


5 November, pose observer review overlap: a noninteractive Claude -p invocation cannot receive Root's independent review findings while it is still running. Early review only saves rework when the author checks the canonical review state/new findings at CODE-FROZEN before expensive final gates, or returns at that bounded checkpoint. The pose author started the full gate with already-known reviewer blockers queued outside its invocation. Future batches must read D3-CURRENT review/explicit findings file at freeze before final validation; batch corrections first. Do not stop a healthy producer merely to save gate time; retain its actual results at their source identity. No additional review fleet or messaging protocol is needed.

5 November16:30 implementation of freeze review-join control verified: same Sonnet session reread canonical review state and full findings file, noticed16:07/16:09 additions, and applied them BEFORE final expensive gate (firstcode12ce81bf8). This joins overlapping reviewer findings without another model lane, interrupt/restart or stale-source validation.

5 November pose correction build-cost learning: the six-command 'early' script rebuilt four fixture profiles before all review comments/source documentation were joined; a test-only fix replayed it, then a header-comment correction triggered the final affected builds. Next production batch must name ONLY default four-crate cargo check and the real shared C dialogue control as initial prechecks, inspect all errors, then join review at CODE-FROZEN. Defer the feature fixture matrix until that final freeze; do not repeat it for test-only changes. Root should put the exact small initial command set into the Claude prompt rather than let 'compile early' expand to the final matrix. This does not waive final exact-source C validation, Windows builds, review or regression.

5 November19:07: freeze-before-final-matrix control IMPLEMENTED. The sole Sonnet correction returned clean pushed runtime f3107332b/metadata41447 with focused checks and terminal0 at18:51:49Z, without the full fixture/clippy matrix. Root joined the same reviewer's corrected-source findings, created a fresh detached no-hardlinks/no-alternates long-path-enabled clone, captured its actual raw/filtered source map, mounted source read-only and started the selected16-step final gate ONCE at19:01:49Z. Final clearance still waits for terminal results and after-map equality. The worktree's three testLua CRLF qualifications are not automatically inherited by a fresh Git checkout; qualify the actual snapshot. No test/build rerun for the caller-disposition prose correction.

5 November19:32: risk-based retry implemented. Original frozen f310 matrix actual1 retains15passes and ONE test-only strict-lint101. Same Sonnet changed only cfg(test) parity expression (production/C unchanged), Root verified exact byte replacement and all2920rawGit inputs/before-after maps for both snapshots, then ran ONLYfmt+2classifier tests+strict4crateclippy at ea2b (all0). SAME reviewer0a00 explicitly joined this reuse and cleared native launch; no16green relabel/full-matrix replay. BothWindows0/all88; newordinary THREE COMPLETE files remain literal80fd. Healthy held vector now6091/6091 with0mismatches and parentphysicalsignature equal; diagnostic changes are reported separately from physical equality.

5 November19:46: a selected target/link is not an aimed physical launch. Actual movingA found a real moving target but targetquestions/NeedUpdated/test0 and hit_land6steps; preserve UNREACHED, noB/Original or parameter search. Source trace of fixturelaunch -> weaponbirth -> turret world viewpoint -> campaign per-frameaim exposed the missing engagement precondition. The next witness observes actual campaign aimvectors/source-target-weapon/frame and last host motion, never guesses aim from a target link or substitutes global counts for per-target evidence. Source plans must verify the complete immediate physical caller chain before asserting scenario viability. Default3COMPLETEfiles remain literal80fd; failed probe is not a solver regression or positive authority.
Windows paths are case-insensitive: prompt and result names must differ beyond letter case. The moving-plan prompt was preserved as a distinct -PROMPT filename before the controller wrote its uppercase-named result. Check terminal receipt before reading a result; an existing path may still contain the prompt while a producer is running.

5 November20:24: stop natural encounter probing after the fixed-checkpoint absence is established. First moving target wasn't on the projectile path; genuine campaign-engagement selection then had no eligiblecase among6aimsources (49aa, prelaunchUNREACHED). Lead recorded ONE bounded direct real-query SERVICE fixture decision in D3-MOVING-QUERY-FIXTURE-DECISION-20261105.md, explicitly preserving the separate naturalweapon gap and wholeD3scope. This is a named missing-capability exception, not an open-ended diagnostic programme; no seed/time/distance search or acceptance relaxation. The active ordinary40/all8 pack is isolated and continues. Prefix comparisons are partial early-regression signals only; final checks/exits/full8 and inputs remain mandatory.
For a NEW bounded implementation batch after this checkpoint, prefer a fresh compact SonnetHigh source-derived context when the old resumed context is hundreds of thousands of tokens. The present relevant session has about312k cached input tokens; keep its healthy current producer running, do not restart it just to save time. Carry exact current contract/source/review/limits, not the full diagnostic history; one implementation lane/model remains. No silent model substitution.

5 November20:52 production lesson (rootcause stillUNKNOWN):30-second observed offers were onlyalivekind1, while the mandatory mirror applies to later actualmobile/lifetime paths. Source proof that every normal creator binds before visibility establishes record availability, not semantic equivalence of every motion field or coherence at every later writer/transition. The accepted fullpack exposed held-motionbit4 at~380seconds despite short3file equality. Characterize the first actual counterexample and the exact writer/retention boundary; add the minimal real-collaborator guard, no guessedactor/zero/formula/queryfallback or mirror suppression. Start established product regression once the compiledcutover/defaultshort is sound enough, overlap fixture-only work safely, and preserve partial prefixes only as early signals. Baseline governance is not an environment fix for a real nativefatal; failed40pack remainsfailed.
Fresh-context process improvement IMPLEMENTED20:43:56: previous soleSonnet terminal0, newlocal session d6d74b7d-71da-4ebc-8a66-60e62fb041fa uses SAMEclaude-sonnet-5-5 High/bypass/6localtools/noMCP/agents. Compact current contract/source/report/limits replaces312k-token resumed history; oldcontexts retained. No healthyproducer was killed or silently substituted. Priority actualproductionfix thenknownfinitefixturecorrections, returnfrozen beforefinalmatrix/fullpackrestart.

5 November21:27 actual production counterexample confirmed (diagnostic bd6e): the first failing collidable actor is Person, stage0, alive, mask20. Active HOSTED apply_person_pose writes the completed Host solver result into Core, but settle_person only updated pose and never retained Outcome.motion_vector in the canonical record. Initial zero therefore survived3752 real completed walking steps. Same positions did not imply motion coherence; attitude*speed is also a different vector and cannot repair this. The unchanged private trace/source/bin/consumedroot remain external; pre/post1179-input checks0 and88payloads resealed, native/helper1 are expected reproductions, not acceptance. The independent source review found the active hosted result application before the runtime trace; reading legacy #else movement bodies alone had incorrectly dispositioned Person as seed-held/not stepped.
Process improvement: for a canonical physical-state extraction, characterize every ACTIVE host result-application boundary that writes the field, including completed multi-phase results. Record availability and creator coverage are different contracts. A meaningful regression compares the real solver result applied to Core with the SAME canonical host record; cover direct completion, terrain-follow completion, and pending/early-return preservation. Fix by retaining that actual result at the real boundary, never by query-time Core refresh or a formula. Root authorized this source-proven correction before the trace completed so a healthy diagnostic need not block a known production fix; actual runtime-rootcause claims still waited for the trace. Same lane/reviewer and unchanged full40/all8 remain required. Diagnostic Windows cache removed21:29 only after terminal native/helper, owner-empty Docker inspection and88payload rehash; exported artifacts retained.

5 November21:48 check-scope correction: `cargo clippy --locked --all-targets` at the workspace root includes EIGHT crates, including unchanged historical eech-sys/campaign/harness, not the FOUR native migration crates. The old eech-sys extraction added6m20s of avoidable build/IO work. Name exact packages in every compile/test/lint command; the word "four" in a log filename is not an execution constraint. Use `-p eech-engine-sys -p eech-engine -p eech-dc -p eech-world`. Root's frozen correction runner implements explicit package scopes for all five selected checks; it never replays all16 profiles. A stop request to ONLY the unintended stage raced natural completion: original d13CargoFinished/footer and exit0 remain literal, with concurrent-stop qualification in cancel-out-of-scope-workspace-lint-20261105.json. No failed/cancelled exit was invented; author/native processes were preserved. Subsequent source-test-only fixes should reuse applicable passes and retry affected checks, not expand another full workspace sweep.

5 November23:00 actual moving query lesson: HostedA and OriginalA are positive and physically identical (three target hits, Hosted3NeedUpdated/3tests/3moving16). The lifted B completed the same rays coherently but had0lift attempts/refusals and unchanged outputs: correctly UNREACHED, not changed-pose authority. A real engine frame and a nonzero returned motion vector do not establish that this actor receives the optional physical input a planned variation needs. The live C bridge supplies routed terrain only on specific waypoint/guide paths, and the old selector did not establish that input. The zeros do not prove which outer predicate failed. Fix only real callback eligibility/provenance at the fixed configuration/identity order; credit actual B-window input application, no synthetic terrain/pose, timing hunt, guard relaxation or rerun of the isolated ordinary pack. New tests must include absent optional inputs and prelaunch noeligible, not only always-eligible mock steps. Source + actual reportd2a1 records the inference boundary; one compact SonnetHigh lane implements the witness correction with only affected Lua tests. Complete A/B provenance requires completed positive A plus SAME source/target/rays/config/input manifests and the actual supplied physical signature, not caller text or script0 alone.

5Nov23:29 terrain eligibility experiment CLOSED: fixed30s actual0d76 has80moving/0terrain-input, honestprelaunchUNREACHED/noquery. Do not repeat prerequisite-only witnesses or tuneactor/time/geometry. For a next configured-input query control, review an input actually present, transform onlythatfield inrealcallback/oneframe/delegateonce/SAMEMovers/normalCoreapply, actual changedphysicalresult+coherencerequired. Describe changeddesired_velocity as changedsolverinput, not failuretofulfillanunchangedcommand. Realproductionbacking remainsnext; noopen-endeddiagnostics.

6Nov finite-control closure: reviewed6fb/compiled3475 A/B/OriginalA/OriginalB all0, exact target/rays/config/inputs and authentic actual helper+native A signatures; all3 contact-point fields change while both Original pairs are literal-equal. Characterize which physical fields changed rather than calling any changed hash a hit/decision change. Received-demand variation must be labeled as changed solver input. The terrain prerequisite experiment stays CLOSED UNREACHED/no tuning; currentcard now records2 witness-only batches and requires productioncut/namedblocker. Nextsectorroster lane started from9ba with complete writer/order/pending/restore/ID constraints; a compact plan is unapproved input and cannot turn active40pack into a falsepass or replace validunpack with refusal. Full3475pack stillactive/prefix613only. Allsource/actualproofs literal inGit9ba; rawphysicaldata external.

6Nov01:24 source review-before-native implemented: SAME7becpass consolidatedexactly3groups(realdriverwirecontract/fullFPrawvalidation/standingkeyallphases) beforeANYWindowbuild/native. Earlyfindings queuedthroughcanonicalfreezejoin. PriorhealthySonnetended0; actualfinalcontext533k, so ONEfreshcompactSonnetHighcorrectsall3withonlyaffectedWorld/Luachecks; unchangedCpassesreuse. No fullmatrixreplayforLua-onlychanges orclippedaggregatecounts. Directchildstatus/separaterawstreams replacecombinedclobberingwhere needed.

6Nov01:55 verified input-map acceleration IMPLEMENTED: oldmapperstarts~7900Gitprocessesfor3952files. Newsector-roster-inputmap-batched-20261106.py same-schema batchedhash-object/cat-file plusrawrereadchangecheck: actual0/2.718806s,3951unchangedfullrowsexactpriorreport+1changedtestrowindividuallychecked;all7existingraw/filterpathsverified. Doesnotnormalizecode/evidence orrelaxidentitychecks. Oldbefore/aftermapsretained; usefastmapperforfrozenfinalbeforeafter, donotrestarthealthyoldcapture. Directfocusedchildexits/rawseparatestreams alsoimplemented.

6 Nov 02:20 process controls applied during roster extraction: prove the NEW backing dependency, not merely a host response that the previous Core-backed implementation also accepted. SAME reviewer required ONE real provider REMOVE of the actually served key, followed by the real next-ENTRY ended-key refusal, with Core untouched and no successful incoherent run. Capture this distinguishing control in the initial extraction contract next time. Test-only Lua changes remain a separately Git-bound literal on immutable compiled5182, avoiding a repeated C/Windows build.

6 Nov Root audit correction learning: inspect the complete public source declarations AND actual archive member list before writing symbol-set assertions. Roster functions live in separate mb_hostplace.o (five functions including the resolver), not the eech_engine.o wrapper. Both mistaken Root finishers/failed assertions were preserved; a single narrow read-only member-binding v2 and a corrected five-symbol parser establish ordinary/hosted inclusion and Original exclusion. No product/build/baseline changed. Prefer semantic proof of the actual bound object over assumed amalgamation or derived flags alone.

6 Nov 02:57 Root context-selection correction: before launching a new substantive implementation batch, inspect the LAST assistant actual usage, including input_tokens + cache_creation_input_tokens + cache_read_input_tokens. Do not estimate context size from JSONL file length or assume a previously fresh context is still small. City batch987c reached609480 actual input tokens; it is healthy and must finish without a size-only restart. For the NEXT substantial batch, use ONE fresh compact Sonnet5.5High context when prior input is already large (operational guide400k+), with the exact source/review/checkpoint prompt. Relevant tiny corrections may resume; old unavailable session IDs still prove nothing. Record observed context and fresh/resume rationale in the launch receipt. This is a delivery-process check, not a model/context-limit claim or permission flow.


6 November fixed/static final-chain and authority review learning (source95, review190e): Root initially selected the earlier *-final chain; the frozen report declared fixedstatic2 after two later edits. Always read the final checkpoint FIRST and derive receipt/map identities from it, then join them to the exact code commit. A green chain may be superseded. git hash-object --stdin-paths supplies FILTERED identity and cannot establish raw byte equality. The implemented Root binding (fixed-static-final-raw-source-binding-95a10-20261106.json, SHA42777e99...) compares actual raw SHA256/size with clean Git blob bytes: all33changed files equal,3949of3952total equal,three inherited Lua raw differences and four filter-only paths qualified. Keep the historical report literal and qualify misleading raw_equals fields. New checkpoint generators must compute raw equality directly.

Same review found two source gaps before any95Windows/native build: actual recorded family must decide canonical-state requirements, never Core mirror discriminators; and a fault counter/final failed verdict is not a current mutation-admission guard. Tests need actual fixed-family Never plus Corefalse/falseCITY and a wrong-served live key the real provider could otherwise answer. Real faults or unrelated fatal errors must not become UNREACHED. One fresh SonnetHigh correction batches both, with unchanged writer/kernel/FP evidence reused; previous718102-token producer was allowed to finish and verified terminal before the fresh context. Immutable code95 review safely overlapped metadata, with final verdict joined to final checkpoint/terminal; no known-bad Windows/native work was started.


6 Nov08:53 fixed-state actual recovery lessons IMPLEMENTED: inspect actual process trees before reporting a summarized job as running. Both old trees were absent, so preserve interrupted outputs and unknown exits; never invent a completed or failed native status. Resume only unfinished validation on fresh roots; reuse completed short and healthy. Current fullpack runs current source, not a replay of superseded3475. Boot/prearmed FIXED_STATE reports populate the SAME held map; fixed_stated counts armed reports only. Root negative launch checker now uses real acknowledged boot state + current canonical join/NeedPlacement/stationary eligibility and never requires an armed writer. Retain true healthy native/helper/outer0/0/1 separately from inner stage0 and negative0/0/0; safe projections must not promote an inner stage exit to the outer exit. Derive inputmap repo/head_rev arguments from the baseline capture to preserve literal metadata; same file rows with slash/HEAD-label differences are source-equal but not literal report equality. All checker failures/corrected projections retained, no product or baseline change.


6 Nov11:00 CITY finite closure lessons IMPLEMENTED: derive diagnostic counts from the FINAL direct stdout, not an earlier green run or a frozen prose summary. Root sourcegate95bb parses619 final C expectations; earlier618 summaries remain literal and explicitly qualified, no source/test rerun. Distinguish the original repofrozen cc6e bytes from final externalb42e that adds publication metadata; strict semantic join proves only those added fields changed, and oldhash guard stops before clone. Preserve both identities. A valid finite canonical-state proof does not move physical construction/predicates: source45e/actual47fe explicitly keep ct_msgs model lookup/rotation/offset/aggregation and fx_int flags REQUIRED D3; card/STATUS/ledger163 now name these next before sectorheight. Next lane uses locallyverified NEWcompactOpus110230 context, explicit --resume plus Sonnet5.5High and verified actual model; previous549118-token Sonnet was allowed to finish and not restarted/resumed. The plan is unapproved input: next implementation must characterize original FPstatus/control/rounding and BOOT availability before trusting assumptions, use no postfatal physical reads, and classify expected protocol-counter deltas without normalizing physical outcomes.


CITY constructor review controls (6 November, 2026-11-06T12:06:08.848620+00:00): implemented process controls, product corrections still pending final SAME review. Root holds Windows/native while source findings remain, preserving clean f06 snapshot and all prior failures. Exact raw/Git/check binding171330 reuses9actual directchildren0/1367passed test executions (including the separately repeated FP characterization) and20literal Git proofs without Root Cargo replay.

Next correction is ONE fresh compact Sonnet5.5High context only after the final full review and actual prior terminal0/resultfalse; prior actual input568481 justified fresh context, not a healthy restart. Prompt now explicitly reads only FIRST CURRENT STATUS/card decisions and relevant source blocks; programmatically join maps/log hashes instead of printing entire histories or2MBmaps into context. This compact-reading control is implemented in d3-city-box-review-correction-IMPLEMENT-PROMPT-20261106.txt.

Known review lessons to implement in that batch: a live child and live city are not a CURRENT relationship; test a wrong LIVE parent before mutation plus legitimate relink/anchor aliases. Reuse strict six-word parsing, and treat every fatal without acknowledged control withdrawal as FAILURE, never clean UNREACHED. Shipping ASSERT only logs: validate finite expanded axes before ANY output/mirror write, retaining the separate valid initial inverted sentinel. Preserve original two-corner union. REQUIRED_CITY_BOX_LIFECYCLE_RETENTION is explicitly whole-D3 work: a new city record currently seeds its prior box from the sole Core mirror after an empty relink/restore. Finite numeric/BOOT authority cannot close or erase that requirement. No accepted baseline/config/tolerance changed.


6 November 2026-11-06T13:16:33.094814+00:00 finite constructor controls VERIFIED: SAME source50eee and actualf267 close the two P2 source groups and the finite constructor/currentBOOT dependency. Wrong-LIVE-parent admission, strict six-word grammar, any unacknowledged boot fatal as FAILURE, and raw numeric expanded-axis checks before ANY output write are implemented and tested. Controlled f06 RED restored exact old header after the fix was initially written; do not relabel it pristine whole-tree evidence. Three Windows profiles0/88/actual13wrappers, ordinary3 COMPLETE files literal80fd, healthy54825prelaunch expansions (not unique writers/objects), exactly one shared counter83408->28583 and180Bterminal literal; real FIRSTjoinedBOOTwithdrawal1/answered0/refusal, all3case triples0. AllsixFULL1179manifestbytes and3953source maps literal. No additional FPprogramme or native malformed-answer experiment.

Source/check reuse stays scoped: current8children0/300passing result instances are not a newly run1152-current matrix; unchanged f06 numerical inputs keep parent identity. Healthy/negative evidence does NOT close REQUIRED_CITY_BOX_LIFECYCLE_RETENTION. It remains mandatory across actual empty-city relink/restore, alongside next physical flags and allten D3 requirements. Closed Windows cache cleanup retained12actual boundCOFF/30proof with3x88rehash/0owners before removing only its named volume. Root checkpoint size guards stop before write; finished producer details belong in receipts/Git/archives so the next active checkpoint keeps headroom and cannot advertise an old running process.
