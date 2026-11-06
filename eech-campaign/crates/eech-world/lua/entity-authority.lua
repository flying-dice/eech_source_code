-- TEST-ONLY host-authority and refusal witness of the production ENTITY_COLLISION narrow-phase handoff (EEDCS-49, D3). Needs a DLL built with the `weapon-fixture` feature (the controlled launch of
-- lua/weapon-case.lua) and an eech-world with `host.model` and `host.create_exclusive`.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's MODEL provider (`host.model`) -> back into a REAL weapon
-- launched through the campaign's own launch call. The hosted weapon flight asks the campaign for the entity-collision lookup (wn_move.c CAMPAIGN_STEP_LOOKUP_COLLISION), which calls
-- get_line_of_sight_collision_entity; for each collision CANDIDATE whose broad phase (still Core: the sector scan and a box of radius + 10 m) passes, the hosted narrow phase asks the HOST
-- `entity_collision` through the model provider (mb_hostentity.h). The module has no model transform or intersection behind that question.
--
-- WHAT THE QUESTION AND THE ANSWER ARE (the corrected shape, mb_hostmodel.h ENTITY_COLLISION): the request's text place `key` holds 18 (a stationary candidate) or 22 (a moving one)
-- comma-separated float bit patterns `0x%08x`: the segment's OLD end (3) and NEW end (3), then the candidate's pose. The caller's intercept point and normal are OUTPUTS and are not sent. The
-- answer is { result = "hit" | "miss", mask = 0..63, p = six bit patterns, name = the scene's compiled name }: `mask` says which output components the original assigned (bits 0 to 2 the
-- point, 3 to 5 the normal; each vector whole, a point implies a normal, a hit is 63), `p` is the point then the normal (zero for an unassigned component). The campaign writes only the
-- assigned components after checking the whole reply.
--
-- BRANCH PRECONDITIONS, from source (nothing here is a guess about a distance):
--   1. the weapon step asks the entity lookup only when the weapon is a server weapon with in-flight collision checking AND (the step is at high collision precision OR the weapon has no target)
--      (wn_flight.c). The launch links the case target for the launch, so the precision path is needed. The existing protocol's VIEW_WEAPON answer is the same configured input as in
--      tree-authority.lua (below): `precision_high` must rise, and the fixture counter `contact_any_tests` must rise, or the actual entity caller was not reached (UNREACHED).
--   2. get_line_of_sight_collision_entity excludes only the launcher (the weapon's los_target is NULL) and visits the sector window of the segment widened by 150 m; a candidate gets a
--      narrow-phase question only if its type is collidable (a vehicle or an aircraft is a mobile candidate, a structure a fixed one; a PERSON has neither flag) and the segment's box meets
--      the candidate's box of radius + 10 m. So the case is the NEAREST non-person enemy of any launcher that has an unguided ballistic weapon in stock: the pair with the smallest distance,
--      so that the target itself is a broad-phase candidate of the weapon's first step. A war in which no candidate ever passes the broad phase makes ZERO questions: that is UNREACHED (the
--      broad phase and the sector order are still the campaign's), never authority.
--
-- TWO PROFILES, named by `profile=`:
--   hosted (default)   the module as shipped: built with `weapon-fixture` and NOT `collision-fixture`.
--   original           the explicitly original characterisation: built with `weapon-fixture` AND `collision-fixture`, whose narrow phase is the unmoved original (EECH_CAMPAIGN_HOST_ENTITY is not
--                      defined). Only mode=control. It is the capability RED of the host route: the fixture API is PRESENT, the weapon step's entity lookup was made (`contact_any_tests` rose,
--                      so the actual entity-collision caller was reached) and the host was asked no entity_collision question. That is reported as `CAPABILITY RED` and the process FAILS on
--                      purpose (the expected red). A module without the fixture API, or a run in which the weapon step made no entity test, is `UNREACHED`: the call this RED is about never
--                      happened. The counter cannot show that the original narrow phase itself ran (it counts the caller's lookup); the row says how many lookups hit.
--
-- One bounded fresh war per process. The provider is the host's own (host.model), delegated to unchanged and counted, until the case weapon is launched ("armed"); from then on the chosen mode
-- may answer an entity_collision question differently. Every other operation is ALWAYS the host's own answer, byte for byte:
--
--   control           nothing is altered: the healthy same-DLL control. Self-asserted from what the provider SEES after the launch: at least one entity_collision question (else UNREACHED),
--                     each request is 18 or 22 well-formed patterns, each real reply is well formed (result hit or miss, a mask inside the domain with whole vectors and a point only with a normal,
--                     a hit with mask 63, zero words for what the mask does not assign, a name), and the questions relate to the fixture's own counters (the lookups the weapon step made,
--                     the hits among them).
--   forced_hit        every armed question is answered HIT with mask 63: the point is the request's own OLD end and the normal is up. The real answer is still asked of the host (its name is the
--                     echo the campaign requires; its result is discarded). A VALID changed answer: the weapon step's actual hit count (contact_any_hits) must rise, and the cohort's PHYSICAL
--                     signature (type, code, steps, position, velocity, lifetime of each case weapon, not its ordinal) is recorded for comparison with the control.
--   forced_miss       every armed question is answered MISS with mask 56 (the normal assigned, the point not): a VALID accepted miss that writes only the normal. No hit may then reach the
--                     weapon step (contact_any_hits does not rise). If the control had no hits this is valid MISS coverage, not a measured changed physical outcome, and the row says so.
--   withheld_entity   the provider raises on the first armed question:                                  [model-provider:entity_collision.callback]
--   bad_write_mask    a HIT answered with mask 56 (a hit has assigned both vectors):                     [model-provider:entity_collision.write_mask]
--   bad_answer        a hit answered with mask 64 (outside the six components):                         [model-provider:entity_collision.answer]
--
-- THE ENTITY_CANDIDATE PATH (third slice; mb_hostmodel.h ENTITY_CANDIDATE, 44 facts, 7 states). In the shipped module the scan OFFERS each candidate to the host (`entity_candidate`) before any
-- narrow phase: the host decides skip / need_updated / need_fixed / need_placement / need_normal / buildings / test, and only a `test` is followed by the `entity_collision` question above. The request's
-- scene is a number >= 0 (0 until a shape was read) and its text place holds 44 comma-separated 0x%08x patterns; nothing else is sent. A real reply is { result = <one of the seven>, name = the scene's
-- compiled name (EMPTY for scene 0) } and, for `test` only, p = 12 or 16 patterns (the narrow phase's candidate pose). The wrapper counts the armed candidate questions, classifies their real answers
-- (global to the war: the provider carries no weapon identity) and delegates the operation byte for byte in every mode that does not alter it. Zero candidate questions is UNREACHED, never authority.
--
--   candidate_real        PAIRED CONTROL. The candidate answers are the host's REAL ones; entity_collision is answered HIT (mask 63, at the request's OLD end, the real name) whenever asked. That second answer is
--                         a CONFIGURED SECONDARY host input, held identically in BOTH paired modes, so that a candidate decision has a physical consequence to change without any guessed geometry: the real
--                         scan must reach the narrow phase (a `test` decision) and the forced hit must reach the weapon step's hit count (contact_any_hits).
--   candidate_skip        every armed candidate question is answered a valid SKIP (the real answer is asked first for its name echo) and entity_collision is answered the SAME configured HIT if it is ever
--                         asked. It must receive positive candidate questions, and the skip must prevent every downstream narrow-phase question and every weapon entity hit: the cohort's physical signature
--                         is then recorded for comparison with candidate_real by the caller (a cross-run comparison, never claimed from one process). A diagnostic world variation, not a source change.
--   withheld_candidate    the provider raises on the first armed candidate question:                         [model-provider:entity_candidate.callback]
--   bad_candidate_state   the first armed candidate question whose facts say it is not a city is answered `buildings` (legal only for a city): [model-provider:entity_candidate.state]
--   bad_candidate_identity the first armed candidate question is answered with the real state under another scene's name: [model-provider:entity_candidate.identity]
-- The old entity_collision modes above are unchanged in what they alter. `control` is the NATURAL healthy control (nothing altered, no configured secondary input) and is the first thing Root runs;
-- the paired modes are kept apart from it and are compared with each other, never with the natural control.
--
-- THE ENTITY_SCAN PATH (fourth slice; mb_hostmodel.h ENTITY_SCAN, 16 facts, 5 states). In the shipped module the host drives the ENUMERATION of the scan (the sector window, the traversal, the exclusions and the first-hit
-- end) through `entity_scan` questions BEFORE any candidate is offered; the campaign only serves the host's cursor over its own sector lists (a transitional backing dependency). The request is scene 0 (a number), no
-- depth, kind or definition, and the text place holds 16 comma-separated 0x%08x patterns: the segment (6), the sector side and the world map (5), the PHASE (0 BEGIN, 1 an entry is served, 2 the list has no more
-- entries, 3 the offered entry is resolved), the cursor sector x and z the host last gave, whether the served entry is the source or the target, whether the offered entry hit. A real reply is { result } with `result` one
-- of first, next, offer, done_none, done_hit; `first` alone carries `p`, two patterns (the sector x and z, whole numbers). The host also owns WHICH entities stand in each sector and in what order (sixth slice): `first` and
-- `next` carry the roster KEY of the entry to serve as `name` (empty when the list has no more; a roster key is 1 to 20 decimal digits without a leading zero), every other state carries no `p` and no name, and the question that
-- follows a served entry (phases 1 and 3) names that entry in the identity place after the 16 patterns (one space, then the key) and no other scan phase carries one. The state each phase and its facts allow is the kernel's
-- (an entry that is not excluded is offered, a hit is done_hit). The roster's EVENTS share the question: phase 4 INSERT (the position, the entity type, the side length, a hosted mobile's identity; answered `placed` with the
-- key the host issued), 5 REMOVE (the key in the identity place; `removed`), 6 RESET (`cleared`); all other facts are zero. The wrapper joins every key it sees (a key is issued once, an ended entry is never served, stood at or
-- removed again, and the scan stands at the entry it was last served), counts the ARMED scan questions by phase and the roster events SEPARATELY (an event is never a query scan, is never altered and credits no scan control),
-- classifies their real answers (global to the war: no weapon identity) and delegates the operation byte for byte in every mode that does not alter it. Zero scan questions is UNREACHED, never authority.
--
--   scan_real             PAIRED CONTROL. The scan answers are the host's REAL ones; entity_collision is answered HIT (the existing forced-hit answer) whenever asked. That second answer is a CONFIGURED SECONDARY host
--                         input held identically in BOTH paired modes. The real enumeration must reach the candidate dialogue, the narrow phase and the weapon step's hit count.
--   scan_none             every armed BEGIN question is answered a valid DONE_NONE (an empty enumeration) and the SAME configured entity_collision hit if it is ever asked. It must receive positive scan questions and
--                         prevent EVERY downstream candidate question, narrow-phase question and entity hit; the caller compares the cohort's physical signature with scan_real (never with the natural control).
--   withheld_scan         the provider raises on the first armed scan question:                                  [model-provider:entity_scan.callback]
--   bad_scan_state        the first armed BEGIN question is answered `next` (illegal at the start of a scan):       [model-provider:entity_scan.state]
--   bad_scan_cursor       the first armed real FIRST is answered with a fractional sector x (0.5), the real z:  [model-provider:entity_scan.cursor]
--   withdrawn_backing     HOST-SIDE, no answer is altered: at the first real FIRST that serves an entry the host calls the SAME provider's REMOVE lifecycle event (phase 5, the key in the identity place) on that key,
--                         requires `removed`, and hands the FIRST reply on completely unchanged (the campaign's lists, the entity and its physics are untouched). The next real ENTRY stands at the ended key and the
--                         provider refuses it by name: [model-provider:entity_scan.refused] [roster.key] ... has ended. Credited only by the acknowledged removal, the entry continuation as the next question and that
--                         refusal as the only fault (reported safely as roster_withdrawn, roster_withdrawn_next_phase and roster_withdrawn_cause); the withdrawal is counted apart from the campaign's roster events.
-- `control` stays the NATURAL healthy control (nothing altered, no configured secondary input); it now also requires a positive scan, candidate and narrow-phase chain. The earlier modes keep their own meaning.
--
-- THE HOST'S PHYSICAL RECORD DECIDES A MOBILE CANDIDATE'S POSE (sixth slice). An ENTITY_CANDIDATE question of a MOBILE candidate (an aircraft or a vehicle of a sector list; never a building listed under a city) now carries the candidate's
-- semantic host identity after the 44 facts (one space, then the identity, 1 to 128 bytes): the host takes the candidate's position, motion vector and attitude from ITS OWN canonical record of that identity (the one Movers registry the movement
-- callbacks step), never from the facts' words for them, which the campaign still reads and ENTITY_POSE compares with the record (a difference stops the engine by `entity_pose.mirror`). A mobile candidate the host holds no record of is
-- refused by name (`entity_candidate.refused`, "candidate record"). The witness counts the mobile candidate questions with and without an identity (`candidate_mobile_named`, `candidate_mobile_unnamed`): the natural control requires none
-- without one. Two HOST-SIDE modes act through the host's own movement callback, before the question is delegated:
--   withdrawn_record      the host retires the identity of the first armed mobile candidate question that named one (the campaign's report that the incarnation ended, made by the host): the real provider then refuses by name
--                         [model-provider:entity_candidate.refused] "retired". Nothing is asked after the stop.
--   displaced_record      the host places that candidate's record somewhere other than where the campaign holds it (a host-only record change): the next observation reports a position difference and the engine stops
--                         [model-provider:entity_pose.mirror]. A host-only pose cannot pass the conformance comparison, which is exactly what this shows.
--
-- THE HOST'S OWN MOVEMENT CHANGES WHAT THE SOLVER DECIDES FROM (two completion modes; neither is a refusal and neither is claimed by the two refusal modes above, which only STOP the engine). The engine's own movement
-- callback is the host's (`move_vehicle = hosted_move`, a pass-through that logs each `move` answer per identity). `moving_control` and `terrain_lift` select a DIFFERENT case from the nearest-enemy rule: the nearest enemy of
-- a launcher in an ENGAGEMENT THE CAMPAIGN ITSELF HAS SET UP: the unchanged aim callback observes each turret's `vectors` request (the campaign's own source, target link and selected weapon of that frame), and a
-- launcher qualifies when its last such request was made in the final warmup frame, its target is an alive routed enemy the host actually stepped in that same frame with a motion vector of squared length above 1 (the host's own
-- `move` answer), and the selected weapon passes the existing unguided, non-decoy, in-stock conditions. The lowest identity order wins (no distance ranking, no fallback to the nearest enemy), and the launch is made with THAT
-- already-linked target and weapon: the fixture then sets no link, so the launch report must say target_link_before=1 (restored=-1). The launch itself does not slew a turret, which is why a link made only for its length
-- (the old nearest-moving selection) supplied no aim and no contact precondition. No such engagement is the exact blocker `UNREACHED: no campaign engagement of a moving routed enemy in the final warmup frame` (the warmup,
-- seed and distances are not tuned; an anti-aircraft gun the host steps with a zero vector does not qualify). The aim `step` request names its source but no weapon, so no heading is recorded: a source may own several turrets
-- and a correlation would be invented.
--   moving_control    the natural case with that selection. It requires the TARGET itself to be asked need_updated (the shared C asks it only of a candidate whose motion vector is above 1: the per-identity proof that the
--                     question was moving), to reach a `test`, and every mobile question to carry an identity.
--   terrain_lift      the same, and after the launch (so the launch's aim is the control's) the host replaces ONE real environmental input of the target's own routed `move` steps: the terrain elevation, by a constant
--                     (LIFT, a source-bounded physical input: only when it stays below the step's own map ceiling). The campaign's waypoint, speed demand and every other input are untouched; the host's physical step
--                     (eech_convoy_step sets the new y from it) answers, the engine writes the answer to Core, the next observation must show Core and the record in step (a difference stops at `entity_pose.mirror`), and the
--                     provider decides the target from that record. It requires a lifted step AND a later question about the target (a cohort that ends first is UNREACHED). Updates in a frame run weapon first, vehicle
--                     after, so the first question that can see a lift is in the NEXT frame. The effect (the verdict, the weapon's terminal and the contact deltas against `moving_control`) is read across runs by Root.
--   Both are allowed under profile=original (they act on movement, not on any entity question): the original solver reads the same Core mirror the engine wrote. What is then comparable is the outcome and the hashes, never a
--   per-candidate question (the original asks none). A coherent Core-and-record change alone does not distinguish which of the two the solver consumed; that is the provider's own record-versus-discarded-mirror test.
--
-- THE CONTROLLED MOVING QUERY CASE (two explicit modes; a QUERY-SERVICE proof, not a natural weapon engagement). The natural moving case above depends on a campaign engagement that the first two actual runs did not find
-- (the case launch slews no turret), so these modes ask the production entity query directly, through the same shared C and the same host route: `engine:wpn_moving_query_prepare` latches three fixed finite segments about ONE
-- real routed vehicle (the collision fixture's structured moving-target recipe: bearings 0, 1 and 2, reach radius + 20 m, the model's bounds, taken from the entity's own position and model: explicit query INPUTS, never an
-- expected answer), one actual engine frame runs, then `engine:wpn_moving_query_run` asks the production get_line_of_sight_collision_entity for the SAME latched segments.
--   moving_query_control   the target is the first alive routed vehicle, in identity order, whose last host step (final warmup frame) had a motion vector above 1 m/s AND was given the terrain input and map ceiling the bounded lift reads (a callback-shape fact copied from the engine's own step: a routed step the core gives no terrain is valid ordinary movement but cannot carry the lift, and such an earlier actor is counted and passed over, never altered); the frame steps normally. The same rule selects in every query mode, hosted and original, and none eligible is an exact prelaunch UNREACHED with finite counts.
--   moving_query_lift      the same, and in that one frame the host's own `move` step of that target carries the existing bounded terrain lift (see terrain_lift): the engine writes the host's answer to Core, the mirror must agree.
--   moving_query_speed_control / moving_query_speed_zero   a SEPARATE pair of modes whose input is the speed demand, not terrain (no terrain, ceiling or lift is read). The target is the first alive routed vehicle in identity
--                          order whose last move step (final warmup frame) had a motion vector above 1 m/s AND carried a finite positive `desired_velocity` (a presence/sign fact copied from the engine's own step, no arithmetic); none eligible is
--                          the same exact prelaunch UNREACHED with finite counts, and the same rule selects in hosted and original. `_control` changes nothing (A). In `_zero` (B), only during the one engine frame between prepare and run and only
--                          on a `move` step of THAT target whose incoming demand is finite and positive, the host delegates a shallow copy that differs ONLY in desired_velocity = 0, exactly once, to the same host.move_vehicle (the engine's
--                          own table is never mutated; no pose, setter, answer or solver is touched; the core's task, route and command storage and every other step field and table are the engine's own). This changes the speed demand the
--                          host's solver RECEIVES in a test-only configuration; it does NOT show that the host failed to fulfil an unchanged solver command, and it is not terrain authority or a whole physical cutover. The row carries the
--                          incoming demand, the substituted 0 and the window's valid, refused, transformed and delegated counts; only a delegated transformed step is credited, a step with a missing or non-positive demand is neither changed nor
--                          credited, and no such step is an exact UNREACHED. B further requires the target OFFERED after that step, the contract held (the existing mirror and fault checks), and a physical result different from the completed
--                          A's (`control_signature`); an identical result is UNCHANGED, never tuned to differ. Query service only: the natural moving weapon integration gap stays open.
-- No retarget, teleport, flag or answer is altered, nothing is searched for a hit, and every segment is asked once. Hosted, the target itself must be asked need_updated, reach a `test` that carries the 16-word moving pose,
-- every mobile question must carry an identity and the conformance assertion must hold; no such target, no such coverage or a refused latch is an exact UNREACHED. Original (profile=original, the module built with weapon-fixture and
-- collision-fixture) asks the SAME segments of the original solver and reports the physical outputs only (no host question exists to count). The rows `query_segment` and `query_authority` carry the physical outputs; comparing
-- control with lift, and hosted with original, is read across runs. They say nothing about a weapon, its aim, launch or cohort, and the natural moving integration gap stays open.
--
-- THE ENTITY_POSE OBSERVATION (fifth slice; mb_hostmodel.h ENTITY_POSE, 15 words). For each MOBILE candidate of a sector list (an aircraft or a vehicle whose collision-test-mobile flag is set) the shipped offer asks the host to
-- OBSERVE it against the host's own record of its semantic host identity (the one Movers registry the movement callbacks step), beside the original's reads. Nothing decides from the answer and no candidate, count or order of
-- the dialogue depends on it. The words (the position and motion vector, and at the placement stage the attitude) are the ones the original read; the observation ADDS three read-only metadata reads, the entity's type, its alive
-- flag and its roster identity, which a transcript of the original's physical getters does not include (it is not a proof of all getters). A candidate that is not mobile, a city and every building listed under it (even one whose
-- flags read as mobile at its placement) are NOT observed. The request is scene 0, depth the stage (0 the offer, 1 the placement), kind the entity's TYPE, definition whether it is alive (0 or 1), and the text place of exactly 15
-- comma-separated 0x%08x patterns, then (when the roster has an identity of 1 to 128 bytes, the roster's whole domain) one space and the HOST IDENTITY. A real reply is { result, kind, difference }: result is recorded,
-- unrecorded, retired, unknown or no_id (no_id exactly when the question named no identity), kind the record's kind (0 none; 1 routed vehicle, 2 ship, 3 person, 4 fixed-wing, 5 helicopter), difference the bits 0..31 of how the
-- campaign's words compare with the record: 1 position, 2 attitude (the placement stage only: an offer-stage reply that carries it is malformed), 4 motion against a held vector (contradicts 8: a reply with both is
-- malformed), 8 no motion vector held, 16 motion not bit for bit the product, rounded toward zero and computed in integers, of the record's attitude row 2 and its speed (also set when an operand is not finite). A class that is not
-- `recorded` carries kind 0 and difference 0.
--
-- WHAT ONE HEALTHY RUN CLASSIFIES TOGETHER. Every observation is classified, BEFORE the launch (boot, warmup and every natural weapon until then: `pose_pre_*`) and after it (`pose_*`), into the histograms of (stage, entity
-- type, alive, class, kind, difference), whose per-stage buckets are the truthful counts of observation QUESTIONS. The `pose_fact_*` fields sum both phases into the facts the host-sourced pose cutover needs, with these DENOMINATORS: an
-- OFFER is one stage-0 observation (the scan offered a mobile candidate to the host once: the same entity offered again by a later scan counts again, so offers are neither unique mobiles nor unique records); every offer-level fact (the
-- classes, dead, the position, the held motion vector, the formula, the kinds) counts STAGE-0 observations only; the attitude facts (`attitude_compared`, `attitude_differs`) count STAGE-1 observations only (a mobile whose broad phase passed
-- is observed a second time, with its attitude); `pose_fact_observations` counts every observation question of both stages. The facts say: how many mobile offers had no identity, had an identity the host never issued, retired or issued
-- without a record, and had a record; how many offers were of a dead entity (a wreck is alive 0); how many records disagreed with the campaign's position and, at the placement, its attitude; how many held a motion vector, how
-- many held none and how many held one that differed; and how many motion vectors were, and how many were not, the record's attitude row 2 times its speed (by kind). It is OBSERVED, never asserted: the natural control only requires
-- that something was observed (a positive count, before or after the launch) and that every question and reply is well formed. WHAT IT DOES NOT SHOW: only the offers and paths this run reached are characterised. A histogram
-- with no mirror difference says nothing about a writer, a pack route, a gunship route or a dependent carry the run never reached, and a difference says THAT a mirror differed, never WHICH writer made it; the disposition of every
-- unreached writer stays required. The counts are global to the war (no weapon identity), and a count of mobile offers is not a measure of moving candidates (`candidate_need_updated` is that counter).
--
--   withheld_pose         the provider raises on the first armed observation:                                    [model-provider:entity_pose.callback]
--   bad_pose_answer       the first armed observation is answered kind 6 (outside the five kinds):              [model-provider:entity_pose.answer]
--   bad_pose_state        the first armed observation that NAMED an identity is answered no_id:                  [model-provider:entity_pose.state]
--
-- THE CONTROLLED VIEW INPUT. See tree-authority.lua: the existing protocol's VIEW_WEAPON lookup answer (mb_hostaim.h, answered r0 by wn_move.c, f[7] in Lua) is set to 1 by an aim delegate over the
-- unmodified host.aim on every armed weapon_step request, hosted and Original alike. It is a CONFIGURED, GLOBAL-while-armed diagnostic input (the protocol carries no weapon identity, so natural
-- weapons are affected too), not a camera, not a per-weapon identity and not an entity call; the entity lookup, the questions and the counters stay measured. A run in which it never reached a
-- weapon step, or the step never took the high-precision path, is UNREACHED.
--
-- WHAT COUNTS AS THE ENGINE STOPPING: only the first line of the failure, anchored: `runtime error: eech: EECH fatal error: host model: [model-provider:<marker>] `. A Lua error that merely
-- mentions a marker, a witness failure, a failure without that wrapper and a different marker are failures of the witness, never proof. After the stop NOTHING is asked of the engine again (no
-- census, report, query or restoration), counters that were not read stay unknown (-1) and no result of the campaign continuing is claimed.
--
-- WHAT COUNTS AS A COMPLETED CASE and the evidence file: as tree-authority.lua (the launch report joined with the logged births, a frozen cohort, one complete terminal per member before the
-- settle cap, terminals of weapons outside the cohort declared foreign; the provider's questions and the fixture's counters are global to the war and are NOT attributed to the case weapons).
-- The evidence file is created exclusively with `host.create_exclusive` (a checked close). Exit 0 is the verdict.
--
-- host.args: root, mode (default control), profile (default hosted), warmup (simulated seconds before the launch, default 30), settle (cap on the simulated seconds after it, default 120),
--            frame_ms (default 100), out (JSON Lines path, must not exist)

io.stdout:setvbuf ("no")

local NAME = "entity-authority"
local function refuse (message) error (NAME .. ": " .. message, 0) end

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

-- BEGIN words
-- ---------------------------------------------------------------------------------------------------------------- float patterns
-- NOTHING HERE DIVIDES. The native host's floating point policy after boot traps a divide by zero in hardware (the sign test `1 / x < 0` of a zero ended the rotor witness's public controls with
-- 0xC000008E, and `pcall` cannot catch a trap), so the sign of a zero is read from its printed form and the two halves of a word are taken by multiplying with an exact power of two.
-- This block is the SAME text as the one in tree-authority.lua (tests/entity_authority.rs compares the two), which runs it with the trap enabled in a child process.
local INVERSE_65536 = 0.0000152587890625 -- 2^-16, exact
local SUBNORMAL_SCALE = 2 ^ 149 -- exact
local function hex8 (bits)
	local high = math.floor (bits * INVERSE_65536)
	return string.format ("%04x%04x", high, bits - high * 65536)
end
-- the IEEE single word of an exact single (eight hex digits), or nil for a number that is not one
local function word_of (x)
	if type (x) ~= "number" or x ~= x then return nil end
	local negative = x < 0 or (x == 0 and string.sub (string.format ("%g", x), 1, 1) == "-")
	if negative then x = -x end
	local bits
	if x == huge then
		bits = 0x7f800000
	elseif x == 0 then
		bits = 0
	else
		local m, e = math.frexp (x)
		local exponent = e + 126
		local mantissa
		if exponent >= 1 then
			if exponent >= 255 then return nil end
			mantissa = (m * 2 - 1) * 8388608
		else
			exponent = 0
			mantissa = x * SUBNORMAL_SCALE
		end
		if mantissa ~= math.floor (mantissa) or mantissa < 0 or mantissa >= 8388608 then return nil end
		bits = exponent * 8388608 + mantissa
	end
	if negative then bits = bits + 2147483648 end
	return hex8 (bits)
end
-- END words

-- ---------------------------------------------------------------------------------------------------------------- the controlled moving QUERY report: one bounded codec and strict grammar
-- (csrc/eech_mq_report.h formats it). A semantic host identity is 1 to 128 bytes of ANY value, so a report carries it ENCODED: every byte but an ASCII letter, digit or one of `._~` is `%XX` (upper-case
-- hexadecimal); the encoding is reversible and canonical, and `-` is always encoded, so a lone `-` is the absent identity. A report is complete lines, single-spaced, whose record names and ordered keys are
-- exactly those below: an unknown, duplicate, missing, reordered or trailing record or key, a malformed word, an empty line or a conditional field of the other branch is a failure of the report, never
-- parsed around. The words are kept as the exact printed bit patterns: nothing is normalised and no numerical answer is computed here.
local MQ_SEGMENTS, MQ_ID_MAX, MQ_ID_ENCODED_MAX = 3, 128, 384
local MQ_TARGET_TYPE = "ENTITY_TYPE_ROUTED_VEHICLE"
-- the three declared bearings of the structured recipe, as the single-precision words the prepare report prints
local MQ_BEARINGS = { "0x3f060a92", "0x40060a92", "0x406a927f" }
local function mq_encode (id)
	if type (id) ~= "string" or #id < 1 or #id > MQ_ID_MAX or string.find (id, "\0", 1, true) then return nil end
	return (string.gsub (id, "[^A-Za-z0-9._~]", function (c) return string.format ("%%%02X", string.byte (c)) end))
end
-- the identity an encoded text names, or nil unless the text is the canonical encoding of an identity of the roster's domain
local function mq_decode (text)
	if type (text) ~= "string" or #text < 1 or #text > MQ_ID_ENCODED_MAX or string.find (text, "[^A-Za-z0-9._~%%]") then return nil end
	local id = string.gsub (text, "%%(%x%x)", function (hex) return string.char (tonumber (hex, 16)) end)
	if mq_encode (id) ~= text then return nil end
	return id
end
local function mq_word (text) return type (text) == "string" and string.match (text, "^0x" .. string.rep ("[0-9a-f]", 8) .. "$") ~= nil end
local function mq_triple (text)
	if type (text) ~= "string" then return false end
	local count = 0
	for word in string.gmatch (text .. ",", "([^,]*),") do
		if not mq_word (word) then return false end
		count = count + 1
	end
	return count == 3 and string.sub (text, -1) ~= ","
end
-- a printed single-precision word whose exponent field is not all ones (neither an infinity nor a NaN)
local function mq_finite_word (text)
	if not mq_word (text) then return false end
	local high, low = tonumber (string.sub (text, 3, 4), 16), tonumber (string.sub (text, 5, 6), 16)
	return (high % 128) * 2 + math.floor (low / 128) ~= 255
end
local function mq_finite_triple (text)
	if not mq_triple (text) then return false end
	for word in string.gmatch (text, "[^,]+") do
		if not mq_finite_word (word) then return false end
	end
	return true
end
-- complete newline-terminated lines, none empty or holding a control byte
local function mq_lines (text)
	if type (text) ~= "string" or text == "" or string.sub (text, -1) ~= "\n" then return nil, "the report is not complete newline-terminated lines" end
	local list, start = {}, 1
	while start <= #text do
		local stop = string.find (text, "\n", start, true)
		local line = string.sub (text, start, stop - 1)
		if line == "" then return nil, "the report holds an empty line" end
		if string.find (line, "%c") then return nil, "the report holds a control byte" end
		list [#list + 1] = line
		start = stop + 1
	end
	return list
end
-- one record: the exact record name, then exactly these keys in this order, single-spaced, each with a value
local function mq_row (line, record, keys)
	local tokens = {}
	for token in string.gmatch (line, "[^ ]+") do tokens [#tokens + 1] = token end
	if table.concat (tokens, " ") ~= line then return nil, "the record is not single-spaced" end
	if tokens [1] ~= record or #tokens ~= #keys + 1 then return nil, "the record is not " .. record .. " with " .. #keys .. " keys" end
	local row = { raw = line }
	for index, key in ipairs (keys) do
		local name, value = string.match (tokens [index + 1], "^([a-z_]+)=(.+)$")
		if name ~= key then return nil, "the key " .. key .. " is missing, unknown or out of order" end
		row [key] = value
	end
	return row
end
-- exactly one prepare header and three ordered segment rows for `target`; the header and rows are returned
local function mq_parse_prepare (text, target)
	local lines, why = mq_lines (text)
	if not lines then return nil, why end
	if #lines ~= 1 + MQ_SEGMENTS then return nil, "the prepare report has " .. #lines .. " records, not one header and " .. MQ_SEGMENTS .. " segments" end
	local head
	head, why = mq_row (lines [1], "wpn-moving-query-prepare", { "host_id", "segments", "target_type", "radius" })
	if not head then return nil, "prepare header: " .. why end
	if mq_decode (head.host_id) ~= target then return nil, "the prepare header does not name the target " .. target end
	if head.segments ~= tostring (MQ_SEGMENTS) or head.target_type ~= MQ_TARGET_TYPE then return nil, "the prepare header's segment count or target type is not the declared " .. MQ_SEGMENTS .. " / " .. MQ_TARGET_TYPE end
	if not mq_finite_word (head.radius) or string.sub (head.radius, 3, 3) >= "8" or head.radius == "0x00000000" then return nil, "the prepare header's radius is not a finite positive word" end
	local segments = {}
	for index = 1, MQ_SEGMENTS do
		local row
		row, why = mq_row (lines [index + 1], "wpn-moving-query-segment", { "seg", "bearing", "a", "b" })
		if not row then return nil, "prepare segment " .. index .. ": " .. why end
		if row.seg ~= tostring (index - 1) or row.bearing ~= MQ_BEARINGS [index] or not mq_finite_triple (row.a) or not mq_finite_triple (row.b) then
			return nil, "prepare segment " .. index .. " is out of order, has an undeclared bearing or a non-finite endpoint: " .. row.raw
		end
		segments [index] = row
	end
	return head, segments
end
-- exactly one run header for `target` and three ordered result rows over the prepared segments; each result is returned with its decoded hit identity
local function mq_parse_run (text, target, prepared)
	local lines, why = mq_lines (text)
	if not lines then return nil, why end
	if #lines ~= 1 + MQ_SEGMENTS then return nil, "the run report has " .. #lines .. " records, not one header and " .. MQ_SEGMENTS .. " results" end
	local head
	head, why = mq_row (lines [1], "wpn-moving-query-run", { "host_id", "segments" })
	if not head then return nil, "run header: " .. why end
	if mq_decode (head.host_id) ~= target or head.segments ~= tostring (MQ_SEGMENTS) then return nil, "the run header does not name the target " .. target .. " with " .. MQ_SEGMENTS .. " segments" end
	local results = {}
	for index = 1, MQ_SEGMENTS do
		local row
		row, why = mq_row (lines [index + 1], "wpn-moving-query-result", { "seg", "hit", "a", "b", "hit_is_target", "hit_type", "hit_host_id", "point", "normal" })
		if not row then return nil, "run result " .. index .. ": " .. why end
		if row.seg ~= tostring (index - 1) or row.a ~= prepared [index].a or row.b ~= prepared [index].b then
			return nil, "run result " .. index .. " is not the prepared segment (same index and the same endpoint words): " .. row.raw
		end
		if row.hit == "0" then
			if row.hit_is_target ~= "0" or row.hit_type ~= "none" or row.hit_host_id ~= "-" or row.point ~= "-" or row.normal ~= "-" then
				return nil, "run result " .. index .. " is a miss that carries hit fields: " .. row.raw
			end
		elseif row.hit == "1" then
			if row.hit_is_target ~= "0" and row.hit_is_target ~= "1" then return nil, "run result " .. index .. " has no hit_is_target flag: " .. row.raw end
			if not string.match (row.hit_type, "^ENTITY_TYPE_[A-Z_]+$") then return nil, "run result " .. index .. " is a hit with no entity type: " .. row.raw end
			if not mq_triple (row.point) or not mq_triple (row.normal) then return nil, "run result " .. index .. " is a hit without exactly three point words and three normal words: " .. row.raw end
			if row.hit_host_id ~= "-" then
				row.hit_id = mq_decode (row.hit_host_id)
				if row.hit_id == nil then return nil, "run result " .. index .. " carries a hit identity that is not a canonical encoding: " .. row.raw end
			end
			if row.hit_is_target == "1" and (row.hit_id ~= target or row.hit_type ~= MQ_TARGET_TYPE) then
				return nil, "run result " .. index .. " says the hit is the target, but its identity or type is not: " .. row.raw
			end
			-- the reverse join: roster identities are unique and the host computes the flag from the same entity, so the target's own identity under a false flag is contradictory
			if row.hit_is_target == "0" and row.hit_id == target then
				return nil, "run result " .. index .. " says the hit is not the target, but its identity is the target's: " .. row.raw
			end
		else
			return nil, "run result " .. index .. " carries hit=" .. tostring (row.hit) .. ": " .. row.raw
		end
		results [index] = row
	end
	return head, results
end
-- END the controlled moving QUERY report

local args = host.args
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local mode = args.mode or "control"
local profile = args.profile or "hosted"
if profile ~= "hosted" and profile ~= "original" then refuse ("profile must be hosted or original, got '" .. tostring (profile) .. "'") end
local warmup = tonumber (args.warmup or "30")
local settle = tonumber (args.settle or "120")
local frame_ms = tonumber (args.frame_ms or "100")
if not (finite (warmup) and finite (settle) and finite (frame_ms)) then refuse ("warmup, settle and frame_ms must be finite numbers") end
if not (warmup >= 1 and warmup <= 600 and settle >= 1 and settle <= 1200 and frame_ms >= 10 and frame_ms <= 1000) then refuse ("warmup, settle or frame_ms is out of range") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local seed = 1
-- the most frames the whole run can need is far below this: the simulated clock must advance on every frame, and the budget is four times what warmup and settle ask for, plus a margin
local frame_budget = 4 * math.ceil ((warmup + settle) * 1000 / frame_ms) + 200


-- ---------------------------------------------------------------------------------------------------------------- the modes
local ZERO = "0x00000000"
local UP = ZERO .. ",0x3f800000," .. ZERO
local NO_POINT = ZERO .. "," .. ZERO .. "," .. ZERO
-- the first three patterns of the request's text: the segment's OLD end
local function old_end (parsed) return parsed.words [1] .. "," .. parsed.words [2] .. "," .. parsed.words [3] end
local MODES = {
	control = { altering = false },
	forced_hit = {
		-- HIT, mask 63, at the segment's OLD end with an up normal: complete write information for a hit
		answer = function (parsed, real) return { result = "hit", mask = 63, name = real.name, p = old_end (parsed) .. "," .. UP } end,
		altering = true, expect = "hit",
	},
	forced_miss = {
		-- MISS, mask 56: the normal assigned (up), the point not: an accepted partial-write miss
		answer = function (_, real) return { result = "miss", mask = 56, name = real.name, p = NO_POINT .. "," .. UP } end,
		altering = true, expect = "miss",
	},
	withheld_entity = { answer = function () error ("the host withholds its entity answer", 0) end, marker = "entity_collision.callback" },
	bad_write_mask = {
		answer = function (_, real) return { result = "hit", mask = 56, name = real.name, p = NO_POINT .. "," .. UP } end,
		marker = "entity_collision.write_mask",
	},
	bad_answer = {
		answer = function (parsed, real) return { result = "hit", mask = 64, name = real.name, p = old_end (parsed) .. "," .. UP } end,
		marker = "entity_collision.answer",
	},
}
-- the ENTITY_CANDIDATE modes (see the header): the configured secondary input is the existing forced-hit entity_collision answer
MODES.candidate_real = { answer = MODES.forced_hit.answer, paired = true }
MODES.candidate_skip = {
	answer = MODES.forced_hit.answer, paired = true, skips = true,
	-- a valid skip: nothing more to do for the candidate; the name is the real answer's echo
	candidate = function (_, real) return { result = "skip", name = real.name } end,
}
MODES.withheld_candidate = { candidate = function () error ("the host withholds its candidate answer", 0) end, once = true, marker = "entity_candidate.callback" }
MODES.bad_candidate_state = {
	-- `buildings` is legal only for a city (fact 9); the first question that is not a city is answered with it
	candidate = function (_, real) return { result = "buildings", name = real.name } end,
	eligible = function (parsed) return parsed.words [9] == ZERO end, once = true, marker = "entity_candidate.state",
}
MODES.bad_candidate_identity = {
	-- the real state, under a name that is not the scene's (and not empty, so it is wrong for scene 0 too)
	candidate = function (_, real) return { result = real.result, name = "NOT_THE_SCENE", p = real.p } end,
	once = true, marker = "entity_candidate.identity",
}
-- the ENTITY_SCAN modes (see the header): the configured secondary input is the existing forced-hit entity_collision answer
MODES.scan_real = { answer = MODES.forced_hit.answer, paired = true }
MODES.scan_none = {
	answer = MODES.forced_hit.answer, paired = true, no_downstream = true,
	-- a valid empty enumeration, only at the start of a scan
	scan = function () return { result = "done_none" } end,
	eligible_scan = function (parsed) return parsed.phase == 0 end,
}
MODES.withheld_scan = { scan = function () error ("the host withholds its scan answer", 0) end, once = true, marker = "entity_scan.callback" }
MODES.bad_scan_state = {
	-- `next` is illegal at the start of a scan: no entry has been served
	scan = function () return { result = "next" } end,
	eligible_scan = function (parsed) return parsed.phase == 0 end, once = true, marker = "entity_scan.state",
}
MODES.bad_scan_cursor = {
	-- the real first sector with a fractional x: finite and a well-formed pattern, so it reaches the campaign's own cursor check
	scan = function (_, real) return { result = "first", p = "0x3f000000," .. string.match (real.p, "^[^,]+,([^,]+)$") } end,
	eligible_scan = function (_, real) return real.result == "first" end, once = true, marker = "entity_scan.cursor",
}
-- the ENTITY_POSE modes (see the header): the observation is never altered in any other mode, and these refuse by marker
MODES.withheld_pose = { pose = function () error ("the host withholds its pose observation", 0) end, once = true, marker = "entity_pose.callback" }
MODES.bad_pose_answer = {
	-- a record kind outside the five
	pose = function () return { result = "recorded", kind = 6, difference = 0 } end, once = true, marker = "entity_pose.answer",
}
MODES.bad_pose_state = {
	-- "no identity" for a question that named one
	pose = function () return { result = "no_id", kind = 0, difference = 0 } end,
	eligible_pose = function (parsed) return parsed.named end, once = true, marker = "entity_pose.state",
}
-- the HOST-SIDE record modes (see the header): the host's own movement callback changes its own record before the question is delegated
MODES.withdrawn_record = { withdraw = true, marker = "entity_candidate.refused" }
MODES.displaced_record = { displace = true, marker = "entity_pose.mirror" }
-- the host withdraws the BACKING of the entry the real scan has just served: its own REMOVE lifecycle event (phase 5) on that key, answered `removed` by the same provider; the next real ENTRY stands at the ended key
MODES.withdrawn_backing = { withdraw_backing = true, marker = "entity_scan.refused" }
-- the same for a city's buildings: the host's own REMOVE (phase 8) of the building the real traversal has just served; the next question of the traversal stands at the ended building
MODES.withdrawn_child = { withdraw_child = true, marker = "entity_scan.refused" }
-- the HOST-SIDE movement modes (completion modes): see the header
MODES.moving_control = { moving_target = true, host_side_movement = true }
MODES.terrain_lift = { moving_target = true, host_side_movement = true, lift = true }
-- the controlled moving QUERY case (completion modes; no weapon is launched): see the header
MODES.moving_query_control = { query = true, host_side_movement = true }
MODES.moving_query_lift = { query = true, host_side_movement = true, lift = true }
-- the controlled moving QUERY case with a configured SPEED-DEMAND input (see the header): the target is chosen by the speed demand its final warmup move step carried, not by terrain
MODES.moving_query_speed_control = { query = true, host_side_movement = true, speed = true }
MODES.moving_query_speed_zero = { query = true, host_side_movement = true, speed = true, speed_zero = true }
local plan = MODES [mode]
if plan == nil then refuse ("unknown mode '" .. tostring (mode) .. "'") end
if profile == "original" and mode ~= "control" and not plan.host_side_movement then refuse ("profile=original runs only mode=control (or a host-side movement mode): no entity_collision question reaches the provider to alter") end

-- ---------------------------------------------------------------------------------------------------------------- the evidence file: created exclusively, every write, flush and close checked, closed on every path
local create = host.create_exclusive
if type (create) ~= "function" then refuse ("this host has no host.create_exclusive: the evidence cannot be created exclusively with a checked close, and the witness does not fall back to a truncating open") end
local out, create_why = create (out_path)
if not out then refuse ("OUTPUT FAILURE: creating " .. out_path .. " failed: " .. tostring (create_why) .. " (the evidence is created exclusively and never overwritten; nothing was run)") end

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then value = json_string (v)
		elseif type (v) == "boolean" then value = v and "true" or "false"
		else value = tostring (v) end
		parts [#parts + 1] = json_string (k) .. ":" .. value
	end
	table.sort (parts)
	local ok, err = out:write ("{" .. table.concat (parts, ",") .. "}\n")
	if not ok then error (NAME .. ": OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err), 0) end
	ok, err = out:flush ()
	if not ok then error (NAME .. ": OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err), 0) end
end
-- a failure of the witness itself: recorded if the evidence can still be written, then raised with the prefix the engine-failure handlers never treat as the engine's
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	error (NAME .. ": " .. message, 0)
end
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end

-- ---------------------------------------------------------------------------------------------------------------- the grammar of the existing C emitters (eech_wpn_fixture.c)
-- a whole number as the C side prints it (%d / %llu): optional minus, digits, nothing else (no fraction, exponent, nan or inf)
local function whole (text)
	if type (text) ~= "string" or not string.match (text, "^%-?%d+$") then return nil end
	return tonumber (text)
end
-- a finite number as %.9g prints it: digits with an optional fraction and an optional signed exponent
local function printed_number (text)
	if type (text) ~= "string" then return nil end
	if not (string.match (text, "^%-?%d+%.?%d*$") or string.match (text, "^%-?%d+%.?%d*e[%+%-]%d+$")) then return nil end
	local value = tonumber (text)
	if value == nil or value ~= value or value == huge or value == -huge then return nil end
	return value
end
-- the first problem of a wpn-case-launch report against the emitter's schema and domains, or nil: launcher is a name, weapon / target links / counts are whole numbers (a link is 0 or 1), creates
-- is a positive whole number, restored is -1 (the launcher's link was already the target: nothing changed, so the link before and after are the same) or 1 (restored); 0 is a failed restoration
-- and nothing else is in the schema
local function launch_report_problem (report, line, weapon)
	local function bad (name) return "the launch report does not follow the wpn-case-launch schema (" .. name .. "): " .. line end
	if type (report.launcher) ~= "string" or report.launcher == "" then return bad ("launcher") end
	local number = {}
	for _, name in ipairs { "weapon", "target_link_before", "target_link", "count_before", "count_after", "births", "creates", "restored" } do
		number [name] = whole (report [name])
		if number [name] == nil then
			return report [name] == nil and bad ("missing " .. name) or ("the launch report field " .. name .. "=" .. tostring (report [name]) .. " is not a whole number: " .. line)
		end
	end
	if number.weapon ~= weapon then return "the launch report says weapon=" .. number.weapon .. " but weapon " .. weapon .. " was launched: " .. line end
	for _, name in ipairs { "target_link_before", "target_link" } do
		if number [name] ~= 0 and number [name] ~= 1 then return "the launch report field " .. name .. "=" .. number [name] .. " is not 0 or 1: " .. line end
	end
	if number.count_before < 0 or number.count_after < 0 or number.births < 0 then return "the launch report has a negative count: " .. line end
	if number.count_after > number.count_before then return "the launch report has count_after=" .. number.count_after .. " above count_before=" .. number.count_before .. ": " .. line end
	if number.creates < 1 then return "the launch created no weapon (creates=" .. number.creates .. "): " .. line end
	if number.restored == 0 then return "the launch could not restore the launcher's temporary target link (restored=0): " .. line end
	if number.restored ~= -1 and number.restored ~= 1 then return "the launch report field restored=" .. number.restored .. " is not -1 or 1 (restored=0 is a failure, nothing else is in the schema): " .. line end
	if number.restored == -1 and number.target_link_before ~= number.target_link then return "restored=-1 means the link was kept, but the link changed from " .. number.target_link_before .. " to " .. number.target_link .. ": " .. line end
	return nil
end
-- the first problem of a weapon's terminal fact, or nil: every field the emitter writes for a registered weapon is present and in its grammar
local function terminal_problem (e)
	if whole (e.w) == nil or whole (e.w) < 0 then return "w" end
	if type (e.type) ~= "string" or e.type == "" then return "type" end
	if type (e.code) ~= "string" or e.code == "" or e.code == "invalid" then return "code" end
	local steps = whole (e.steps)
	if steps == nil or steps < 0 then return "steps" end
	local x, y, z = string.match (type (e.pos) == "string" and e.pos or "", "^([^,]+),([^,]+),([^,]+)$")
	if printed_number (x) == nil or printed_number (y) == nil or printed_number (z) == nil then return "pos" end
	if printed_number (e.velocity) == nil then return "velocity" end
	if printed_number (e.lifetime) == nil then return "lifetime" end
	return nil
end


-- ---------------------------------------------------------------------------------------------------------------- the engine's own stop, recognised only by its anchored first line
local FATAL = "^runtime error: eech: EECH fatal error: host model: %[model%-provider:([%w_%.]+)%] "
local function engine_marker (err) return string.match (first_line (err), FATAL) end

-- ---------------------------------------------------------------------------------------------------------------- the host's model provider, counted, altered only when armed
local armed = false
local reference -- the host's own model provider (host.model), resolved inside the owned close scope (main)
local before, after = { entity = 0, other = 0, candidate = 0, scan = 0, roster = 0, city = 0, pose = 0 }, { entity = 0, other = 0, candidate = 0, scan = 0, roster = 0, city = 0, pose = 0 }
local altered = 0
local cand_altered, cand_skipped = 0, 0
local scan_altered, scan_none_answered = 0, 0
local pose_altered = 0
local withdrawn, displaced = 0, 0
-- the frames the engine has run (the host-movement wrapper stamps each of its answers with the frame it was given in), the host's last `move` answer per identity, the case target's identity and its questions
local frames = 0
local last_move = {}
-- the last `vectors` aim request seen per source identity (target link, selected weapon, the target's motion words, frame): an observation of the campaign's own engagement, forwarded unchanged
local last_vectors = {}
local watched
-- (window_*: the target's own steps seen in the controlled query's one frame, by the shape the bounded lift needs; type presence only, no value is kept; counted in both query modes)
-- (speed_*: the speed-demand modes' counts for the same window: move steps that carried a finite positive desired_velocity, those that did not, the transformed copies made and delegated, the first incoming demand and the
-- substituted one as text, and the target's questions after a transformed step was delegated)
local tgt = { questions = 0, skip = 0, need_updated = 0, need_fixed = 0, need_placement = 0, need_normal = 0, buildings = 0, test = 0, test_moving16 = 0, after_lift = 0, after_lift_test = 0,
	window_open = false, window_steps = 0, window_moves = 0, window_with_terrain = 0,
	speed_valid = 0, speed_bad = 0, speed_transforms = 0, speed_delegations = 0, speed_incoming = "none", speed_substituted = "none", after_speed = 0, after_speed_test = 0 }
-- true only while the controlled query case runs its one frame: the lift of the target's own step is allowed then although the weapon launch never arms the witness
local lift_window = false
-- the lifted terrain elevation (metres added to the target's own routed step; a constant declared before any run) and what the lifted steps did
local LIFT = 500
local lifted, lift_refused, lift_row = 0, 0, { baseline = 0, elevation = 0, answered_y = 0, frame = -1 }
-- the armed pose observations by stage and by the class the REAL host answered, the ones that named an identity, and the histogram of (stage, entity type, alive, class, kind, difference)
local pose = { total = 0, stage_offer = 0, stage_placement = 0, named = 0, recorded = 0, unrecorded = 0, retired = 0, unknown = 0, no_id = 0 }
local pose_histogram = {}
-- every observation BEFORE the launch (boot, warmup, the natural weapons until then): classified exactly as the armed ones, never altered
local pose_pre = { total = 0, stage_offer = 0, stage_placement = 0, named = 0, recorded = 0, unrecorded = 0, retired = 0, unknown = 0, no_id = 0 }
local pose_pre_histogram = {}
-- the armed scan questions by phase, and the REAL answers they got
local scan = { total = 0, phase_begin = 0, phase_entry = 0, phase_end = 0, phase_resolved = 0, excluded = 0, first = 0, next = 0, offer = 0, done_none = 0, done_hit = 0 }
-- the armed candidate questions and the REAL answers they got
local cand = { total = 0, scene0 = 0, mobile = 0, mobile_named = 0, mobile_unnamed = 0, skip = 0, need_updated = 0, need_fixed = 0, need_placement = 0, need_normal = 0, buildings = 0, test = 0 }
-- what the provider SEES after the launch (the whole war's questions: the provider carries no weapon identity)
local seen = { stationary = 0, moving = 0, real_hit = 0, real_miss = 0, real_mask_63 = 0, real_mask_56 = 0, real_mask_other = 0, answered_hit = 0, answered_miss = 0 }
local faults = { request = 0, reply = 0 }
local first_fault -- the first malformed request or real reply, kept for the evidence

local PATTERN = "^0x%x%x%x%x%x%x%x%x$"
local function words_of_text (text, count)
	local words = {}
	for token in string.gmatch (text, "[^,]+") do
		if not string.match (token, PATTERN) then return nil end
		words [#words + 1] = token
	end
	if (count and #words ~= count) or table.concat (words, ",") ~= text then return nil end
	return words
end
-- the request as the corrected contract states it: scene >= 1 and nothing but the text place, which holds 18 (stationary) or 22 (moving) float patterns
local function parse_request (scene, depth, kind, definition, key)
	if type (scene) ~= "number" or scene < 1 then return nil, "scene " .. tostring (scene) end
	if depth ~= nil or kind ~= nil or definition ~= nil then return nil, "an entity question carries only a scene and the pose text" end
	if type (key) ~= "string" then return nil, "no pose text" end
	local words = words_of_text (key)
	if words == nil then return nil, "the pose text is not comma separated 0x%08x patterns" end
	if #words ~= 18 and #words ~= 22 then return nil, "the pose has " .. #words .. " floats, not 18 or 22" end
	return { scene = scene, words = words, moving = #words == 22 }
end
-- the first problem of a REAL reply against the contract, or nil: result, mask domain, whole vectors, a point only with a normal, a hit is 63, zero words for what the mask does not assign, a name
local function reply_problem (answer)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if answer.result ~= "hit" and answer.result ~= "miss" then return "result " .. tostring (answer.result) end
	local mask = answer.mask
	if not finite (mask) or mask ~= math.floor (mask) or mask < 0 or mask > 63 then return "mask " .. tostring (mask) end
	if type (answer.name) ~= "string" or answer.name == "" then return "no scene name" end
	if type (answer.p) ~= "string" then return "no words" end
	local words = words_of_text (answer.p, 6)
	if words == nil then return "the words are not six 0x%08x patterns" end
	local point, normal = mask % 8, math.floor (mask / 8)
	if (point ~= 0 and point ~= 7) or (normal ~= 0 and normal ~= 7) then return "mask " .. mask .. " assigns a vector in part" end
	if point ~= 0 and normal == 0 then return "mask " .. mask .. " assigns the point without the normal" end
	if answer.result == "hit" and mask ~= 63 then return "a hit with mask " .. mask end
	for index = 1, 6 do
		local assigned = (index <= 3 and point ~= 0) or (index > 3 and normal ~= 0)
		if not assigned and words [index] ~= ZERO then return "word " .. index .. " is not assigned by mask " .. mask .. " but is not zero" end
	end
	return nil
end
-- a text place of `count` fixed-width patterns and, when the campaign has an identity, one space and the identity (1 to 128 bytes); nil for a text that is neither
local function split_identity (key, count)
	local words_text = count * 10 + (count - 1)
	if #key > words_text then
		if string.sub (key, words_text + 1, words_text + 1) ~= " " then return nil end
		return string.sub (key, 1, words_text), string.sub (key, words_text + 2)
	end
	return key, ""
end
-- the candidate request as the contract states it: a scene >= 0 (0 until a shape was read) and nothing but the text place, which holds exactly 44 float patterns; fact 7 is the mobile flag
local function parse_candidate_request (scene, depth, kind, definition, key)
	if type (scene) ~= "number" or scene < 0 or scene ~= math.floor (scene) then return nil, "scene " .. tostring (scene) end
	if depth ~= nil or kind ~= nil or definition ~= nil then return nil, "an entity candidate question carries only a scene and the facts text" end
	if type (key) ~= "string" then return nil, "no facts text" end
	local text, host_id = split_identity (key, 44)
	local words = text and words_of_text (text, 44)
	if words ~= nil and #host_id > 128 then return nil, "the identity is " .. #host_id .. " bytes, outside the roster's domain of 1 to 128" end
	if words == nil then return nil, "the facts text is not 44 comma separated 0x%08x patterns" end
	return { scene = scene, words = words, mobile = words [7] ~= ZERO, host_id = host_id, named = host_id ~= "" }
end
local CANDIDATE_STATES = { skip = true, need_updated = true, need_fixed = true, need_placement = true, need_normal = true, buildings = true, test = true }
-- the first problem of a REAL candidate reply against the contract, or nil: one of the seven states, a name (EMPTY for scene 0, the scene's compiled name otherwise) and a pose of 12 or 16 patterns for a test only
local function candidate_reply_problem (answer, scene)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if not CANDIDATE_STATES [answer.result] then return "result " .. tostring (answer.result) end
	if type (answer.name) ~= "string" then return "no scene name" end
	if scene == 0 and answer.name ~= "" then return "a name for scene 0" end
	if scene ~= 0 and answer.name == "" then return "no scene name for scene " .. tostring (scene) end
	if answer.result == "test" then
		if type (answer.p) ~= "string" then return "a test without a pose" end
		local words = words_of_text (answer.p)
		if words == nil or (#words ~= 12 and #words ~= 16) then return "the pose is not 12 or 16 0x%08x patterns" end
	elseif answer.p ~= nil then
		return "a pose with " .. answer.result
	end
	return nil
end
-- the scan request as the contract states it (mb_hostentity.h): scene 0, nothing else but the text place, which holds exactly 16 patterns and, when the question carries one, one space and the identity place
-- (1 to 128 bytes). The phase (fact 12) is one of the scan's own, 0 BEGIN, 1.0 ENTRY, 2.0 END, 3.0 RESOLVED, or one of the ROSTER's events, 4.0 INSERT, 5.0 REMOVE, 6.0 RESET. The identity place holds the key of the
-- entry the scan stands at in ENTRY and RESOLVED (a roster key: 1 to 20 decimal digits, no leading zero) and in no other scan phase; an INSERT's optional hosted-mobile identity; a REMOVE's key; nothing for a RESET.
-- The facts an event does not state are zero.
local PHASES = { [ZERO] = 0, ["0x3f800000"] = 1, ["0x40000000"] = 2, ["0x40400000"] = 3, ["0x40800000"] = 4, ["0x40a00000"] = 5, ["0x40c00000"] = 6,
	["0x40e00000"] = 7, ["0x41000000"] = 8, ["0x41100000"] = 9, ["0x41200000"] = 10, ["0x41300000"] = 11, ["0x41400000"] = 12 }
local ROSTER_EVENTS = { [4] = "insert", [5] = "remove", [6] = "reset" }
-- the facts an INSERT states (the position's x, y and z, the entity type, the side length: facts 1 to 3, 4 and 7) and the phase (12); all others are zero
local INSERT_STATED = { [1] = true, [2] = true, [3] = true, [4] = true, [7] = true, [12] = true }
local function is_roster_key (text) return type (text) == "string" and #text <= 20 and string.match (text, "^[1-9]%d*$") ~= nil end
local function shown (text) return string.sub (tostring (text), 1, 40) end
local function others_zero (words, stated)
	for index = 1, 16 do
		if not stated [index] and words [index] ~= ZERO then return false end
	end
	return true
end
local CITY_EVENTS = { [7] = "child_insert", [8] = "child_remove", [9] = "child_first", [10] = "child_next", [11] = "link", [12] = "name" }
local CITY_RESULTS = { child_insert = "placed", child_remove = "removed", child_first = "next", child_next = "next", link = "placed", name = "placed" }
-- the keys (and, for a NAME, the object identity after the first space) of a city event's identity place, or nil and the problem: one key for the first building of a city, one or two for an insertion (the city, and
-- the predecessor), two for a removal, a next building and a LINK, a key and an object identity for a NAME
local function city_identity (event, identity)
	local space = string.find (identity, " ", 1, true)
	local first, second = identity, nil
	if space then first, second = string.sub (identity, 1, space - 1), string.sub (identity, space + 1) end
	if not is_roster_key (first) then return nil, "the identity place of a city event begins with \"" .. shown (first) .. "\", which is not a roster key" end
	if event == "name" then
		if second == nil or second == "" then return nil, "a NAME carries an empty object identity" end
		if string.find (second, "[%c\127]") then return nil, "a NAME carries an object identity with a control character" end
	elseif event == "child_first" then
		if second ~= nil then return nil, "a request for the first building of a city names the city only" end
	elseif event == "child_insert" then
		if second ~= nil and not is_roster_key (second) then return nil, "the predecessor \"" .. shown (second) .. "\" is not a roster key" end
	else
		if second == nil then return nil, "the identity place of a " .. event .. " names one key where it needs two" end
		if not is_roster_key (second) then return nil, "the second key \"" .. shown (second) .. "\" is not a roster key" end
	end
	return first, second
end
local function parse_scan_request (scene, depth, kind, definition, key)
	if scene ~= 0 then return nil, "scene " .. tostring (scene) end
	if depth ~= nil or kind ~= nil or definition ~= nil then return nil, "an entity scan question carries only the scene 0 and the facts text" end
	if type (key) ~= "string" then return nil, "no facts text" end
	local text, identity = split_identity (key, 16)
	if text == nil then return nil, "the identity does not follow the 16 patterns after one space" end
	if #key > #text and identity == "" then return nil, "an empty identity follows the 16 patterns" end
	if #identity > 128 then return nil, "the identity is " .. #identity .. " bytes, outside the roster's domain of 1 to 128" end
	local words = words_of_text (text, 16)
	if words == nil then return nil, "the facts text is not 16 comma separated 0x%08x patterns" end
	local phase = PHASES [words [12]]
	if phase == nil then return nil, "the phase " .. words [12] .. " is not 0 to 12" end
	if phase <= 3 then
		local stands = phase == 1 or phase == 3
		if stands and identity == "" then return nil, "phase " .. phase .. " stands at a served entry and names no roster key" end
		if stands and not is_roster_key (identity) then return nil, "the key of the entry phase " .. phase .. " stands at, \"" .. shown (identity) .. "\", is not a roster key" end
		if not stands and identity ~= "" then return nil, "phase " .. phase .. " carries no standing key" end
		return { kind = "scan", words = words, phase = phase, excluded = words [15] ~= ZERO, hit = words [16] ~= ZERO, standing = stands and identity or nil }
	end
	if phase >= 7 then
		-- a CITY event: nothing is stated but the phase (and an insertion's entity type, fact 4); the identity place holds the keys of the phase's grammar
		local event = CITY_EVENTS [phase]
		if not others_zero (words, phase == 7 and { [4] = true, [12] = true } or { [12] = true }) then return nil, "a city event states nothing but its phase" .. (phase == 7 and " and its entity type" or "") end
		local first, second = city_identity (event, identity)
		if first == nil then return nil, second end
		return { kind = "city", words = words, phase = phase, event = event, keys = { first, second }, object = event == "name" and second or nil }
	end
	local event = ROSTER_EVENTS [phase]
	if event == "insert" then
		if not others_zero (words, INSERT_STATED) then return nil, "an insertion states only the position, the type and the side length" end
	elseif not others_zero (words, { [12] = true }) then
		return nil, "a " .. (event == "remove" and "removal" or "boot") .. " states nothing but its phase"
	elseif event == "remove" and not is_roster_key (identity) then
		return nil, "a removal names \"" .. shown (identity) .. "\", which is not a roster key"
	elseif event == "reset" and identity ~= "" then
		return nil, "a boot carries no identity"
	end
	return { kind = "roster", words = words, phase = phase, event = event, identity = identity }
end
-- the keys the host issued and ended, as the questions and answers of the roster events show them (from the first event this wrapper sees, before or after the launch), and the entry the scan was last served
local roster_live, roster_ended, last_served = {}, {}, nil
local SCAN_STATES = { first = true, next = true, offer = true, done_none = true, done_hit = true }
-- the states the kernel allows after each scan phase: the start and the end of a list serve a sector or end the scan, an entry is offered or skipped, a resolved offer ends the scan or goes on
local PHASE_STATES = { [0] = { first = true, done_none = true }, [1] = { offer = true, next = true }, [2] = { first = true, done_none = true }, [3] = { next = true, done_hit = true } }
local ROSTER_RESULTS = { insert = "placed", remove = "removed", reset = "cleared" }
-- the first problem of a REAL scan reply against the contract, or nil: one of the five states, two patterns for `first` only, a name (the key of the entry served) for `first` and `next` only and never one that
-- ended, and, for a well-formed question, the state its phase and facts allow
local function scan_reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if not SCAN_STATES [answer.result] then return "result " .. tostring (answer.result) end
	local name = answer.name
	if name ~= nil and type (name) ~= "string" then return "the name is not text" end
	if name ~= nil and name ~= "" then
		if answer.result ~= "first" and answer.result ~= "next" then return "a name with " .. answer.result end
		if not is_roster_key (name) then return "the name \"" .. shown (name) .. "\" is not a roster key" end
		if roster_ended [name] then return "the entry " .. name .. " it serves has ended" end
	end
	if answer.result == "first" then
		if type (answer.p) ~= "string" or words_of_text (answer.p, 2) == nil then return "first without two 0x%08x patterns" end
	elseif answer.p ~= nil then
		return "a cursor with " .. answer.result
	end
	if parsed then
		local allowed = "the state " .. answer.result .. " is not what the kernel decides after phase " .. parsed.phase
		if not PHASE_STATES [parsed.phase][answer.result] then return allowed end
		if parsed.phase == 1 and (answer.result == "next") ~= parsed.excluded then return allowed .. " for an entry that is " .. (parsed.excluded and "excluded" or "not excluded") end
		if parsed.phase == 3 and (answer.result == "done_hit") ~= parsed.hit then return allowed .. " for an offer that " .. (parsed.hit and "hit" or "missed") end
	end
	return nil
end
-- the first problem of a REAL roster reply against the contract, or nil: the state its phase allows, a key only for an insertion (a roster key the host never issued before) and no cursor
local function roster_reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	local wanted = ROSTER_RESULTS [parsed.event]
	if answer.result ~= wanted then return "a roster event of phase " .. parsed.phase .. " (" .. parsed.event .. ") answered " .. tostring (answer.result) .. ", not " .. wanted end
	if answer.p ~= nil then return "a cursor with " .. wanted end
	local name = answer.name
	if name ~= nil and type (name) ~= "string" then return "the name is not text" end
	if parsed.event == "insert" then
		if name == nil or name == "" then return "an insertion names no key" end
		if not is_roster_key (name) then return "the key \"" .. shown (name) .. "\" is not a roster key" end
		if roster_live [name] or roster_ended [name] then return "the key " .. name .. " was issued before" end
	elseif name ~= nil and name ~= "" then
		return "a " .. parsed.event .. " names a key (" .. shown (name) .. ")"
	end
	return nil
end
-- what a well-formed roster event and its well-formed reply did to the keys: an insertion issues one, a removal ends one and a boot ends every one
local function roster_track (parsed, answer)
	if parsed.event == "insert" then
		roster_live [answer.name] = true
	elseif parsed.event == "remove" then
		roster_live [parsed.identity], roster_ended [parsed.identity] = nil, true
	else
		for key in pairs (roster_live) do roster_ended [key] = true end
		roster_live, last_served = {}, nil
		scan.city.parent, scan.city.last = {}, nil
	end
end
-- the first problem of a REAL city reply against the contract, or nil: the state its event allows, a key for an insertion (never issued before) and none for a removal, the building a traversal serves (a live child of
-- the city asked about, never an ended one, empty at the end of the list), and for a LINK and a NAME the key the host acknowledges
local function city_reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if answer.result == "refused" then return "result refused" end
	local wanted = CITY_RESULTS [parsed.event]
	if answer.result ~= wanted then return "a city event of phase " .. parsed.phase .. " (" .. parsed.event .. ") answered " .. tostring (answer.result) .. ", not " .. wanted end
	if answer.p ~= nil then return "a cursor with " .. wanted end
	local name = answer.name
	if name ~= nil and type (name) ~= "string" then return "the name is not text" end
	if parsed.event == "child_insert" then
		if name == nil or name == "" then return "a city insertion names no key" end
		if not is_roster_key (name) then return "the key \"" .. shown (name) .. "\" is not a roster key" end
		if roster_live [name] or roster_ended [name] then return "the key " .. name .. " was issued before" end
	elseif parsed.event == "child_remove" then
		if name ~= nil and name ~= "" then return "a city removal names a key (" .. shown (name) .. ")" end
	elseif parsed.event == "child_first" or parsed.event == "child_next" then
		if name ~= nil and name ~= "" then
			if not is_roster_key (name) then return "the building \"" .. shown (name) .. "\" served is not a roster key" end
			if roster_ended [name] then return "the building " .. name .. " it serves has ended" end
			local parent = scan.city.parent [name]
			if parent ~= nil and parent ~= parsed.keys [1] then return "the building " .. name .. " it serves is listed under " .. parent .. ", not under the city " .. parsed.keys [1] end
		end
	elseif parsed.event == "link" then
		if name ~= parsed.keys [2] then return "a LINK acknowledged \"" .. shown (name) .. "\" for the child " .. parsed.keys [2] end
	elseif name ~= parsed.keys [1] then
		return "a NAME acknowledged \"" .. shown (name) .. "\" for the placement " .. parsed.keys [1]
	end
	return nil
end
-- what a well-formed city event and its well-formed reply did to the keys: an insertion lists a child under its city, a removal ends one, a traversal remembers the building it was last served
local function city_track (parsed, answer)
	local city = scan.city
	if parsed.event == "child_insert" then
		roster_live [answer.name], city.parent [answer.name] = true, parsed.keys [1]
	elseif parsed.event == "child_remove" then
		roster_live [parsed.keys [2]], roster_ended [parsed.keys [2]], city.parent [parsed.keys [2]] = nil, true, nil
	elseif parsed.event == "child_first" or parsed.event == "child_next" then
		local served = answer.name ~= nil and answer.name ~= ""
		city.last = served and { parent = parsed.keys [1], child = answer.name } or nil
		if served then city.served = city.served + 1 end
	end
end
-- the first problem of a well-formed question against the keys seen so far, or nil: a removal names an entry that is not already ended, and a scan stands at the entry the host last served and never at one that ended
local function request_join_problem (parsed)
	if parsed.kind == "city" then
		local keys, city = parsed.keys, scan.city
		if roster_ended [keys [1]] then return "the city event names the placement " .. keys [1] .. ", which has ended" end
		if (parsed.event == "child_remove" or parsed.event == "child_next") and roster_ended [keys [2]] then return "the city event names the building " .. keys [2] .. ", which has ended" end
		if parsed.event == "child_remove" and city.parent [keys [2]] ~= nil and city.parent [keys [2]] ~= keys [1] then return "the removal names the building " .. keys [2] .. " under the city " .. keys [1] .. ", but it is listed under " .. city.parent [keys [2]] end
		if parsed.event == "child_next" and (city.last == nil or city.last.parent ~= keys [1] or city.last.child ~= keys [2]) then
			return "the traversal of the city " .. keys [1] .. " stands at the building " .. keys [2] .. ", which is not the building the host last served"
		end
		return nil
	end
	if parsed.kind == "roster" then
		if parsed.event == "remove" and roster_ended [parsed.identity] then return "a removal names the entry " .. parsed.identity .. ", which has ended" end
		return nil
	end
	if parsed.standing then
		if roster_ended [parsed.standing] then return "the scan stands at the entry " .. parsed.standing .. ", which has ended" end
		if parsed.standing ~= last_served then return "the scan stands at the entry " .. parsed.standing .. ", which is not the entry the host last served (" .. tostring (last_served) .. ")" end
	end
	return nil
end
-- the roster events are counted in `scan.roster`, apart from the scan's own questions (the driver's main function is at the interpreter's limit of upvalues, so no further local is added)
scan.roster = { total = 0, insert = 0, remove = 0, reset = 0, placed = 0, removed = 0, cleared = 0 }
-- the host's own withdrawals of a served entry (withdrawn_backing; 1 once acknowledged `removed`, -1 when it was not) and the phase of the question that followed it, counted apart from the campaign's roster events
scan.withdrawn, scan.withdrawn_next = 0, ""
-- the CITY events (phases 7 to 12), counted apart from the roster's and the scan's: by event and by answer; the buildings served, the parent of each live child, the building a traversal was last served, and the host's own
-- withdrawal of a served building (withdrawn_child; 1 once acknowledged `removed`, -1 when it was not) with the event that followed it
scan.city = { total = 0, child_insert = 0, child_remove = 0, child_first = 0, child_next = 0, link = 0, name = 0, placed = 0, removed = 0, next = 0, served = 0, parent = {}, last = nil, withdrawn = 0, withdrawn_next = "" }
-- one ENTITY_SCAN question: a roster event (phases 4 to 6) or a step of the scan. The two are counted apart in both windows: an event is never a query scan, and no event alters or credits a scan control
local function scan_question (scene, depth, kind, definition, key, counts)
	local roster = scan.roster
	local parsed, why = parse_scan_request (scene, depth, kind, definition, key)
	-- an EVENT is a roster event (phases 4 to 6) or a city event (7 to 12): it is counted apart from the scan's own questions and from the other kind, in both windows, and it is never altered
	local event = parsed ~= nil and parsed.kind ~= "scan"
	if event then counts [parsed.kind] = counts [parsed.kind] + 1 else counts.scan = counts.scan + 1 end
	local real = reference ("entity_scan", scene, depth, kind, definition, key)
	if not armed then
		-- before the launch the host's answer comes back unchanged; only the keys it issued and ended are remembered, so that an armed question can be joined to them
		if event then
			if parsed.kind == "city" then
				if city_reply_problem (real, parsed) == nil then city_track (parsed, real) end
			elseif roster_reply_problem (real, parsed) == nil then
				roster_track (parsed, real)
			end
		end
		return real
	end
	-- the request's join with the keys seen so far, kept for the whole function: a well-formed question that fails it is a request fault and is never altered, like a parse or reply failure
	local joined = parsed ~= nil and request_join_problem (parsed) or nil
	if parsed then
		if joined then
			faults.request = faults.request + 1
			first_fault = first_fault or ("scan request: " .. joined)
		end
		if event and parsed.kind == "city" then
			scan.city.total = scan.city.total + 1
			scan.city [parsed.event] = scan.city [parsed.event] + 1
			if scan.city.withdrawn == 1 and scan.city.withdrawn_next == "" then scan.city.withdrawn_next = parsed.event end
		elseif event then
			roster.total = roster.total + 1
			roster [parsed.event] = roster [parsed.event] + 1
		else
			scan.total = scan.total + 1
			local name = ({ [0] = "phase_begin", "phase_entry", "phase_end", "phase_resolved" }) [parsed.phase]
			scan [name] = scan [name] + 1
			if parsed.excluded then scan.excluded = scan.excluded + 1 end
			if scan.withdrawn == 1 and scan.withdrawn_next == "" then scan.withdrawn_next = name end
			-- the scan begins again, with no entry served
			if parsed.phase == 0 then last_served = nil end
		end
	else
		faults.request = faults.request + 1
		first_fault = first_fault or ("scan request: " .. why)
	end
	local problem
	if event and parsed.kind == "city" then problem = city_reply_problem (real, parsed)
	elseif event then problem = roster_reply_problem (real, parsed)
	else problem = scan_reply_problem (real, parsed) end
	if problem then
		faults.reply = faults.reply + 1
		first_fault = first_fault or ("scan reply: " .. problem)
	elseif event and parsed.kind == "city" then
		scan.city [real.result] = scan.city [real.result] + 1
		city_track (parsed, real)
	elseif event then
		roster [real.result] = roster [real.result] + 1
		roster_track (parsed, real)
	else
		scan [real.result] = scan [real.result] + 1
		if real.result == "first" or real.result == "next" then
			last_served = (real.name ~= nil and real.name ~= "") and real.name or nil
		elseif real.result == "done_none" or real.result == "done_hit" then
			last_served = nil
		end
	end
	-- HOST-SIDE (withdrawn_backing): once, on the first real FIRST that serves an entry, the host's own REMOVE of that key goes to the SAME provider and must come back `removed`; the FIRST reply is handed on
	-- completely unchanged, so the campaign's lists and the entity are untouched and only the host's roster has lost the entry
	if plan.withdraw_backing and not event and not problem and scan.withdrawn == 0 and real.result == "first" and real.name ~= nil and real.name ~= "" then
		local words = {}
		for index = 1, 16 do words [index] = ZERO end
		words [12] = "0x40a00000"
		local ok, acknowledged = pcall (reference, "entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. " " .. real.name)
		if ok and type (acknowledged) == "table" and acknowledged.result == "removed" and (acknowledged.name == nil or acknowledged.name == "") then
			scan.withdrawn = 1
		else
			scan.withdrawn = -1
			faults.reply = faults.reply + 1
			first_fault = first_fault or "scan reply: the host's removal of the served entry was not acknowledged as removed"
		end
	end
	-- HOST-SIDE (withdrawn_child): once, on the first real city traversal reply that serves a building, the host's own REMOVE of that building (the city's key and the building's, phase 8) goes to the SAME provider and must
	-- come back `removed`; the reply is handed on completely unchanged, so the campaign's lists, the building and its physics are untouched and only the host's list has lost the building. The campaign's next
	-- question of the traversal then stands at the ended building (the wrapper does not record the removal, so the request join passes) and the provider refuses it by name
	if plan.withdraw_child and event and parsed.kind == "city" and not problem and scan.city.withdrawn == 0 and (parsed.event == "child_first" or parsed.event == "child_next") and real.name ~= nil and real.name ~= "" then
		local words = {}
		for index = 1, 16 do words [index] = ZERO end
		words [12] = "0x41000000"
		local ok, acknowledged = pcall (reference, "entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. " " .. parsed.keys [1] .. " " .. real.name)
		if ok and type (acknowledged) == "table" and acknowledged.result == "removed" and (acknowledged.name == nil or acknowledged.name == "") then
			scan.city.withdrawn = 1
		else
			scan.city.withdrawn = -1
			faults.reply = faults.reply + 1
			first_fault = first_fault or "scan reply: the host's removal of the served building was not acknowledged as removed"
		end
	end
	-- an event is never altered
	if event then return real end
	if plan.scan == nil then return real end
	-- an altered answer is formed only from a well-formed question, a request that joins the keys seen and a well-formed real reply (a witness failure raised here would come back carrying the callback marker the refusal mode credits)
	if parsed == nil or joined or problem then return real end
	if plan.eligible_scan and not plan.eligible_scan (parsed, real) then return real end
	if plan.once and scan_altered >= 1 then return real end
	scan_altered = scan_altered + 1
	local answer = plan.scan (parsed, real)
	if type (answer) == "table" and answer.result == "done_none" then scan_none_answered = scan_none_answered + 1 end
	return answer
end
-- the pose request as the contract states it: scene 0, the stage (0 or 1) in depth, the entity's type in kind, whether it is alive (0 or 1) in definition, and the text place of 15 patterns followed, when the
-- campaign has an identity for the candidate, by one space and the identity
local WORDS_TEXT = 15 * 10 + 14
local function parse_pose_request (scene, depth, kind, definition, key)
	if scene ~= 0 then return nil, "scene " .. tostring (scene) end
	if depth ~= 0 and depth ~= 1 then return nil, "stage " .. tostring (depth) end
	if not finite (kind) or kind ~= math.floor (kind) then return nil, "entity type " .. tostring (kind) end
	if definition ~= 0 and definition ~= 1 then return nil, "alive " .. tostring (definition) end
	if type (key) ~= "string" then return nil, "no words text" end
	local text, host_id = key, ""
	if #key > WORDS_TEXT then
		if string.sub (key, WORDS_TEXT + 1, WORDS_TEXT + 1) ~= " " then return nil, "the identity does not follow the words after one space" end
		text, host_id = string.sub (key, 1, WORDS_TEXT), string.sub (key, WORDS_TEXT + 2)
		if #host_id > 128 then return nil, "the identity is " .. #host_id .. " bytes, outside the roster's domain of 1 to 128" end
	end
	if words_of_text (text, 15) == nil then return nil, "the words are not 15 comma separated 0x%08x patterns" end
	return { stage = depth, entity_type = kind, alive = definition, host_id = host_id, named = host_id ~= "" }
end
local POSE_CLASSES = { recorded = true, unrecorded = true, retired = true, unknown = true, no_id = true }
-- the first problem of a REAL pose reply against the contract, or nil: one of the five classes, a kind of 0..5 and a difference of 0..31 (whole numbers), nothing but a record carries either, a record has a kind, and
-- no_id exactly when the question named no identity
local function pose_reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if not POSE_CLASSES [answer.result] then return "result " .. tostring (answer.result) end
	local kind, difference = answer.kind, answer.difference
	if not finite (kind) or kind ~= math.floor (kind) or kind < 0 or kind > 5 then return "kind " .. tostring (kind) end
	if not finite (difference) or difference ~= math.floor (difference) or difference < 0 or difference > 31 then return "difference " .. tostring (difference) end
	if answer.result ~= "recorded" and (kind ~= 0 or difference ~= 0) then return "a kind or difference with " .. answer.result end
	if answer.result == "recorded" and kind == 0 then return "a record of no kind" end
	if parsed and (answer.result == "no_id") == parsed.named then return "the class " .. answer.result .. " for a question that " .. (parsed.named and "named" or "did not name") .. " an identity" end
	-- the stage's own masks: the offer stage reads no attitude, and a difference from a held vector contradicts "none held"
	if parsed and parsed.stage == 0 and math.floor (difference / 2) % 2 == 1 then return "an attitude difference (" .. difference .. ") at the offer stage, which carries no attitude" end
	if math.floor (difference / 4) % 2 == 1 and math.floor (difference / 8) % 2 == 1 then return "a difference from a held motion vector together with none held (" .. difference .. ")" end
	return nil
end
local function pose_question (scene, depth, kind, definition, key, is_armed)
	local book, histogram = pose, pose_histogram
	if not is_armed then book, histogram = pose_pre, pose_pre_histogram end
	local parsed, why = parse_pose_request (scene, depth, kind, definition, key)
	if parsed then
		book.total = book.total + 1
		if parsed.stage == 0 then book.stage_offer = book.stage_offer + 1 else book.stage_placement = book.stage_placement + 1 end
		if parsed.named then book.named = book.named + 1 end
	else
		faults.request = faults.request + 1
		first_fault = first_fault or ("pose request: " .. why)
	end
	-- HOST-SIDE: the host places this mobile's record somewhere else (the first named armed observation only), so the campaign's words no longer match it
	if is_armed and plan.displace and parsed and parsed.named and displaced == 0 then
		displaced = displaced + 1
		host.move_vehicle ({ host_id = parsed.host_id, mode = "place", pose = { position = { 1, 2, 3 }, velocity = 0, attitude = { 1, 0, 0, 0, 1, 0, 0, 0, 1 } }, motion_vector_in = { 0, 0, 0 } })
	end
	local real = reference ("entity_pose", scene, depth, kind, definition, key)
	local problem = pose_reply_problem (real, parsed)
	if problem then
		faults.reply = faults.reply + 1
		first_fault = first_fault or ("pose reply: " .. problem)
	else
		book [real.result] = book [real.result] + 1
		if parsed then
			local bucket = string.format ("%d:%d:%d:%s:%d:%d", parsed.stage, parsed.entity_type, parsed.alive, real.result, real.kind, real.difference)
			histogram [bucket] = (histogram [bucket] or 0) + 1
		end
	end
	if not is_armed or plan.pose == nil then return real end
	-- an altered answer is formed only from a well-formed question and real reply (a witness failure raised here would come back carrying the callback marker the refusal mode credits)
	if parsed == nil or problem then return real end
	if plan.eligible_pose and not plan.eligible_pose (parsed, real) then return real end
	if plan.once and pose_altered >= 1 then return real end
	pose_altered = pose_altered + 1
	return plan.pose (parsed, real)
end
local function pose_histogram_text (histogram)
	local keys = {}
	for bucket in pairs (histogram) do keys [#keys + 1] = bucket end
	table.sort (keys)
	local parts = {}
	for _, bucket in ipairs (keys) do parts [#parts + 1] = bucket .. "=" .. histogram [bucket] end
	return table.concat (parts, ";")
end
-- the facts the cutover needs, summed over BOTH phases' histograms (each bucket is stage:type:alive:class:kind:difference); the denominators are in the header: offers and every offer-level fact are STAGE-0 observations, the attitude facts STAGE-1
local function pose_facts ()
	local facts = {
		observations = 0, offers = 0, no_id = 0, unrecorded = 0, retired = 0, unknown = 0, recorded = 0, dead = 0, dead_recorded = 0, position_differs = 0, attitude_compared = 0, attitude_differs = 0,
		motion_held = 0, motion_held_differs = 0, motion_not_held = 0, formula_equal = 0, formula_differs = 0, vehicle_formula_equal = 0, vehicle_formula_differs = 0,
		aircraft_motion_differs = 0,
	}
	local by_kind = {}
	local function has (difference, bit) return math.floor (difference / bit) % 2 == 1 end
	for _, histogram in ipairs ({ pose_histogram, pose_pre_histogram }) do
		for bucket, count in pairs (histogram) do
			local stage, _, alive, class, kind, difference = string.match (bucket, "^(%d+):(%d+):(%d+):([%a_]+):(%d+):(%d+)$")
			stage, alive, kind, difference = tonumber (stage), tonumber (alive), tonumber (kind), tonumber (difference)
			facts.observations = facts.observations + count
			if stage == 1 then
				-- the placement observation repeats the offer's position and motion: only the attitude is its own
				if class == "recorded" then
					facts.attitude_compared = facts.attitude_compared + count
					if has (difference, 2) then facts.attitude_differs = facts.attitude_differs + count end
				end
			else
			facts.offers = facts.offers + count
			facts [class] = facts [class] + count
			if alive == 0 then facts.dead = facts.dead + count end
			if class == "recorded" then
				if alive == 0 then facts.dead_recorded = facts.dead_recorded + count end
				by_kind [kind] = (by_kind [kind] or 0) + count
				if has (difference, 1) then facts.position_differs = facts.position_differs + count end
				if has (difference, 8) then facts.motion_not_held = facts.motion_not_held + count
				else
					facts.motion_held = facts.motion_held + count
					if has (difference, 4) then facts.motion_held_differs = facts.motion_held_differs + count end
				end
				if has (difference, 16) then facts.formula_differs = facts.formula_differs + count else facts.formula_equal = facts.formula_equal + count end
				if kind >= 1 and kind <= 3 then
					if has (difference, 16) then facts.vehicle_formula_differs = facts.vehicle_formula_differs + count else facts.vehicle_formula_equal = facts.vehicle_formula_equal + count end
				elseif has (difference, 4) then
					facts.aircraft_motion_differs = facts.aircraft_motion_differs + count
				end
			end
			end
		end
	end
	local kinds = {}
	for kind = 1, 5 do if by_kind [kind] then kinds [#kinds + 1] = kind .. "=" .. by_kind [kind] end end
	facts.recorded_by_kind = table.concat (kinds, ";")
	return facts
end
-- The host's own movement callback as the engine calls it (host.move_vehicle). Every step reaches the host unchanged except, in terrain_lift and only after the launch, the target's own routed `move` step, whose
-- terrain elevation is raised by LIFT when that stays within the step's map ceiling. The step is copied: the engine's table is never mutated. Each `move` answer is logged per identity with its frame (the host's
-- answer to the campaign, used only to select the case; it is not provider state).
local function hosted_move (step)
	local delegated, lifting, demanding = step, false, false
	-- the callback's own input shape, read before the delegation and kept as booleans: whether the step carries the terrain input and the map ceiling the bounded lift reads, and a finite positive
	-- desired_velocity (the speed demand the solver receives) (types, finiteness and sign only, no arithmetic)
	local terrain_input, lift_input, speed_input = false, false, false
	if type (step) == "table" then
		terrain_input = type (step.terrain) == "table"
		lift_input = terrain_input and type (step.terrain.elevation) == "number" and type (step.map_max) == "table" and type (step.map_max [2]) == "number"
		speed_input = finite (step.desired_velocity) and step.desired_velocity > 0
		if tgt.window_open and watched ~= nil and step.host_id == watched then
			tgt.window_steps = tgt.window_steps + 1
			if step.mode == "move" then tgt.window_moves = tgt.window_moves + 1 end
			if step.mode == "move" and terrain_input then tgt.window_with_terrain = tgt.window_with_terrain + 1 end
			if step.mode == "move" and plan.speed then
				if not speed_input then
					-- a missing, non-numeric, non-finite or non-positive demand is never replaced and never credited
					tgt.speed_bad = tgt.speed_bad + 1
				else
					tgt.speed_valid = tgt.speed_valid + 1
					if plan.speed_zero then
						-- the only change: a shallow copy whose desired_velocity is 0 (every other field, and every table in it, is the engine's own); delegated once below
						delegated = {}
						for key, value in pairs (step) do delegated [key] = value end
						delegated.desired_velocity = 0
						demanding = true
						tgt.speed_transforms = tgt.speed_transforms + 1
						if tgt.speed_incoming == "none" then tgt.speed_incoming, tgt.speed_substituted = string.format ("%.17g", step.desired_velocity), "0" end
					end
				end
			end
		end
	end
	if plan.lift and (armed or lift_window) and watched ~= nil and type (step) == "table" and step.host_id == watched and step.mode == "move" and type (step.terrain) == "table" then
		local base = step.terrain.elevation
		local ceiling = type (step.map_max) == "table" and step.map_max [2] or nil
		if type (base) == "number" and base == base and type (ceiling) == "number" and base + LIFT <= ceiling then
			delegated = {}
			for key, value in pairs (step) do delegated [key] = value end
			delegated.terrain = {}
			for key, value in pairs (step.terrain) do delegated.terrain [key] = value end
			delegated.terrain.elevation = base + LIFT
			lifting = true
			lift_row.baseline, lift_row.elevation, lift_row.frame = base, base + LIFT, frames
		else
			lift_refused = lift_refused + 1
		end
	end
	local answer = host.move_vehicle (delegated)
	if type (step) == "table" and step.mode == "move" and type (answer) == "table" then
		last_move [step.host_id] = { frame = frames, motion = answer.motion_vector, position = answer.position, terrain_input = terrain_input, lift_input = lift_input, speed_input = speed_input, demand = step.desired_velocity }
	end
	if demanding then tgt.speed_delegations = tgt.speed_delegations + 1 end
	if lifting then
		lifted = lifted + 1
		lift_row.answered_y = type (answer) == "table" and type (answer.position) == "table" and answer.position [2] or 0 / 0
	end
	return answer
end
local function candidate_question (scene, depth, kind, definition, key)
	local parsed, why = parse_candidate_request (scene, depth, kind, definition, key)
	if parsed then
		cand.total = cand.total + 1
		if parsed.scene == 0 then cand.scene0 = cand.scene0 + 1 end
		if parsed.mobile then
			cand.mobile = cand.mobile + 1
			if parsed.named then cand.mobile_named = cand.mobile_named + 1 else cand.mobile_unnamed = cand.mobile_unnamed + 1 end
			-- HOST-SIDE: the host retires the identity this question names, before the question is delegated (the first one only)
			if plan.withdraw and parsed.named and withdrawn == 0 and plan.candidate == nil then
				withdrawn = withdrawn + 1
				host.move_vehicle ({ host_id = parsed.host_id, mode = "retire" })
			end
		end
	else
		faults.request = faults.request + 1
		first_fault = first_fault or ("candidate request: " .. why)
	end
	local real = reference ("entity_candidate", scene, depth, kind, definition, key)
	local problem = candidate_reply_problem (real, scene)
	if problem then
		faults.reply = faults.reply + 1
		first_fault = first_fault or ("candidate reply: " .. problem)
	else
		cand [real.result] = cand [real.result] + 1
		-- the questions about the case target itself, by the host's identity (and those after a lifted step): the per-identity evidence the global counts cannot give
		if parsed and parsed.mobile and watched ~= nil and parsed.host_id == watched then
			tgt.questions = tgt.questions + 1
			tgt [real.result] = tgt [real.result] + 1
			-- a test decision of a MOVING candidate carries the 16-word moving pose (a stationary one 12)
			if real.result == "test" and type (real.p) == "string" then
				local words = 0
				for _ in string.gmatch (real.p, "[^,]+") do words = words + 1 end
				if words == 16 then tgt.test_moving16 = tgt.test_moving16 + 1 end
			end
			if lifted > 0 then
				tgt.after_lift = tgt.after_lift + 1
				if real.result == "test" then tgt.after_lift_test = tgt.after_lift_test + 1 end
			end
			if tgt.speed_delegations > 0 then
				tgt.after_speed = tgt.after_speed + 1
				if real.result == "test" then tgt.after_speed_test = tgt.after_speed_test + 1 end
			end
		end
	end
	if plan.candidate == nil then return real end
	-- an altered answer is formed only from a well-formed question and real reply (a witness failure raised here would come back carrying the callback marker the refusal mode credits)
	if parsed == nil or problem then return real end
	if plan.eligible and not plan.eligible (parsed, real) then return real end
	if plan.once and cand_altered >= 1 then return real end
	cand_altered = cand_altered + 1
	local answer = plan.candidate (parsed, real)
	if type (answer) == "table" and answer.result == "skip" then cand_skipped = cand_skipped + 1 end
	return answer
end
local function model (op, scene, depth, kind, definition, key)
	local counts = armed and after or before
	if op == "entity_collision" then counts.entity = counts.entity + 1
	elseif op == "entity_candidate" then counts.candidate = counts.candidate + 1
	elseif op == "entity_scan" then -- counted by kind (a roster event or a scan question) in scan_question
	elseif op == "entity_pose" then counts.pose = counts.pose + 1
	else counts.other = counts.other + 1 end
	-- the armed candidate questions are counted and classified; every other operation, and every question before the launch, is the host's own answer, returned unchanged
	if op == "entity_scan" then return scan_question (scene, depth, kind, definition, key, counts) end
	if op == "entity_pose" then return pose_question (scene, depth, kind, definition, key, armed) end
	if op == "entity_candidate" and armed then return candidate_question (scene, depth, kind, definition, key) end
	if op ~= "entity_collision" or not armed then return reference (op, scene, depth, kind, definition, key) end
	local parsed, why = parse_request (scene, depth, kind, definition, key)
	if parsed then
		if parsed.moving then seen.moving = seen.moving + 1 else seen.stationary = seen.stationary + 1 end
	else
		faults.request = faults.request + 1
		first_fault = first_fault or ("request: " .. why)
	end
	local real = reference (op, scene, depth, kind, definition, key)
	local problem = reply_problem (real)
	if problem then
		faults.reply = faults.reply + 1
		first_fault = first_fault or ("reply: " .. problem)
	else
		if real.result == "hit" then seen.real_hit = seen.real_hit + 1 else seen.real_miss = seen.real_miss + 1 end
		if real.mask == 63 then seen.real_mask_63 = seen.real_mask_63 + 1 elseif real.mask == 56 then seen.real_mask_56 = seen.real_mask_56 + 1 else seen.real_mask_other = seen.real_mask_other + 1 end
	end
	if plan.answer == nil then return real end
	-- an altered answer is formed only from a well-formed question and real reply; otherwise the host's own answer is returned unaltered and the fault (counted above) fails the verdict. A witness failure is NOT raised
	-- here: it would cross the engine's callback boundary and come back carrying the callback marker, which the refusal modes credit as the host withholding an answer
	if parsed == nil or problem then return real end
	altered = altered + 1
	local answer = plan.answer (parsed, real)
	if type (answer) == "table" then
		if answer.result == "hit" then seen.answered_hit = seen.answered_hit + 1 elseif answer.result == "miss" then seen.answered_miss = seen.answered_miss + 1 end
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- the controlled view input (an aim delegate over the unmodified host.aim)
local VIEW_WEAPON = 1 -- mb_hostaim.h CAMPAIGN_STEP_LOOKUP_VIEW_WEAPON; its answer r0 is f[7] in Lua (f[6..11] hold r0..r5)
local view = { steps_seen = 0, answers_seen = 0, applied = 0, already_set = 0 }
local base_aim -- the host's own aim function, resolved inside the owned close scope (main)
-- a copy of one lookup answer with the sixth word set: kind, a..d and all twelve floats are copied, nothing is shared with the caller's table
local function viewed_answer (answer)
	if type (answer.f) ~= "table" or #answer.f ~= 12 then hard ("a VIEW_WEAPON answer does not carry twelve result words, so the controlled view cannot be applied exactly") end
	local copy = { kind = answer.kind, a = answer.a, b = answer.b, c = answer.c, d = answer.d, f = {} }
	for index = 1, 12 do copy.f [index] = answer.f [index] end
	if copy.f [7] ~= 0 then view.already_set = view.already_set + 1 end
	copy.f [7] = 1
	return copy
end
local function aim (request)
	if not armed and type (request) == "table" and request.mode == "vectors" and type (request.source_host_id) == "string" and type (request.target_host_id) == "string"
		and type (request.weapon) == "table" and type (request.weapon.weapon) == "number" and type (request.target) == "table" then
		last_vectors [request.source_host_id] = { frame = frames, target = request.target_host_id, weapon = request.weapon.weapon, motion = request.target.motion_vector }
	end
	if armed and type (request) == "table" and request.mode == "weapon_step" then
		view.steps_seen = view.steps_seen + 1
		local answers = request.answers
		if type (answers) == "table" then
			local changed, replaced = false, {}
			for index, answer in ipairs (answers) do
				if type (answer) == "table" and answer.kind == VIEW_WEAPON then
					view.answers_seen = view.answers_seen + 1
					replaced [index] = viewed_answer (answer)
					changed = true
				else
					replaced [index] = answer
				end
			end
			if changed then
				local copy = {}
				for key, value in pairs (request) do copy [key] = value end
				copy.answers = replaced
				view.applied = view.applied + 1
				request = copy
			end
		end
	end
	return base_aim (request)
end


-- ---------------------------------------------------------------------------------------------------------------- the run
local map = "..\\common\\maps\\map5"
local function main ()
	-- every fallible setup step is inside the owned close scope: a missing or raising factory is a failure of the witness with the evidence closed
	reference = host.model
	if type (reference) ~= "function" then hard ("this host has no host.model") end
	local terrain_factory = host.terrain_provider
	if type (terrain_factory) ~= "function" then hard ("this host has no host.terrain_provider") end
	local terrain = terrain_factory (root, map)
	base_aim = host.aim
	if type (base_aim) ~= "function" then hard ("this host has no host.aim") end
	local descriptors, order = {}, {}
	local function bind_mobile (descriptor)
		local id = host.bind_mobile (descriptor)
		descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
		order [#order + 1] = id
		return id
	end

	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	local engine = dc.boot {
		install_root = root, map = map, terrain = terrain, environment = host.env_provider (), campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
		seed = seed, bind_mobile = bind_mobile, move_vehicle = hosted_move, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = model, sense = host.sense, aim = aim,
	}
	emit { event = "boot", mode = mode, profile = profile, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed, frame_budget = frame_budget }

	-- the original profile needs the fixture API of collision-fixture; a module without it is not the original profile
	local fixture_api = false
	do
		local ok, value = pcall (function () return engine.collision_probe end)
		fixture_api = ok and type (value) == "function"
	end
	if profile == "original" and not fixture_api then hard ("UNREACHED: profile=original, but the module has no collision-fixture API (it was not built with collision-fixture): this is not the original profile") end
	if profile == "hosted" and fixture_api then hard ("profile=hosted, but the module has the collision-fixture API: it keeps the original narrow phase and is not the shipped module (use profile=original)") end

	local clock = engine:clock ()
	-- one frame, with finite progress: the simulated clock must be a finite number that has moved forward, and the run may not use more frames than its budget
	local function frame ()
		if frames >= frame_budget then hard ("FRAME BUDGET: " .. frames .. " frames were run and the simulated clock is at " .. tostring (clock.elapsed_seconds) .. " s") end
		local was = clock.elapsed_seconds
		engine:frame (frame_ms)
		frames = frames + 1
		clock = engine:clock ()
		local now = clock.elapsed_seconds
		if not finite (now) or not (now > was) then hard ("STALLED CLOCK: a frame left the simulated clock at " .. tostring (now) .. " s from " .. tostring (was) .. " s") end
	end
	local function step (seconds_more)
		local until_time = clock.elapsed_seconds + seconds_more
		while clock.elapsed_seconds < until_time do frame () end
	end
	-- the engine's own stop, or nil. Only the anchored engine fatal is the engine's; a witness failure is re-raised as it is; anything else is a failure of the witness, not proof of anything
	local function guarded (call, ...)
		local ok, err = pcall (call, ...)
		if ok then return nil end
		local text = tostring (err)
		if string.sub (first_line (text), 1, #NAME + 1) == NAME .. ":" then error (text, 0) end
		local marker = engine_marker (text)
		if marker == nil then hard ("an unrecognised failure, not the engine's known fatal at the model boundary: " .. first_line (text)) end
		return { marker = marker, line = first_line (text) }
	end
	local function engine_fatal_text (err)
		local text = tostring (err)
		return text:find ("EECH fatal error", 1, true) ~= nil or text:find ("poisoned", 1, true) ~= nil
	end

	local setup = guarded (step, warmup)
	if setup then hard ("the engine stopped during the warmup, before anything was altered: [model-provider:" .. setup.marker .. "]") end
	emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, entity_before = before.entity }

	-- ------------------------------------------------------------------------------------------------------------ the case launch (preconditions, deterministic)
	local function fields (line)
		local t = {}
		for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
		return t
	end
	local function numbers (list)
		local t = {}
		for v in string.gmatch (list, "[^,]+") do t [#t + 1] = tonumber (v) end
		return t
	end
	-- a query that fails for its arguments is a candidate that does not qualify; a failure of the ENGINE is never swallowed
	local function state (id, weapon)
		local ok, text = pcall (engine.wpn_case_query, engine, id, weapon or 0)
		if not ok then
			if engine_fatal_text (text) then hard ("the engine had stopped before the launch: " .. first_line (text)) end
			return nil
		end
		local s = fields (text)
		s.alive, s.weapon_count = tonumber (s.alive), tonumber (s.weapon_count)
		s.warhead, s.guidance, s.aiming, s.decoy = tonumber (s.warhead), tonumber (s.guidance), tonumber (s.aiming), tonumber (s.decoy)
		s.position = numbers (s.position or "")
		return s
	end
	local function events ()
		local list = {}
		for line in string.gmatch (engine:wpn_events (), "[^\n]+") do
			local f = fields (line)
			f.raw = line
			list [#list + 1] = f
		end
		return list
	end
	-- one named counter of the weapon fixture's run-wide report
	local function counter (name)
		local text = engine:wpn_trace ()
		local value = string.match (text, "wpn%-counter name=" .. name .. " value=(%d+)")
		if value == nil then hard ("the weapon fixture's report has no counter " .. name) end
		return tonumber (value)
	end

	local ordered = {}
	for _, id in ipairs (order) do ordered [#ordered + 1] = id end
	table.sort (ordered, function (a, b) return tonumber (string.match (a, "(%d+)$")) < tonumber (string.match (b, "(%d+)$")) end)
	local alive = {}
	for _, id in ipairs (ordered) do
		local s = state (id, 0)
		if s and s.alive == 1 then s.side = descriptors [id].side alive [id] = s end
	end
	-- the case: the pair of a launcher and an enemy that is a collision candidate TYPE (not a person: a person has neither collision flag) with the smallest distance, whose launcher has an
	-- unguided ballistic weapon in stock. The weapon's lookup excludes only the launcher, so the target is the weapon's first broad-phase candidate when it is within the candidate's box
	-- of radius + 10 m of the first segment; a war in which no pair is that close simply makes no narrow-phase question and the run is UNREACHED, never a pass.
	local PERSON = 3 -- host.bind_mobile movement_kind of a person
	local ROUTED = 1 -- host.bind_mobile movement_kind of a routed vehicle
	-- moving_control / terrain_lift: the enemy must be a routed vehicle the host actually stepped in the final warmup frame with a motion vector above 1 m/s (the host's own answer, never a guess from its kind)
	local function moving_routed (id)
		local last = last_move [id]
		if descriptors [id].kind ~= ROUTED or last == nil or last.frame ~= frames - 1 or type (last.motion) ~= "table" then return false end
		local x, y, z = last.motion [1], last.motion [2], last.motion [3]
		if not (finite (x) and finite (y) and finite (z)) then return false end
		return x * x + y * y + z * z > 1
	end
	if plan.query then
		-- the controlled moving QUERY case (see the header): no weapon, no launch, no cohort
		-- the moving criterion is unchanged; the target must ALSO be one whose final warmup `move` step the host was actually given with the terrain input and map ceiling the bounded lift reads (the callback shape
		-- hosted_move recorded from the engine's own step: a routed step the core gives no terrain, e.g. no terrain-follow guide or a waypoint within a metre, is a valid ordinary movement but cannot carry the lift).
		-- The same rule selects in the control and the lifted mode, hosted and original, so the pair asks about one actor; an earlier moving actor without it is counted, never mutated or given a terrain input
		-- the speed-demand modes (moving_query_speed_*) use the same moving criterion and the same order, but their input is the speed demand: no terrain is required of them
		local target, moving_candidates, input_candidates, terrain_only, speed_candidates = nil, 0, 0, 0, 0
		for _, id in ipairs (ordered) do
			if alive [id] and moving_routed (id) then
				moving_candidates = moving_candidates + 1
				if last_move [id].speed_input then speed_candidates = speed_candidates + 1 end
				if plan.speed then
					if last_move [id].speed_input then target = target or id end
				elseif last_move [id].lift_input then
					input_candidates = input_candidates + 1
					target = target or id
				elseif last_move [id].terrain_input then
					terrain_only = terrain_only + 1
				end
			end
		end
		emit { event = "query_case_selected", target = target or "none", moving_candidates = moving_candidates, lift_input_candidates = input_candidates, terrain_without_ceiling_candidates = terrain_only,
			speed_input_candidates = speed_candidates, selected_demand = (target and last_move [target].speed_input) and string.format ("%.17g", last_move [target].demand) or "none",
			selection = "first_alive_moving_routed_target_in_identity_order",
			eligibility = plan.speed and "final_warmup_move_step_carried_finite_positive_desired_velocity" or "final_warmup_move_step_carried_terrain_input_and_map_ceiling", final_frame = frames - 1 }
		if not target and moving_candidates == 0 then hard ("UNREACHED: no alive routed vehicle was stepped by the host above 1 m/s in the final warmup frame (" .. #ordered .. " bound): the controlled query case is not run, nothing is tuned") end
		if not target and plan.speed then hard (string.format ("UNREACHED: %d alive routed vehicles were stepped by the host above 1 m/s in the final warmup frame (%d bound), but none of those move steps carried a finite positive desired_velocity: the controlled query case is not run, nothing is tuned", moving_candidates, #ordered)) end
		if not target then hard (string.format ("UNREACHED: %d alive routed vehicles were stepped by the host above 1 m/s in the final warmup frame (%d bound), but none of those move steps carried the terrain input and map ceiling the bounded lift needs (%d carried a terrain input without the ceiling): the controlled query case is not run, nothing is tuned", moving_candidates, #ordered, terrain_only)) end
		for _, method in ipairs { "wpn_moving_query_prepare", "wpn_moving_query_run" } do
			local present, value = pcall (function () return engine [method] end)
			if not (present and type (value) == "function") then hard ("this module has no controlled moving query surface (" .. method .. "): it was not built with weapon-fixture") end
		end
		watched = target
		-- one engine call: nil and a failure when the engine itself stopped (anchored marker); a refusal of the case (a named reason) is UNREACHED, never swallowed
		local function ask (method, ...)
			local ok, text = pcall (engine [method], engine, ...)
			if ok then return text, nil end
			if string.sub (first_line (text), 1, #NAME + 1) == NAME .. ":" then error (tostring (text), 0) end
			local marker = engine_marker (text)
			if marker == nil then
				-- the bridge keeps the case's NAMED refusal (`wpn-case-error reason=...`, the non-poison argument status); a failure without one is not a refusal of the case and is never read as UNREACHED
				if not string.find (tostring (text), "wpn-case-error reason=", 1, true) then hard ("the controlled query case failed " .. method .. " without a named case refusal (the bridge lost its reason?): " .. first_line (text)) end
				hard ("UNREACHED: the controlled query case refused " .. method .. ": " .. first_line (text))
			end
			return nil, { marker = marker, line = first_line (text) }
		end
		local prepared, failure = ask ("wpn_moving_query_prepare", target)
		if failure then hard ("the engine stopped while the controlled query case was prepared: [model-provider:" .. failure.marker .. "]") end
		local head, latched = mq_parse_prepare (prepared, target)
		if not head then hard ("the prepare report is malformed: " .. latched) end
		emit { event = "query_prepared", target = target, target_type = head.target_type, radius = head.radius,
			segments = table.concat ({ latched [1].a .. ">" .. latched [1].b, latched [2].a .. ">" .. latched [2].b, latched [3].a .. ">" .. latched [3].b }, ";") }
		-- ONE actual engine frame, the lift (when the mode has it) in the target's own host step; then the SAME segments are asked
		lift_window = plan.lift and true or false
		tgt.window_open = true
		failure = guarded (frame)
		lift_window, tgt.window_open = false, false
		local ran
		if not failure then
			armed = true
			ran, failure = ask ("wpn_moving_query_run")
		end
		local results, hits, target_hits, signature = {}, 0, 0, {}
		if ran then
			local run_head
			run_head, results = mq_parse_run (ran, target, latched)
			if not run_head then hard ("the query run report is malformed: " .. results) end
			for _, result in ipairs (results) do
				if result.hit == "1" then
					hits = hits + 1
					if result.hit_is_target == "1" then target_hits = target_hits + 1 end
				end
				signature [#signature + 1] = table.concat ({ result.hit, result.hit_type, result.hit_is_target, result.hit_host_id, result.point, result.normal }, "|")
				emit { event = "query_segment", seg = tonumber (result.seg), hit = tonumber (result.hit), hit_is_target = tonumber (result.hit_is_target), hit_type = result.hit_type, hit_host_id = result.hit_host_id,
					point = result.point, normal = result.normal, a = result.a, b = result.b }
			end
		end
		local pose_facts_row = pose_facts ()
		emit {
			event = "query_authority", mode = mode, profile = profile, completed = failure == nil and ran ~= nil, failure_marker = failure and failure.marker or "", failure_line = failure and failure.line or "",
			target = target, segments = #results, query_hits = hits, query_target_hits = target_hits, query_signature = table.concat (signature, ";"),
			lifted = lifted, lift_refused = lift_refused, lift_constant = LIFT, lift_baseline_elevation = lift_row.baseline, lift_elevation = lift_row.elevation, lift_answered_y = lift_row.answered_y, lift_frame = lift_row.frame,
			window_target_steps = tgt.window_steps, window_target_moves = tgt.window_moves, window_target_moves_with_terrain = tgt.window_with_terrain,
			speed_window_valid = tgt.speed_valid, speed_window_bad = tgt.speed_bad, speed_transforms = tgt.speed_transforms, speed_delegations = tgt.speed_delegations,
			speed_incoming_demand = tgt.speed_incoming, speed_substituted_demand = tgt.speed_substituted, target_after_speed = tgt.after_speed, target_after_speed_test = tgt.after_speed_test,
			target_questions = tgt.questions, target_skip = tgt.skip, target_need_updated = tgt.need_updated, target_need_fixed = tgt.need_fixed, target_need_placement = tgt.need_placement,
			target_need_normal = tgt.need_normal, target_buildings = tgt.buildings, target_test = tgt.test, target_test_moving16 = tgt.test_moving16, target_after_lift = tgt.after_lift, target_after_lift_test = tgt.after_lift_test,
			candidate_after = after.candidate, candidate_mobile = cand.mobile, candidate_mobile_named = cand.mobile_named, candidate_mobile_unnamed = cand.mobile_unnamed, entity_after = after.entity,
			scan_after = after.scan, pose_after = after.pose, pose_recorded = pose.recorded, pose_named = pose.named, pose_histogram = pose_histogram_text (pose_histogram),
			pose_fact_position_differs = pose_facts_row.position_differs, pose_fact_attitude_differs = pose_facts_row.attitude_differs, pose_fact_motion_held_differs = pose_facts_row.motion_held_differs,
			moving_pose_questions = seen.moving, request_faults = faults.request, reply_faults = faults.reply, first_fault = first_fault or "",
			frames = frames, elapsed = clock.elapsed_seconds, natural_weapon_engagement = "not_claimed_query_service_only",
		}
		-- ------------------------------------------------------------------------------------------------ the verdict of the query modes
		if failure then return "mode " .. mode .. " failed in the engine at [model-provider:" .. failure.marker .. "]: " .. failure.line end
		if plan.lift then
			if lifted < 1 then return string.format ("UNREACHED: %s lifted no step of the target (%d refused by the map ceiling; in the query frame the target was given %d steps, %d of mode move, %d of those with a terrain input): the host's own routed move of the target never carried a terrain input within the bound", mode, lift_refused, tgt.window_steps, tgt.window_moves, tgt.window_with_terrain) end
			local y, e = lift_row.answered_y, lift_row.elevation
			if not finite (y) or not (math.abs (y - e) <= math.abs (e) * 2 ^ -23) then return string.format ("%s: the host's answer to the lifted step is at y=%s, not at the lifted elevation %s", mode, tostring (y), tostring (e)) end
		end
		if plan.speed_zero and tgt.speed_delegations < 1 then
			return string.format ("UNREACHED: %s changed no step of the target (in the query frame the target was given %d steps, %d of mode move, %d with a finite positive desired_velocity, %d without): the host's own routed move of the target never carried a speed demand to replace", mode, tgt.window_steps, tgt.window_moves, tgt.speed_valid, tgt.speed_bad)
		end
		if profile == "original" then
			if after.entity ~= 0 or after.candidate ~= 0 or after.scan ~= 0 or after.roster ~= 0 or after.city ~= 0 or after.pose ~= 0 then return "profile=original, but the host was asked " .. after.entity .. " entity_collision, " .. after.candidate .. " entity_candidate, " .. after.scan .. " entity_scan, " .. after.roster .. " roster events, " .. after.city .. " city events and " .. after.pose .. " entity_pose questions: the module is not the original" end
			-- the original asks no hosted question, so only a control the caller bound is compared here (Root binds the Original control's signature)
			if plan.speed_zero and args.control_signature ~= nil and args.control_signature ~= "" and args.control_signature == table.concat (signature, ";") then
				return string.format ("UNCHANGED: %s: the original's physical result under the replaced speed demand is exactly the control's: the changed demand changed no output", mode)
			end
			return nil
		end
		if faults.request + faults.reply ~= 0 then return string.format ("the entity question or the host's reply is not the contract's (request_faults=%d reply_faults=%d): %s", faults.request, faults.reply, tostring (first_fault)) end
		if cand.mobile_unnamed ~= 0 then return string.format ("%s: %d mobile candidate questions carried no host identity", mode, cand.mobile_unnamed) end
		if after.scan < 1 then return string.format ("UNREACHED: %s: the production query asked the host no entity_scan question", mode) end
		if plan.speed_zero then
			-- B (speed demand): the SAME latched segments asked after the target's real move step was delegated ONCE with desired_velocity 0 in place of the engine's finite positive demand. This changes the demand the host's
			-- solver RECEIVED in a test-only configuration; it is not a claim that the host failed to fulfil an unchanged command. As for the lift, the broad phase may legitimately skip the target (no NeedUpdated or test
			-- is required), but the target must have been OFFERED and ANSWERED after the delegated step in the query phase, every request and reply must be the contract's, the engine must have completed (the C
			-- offer-conformance assertion stops it on a mirror that differs from the host's record) and the physical result must differ from the completed A's.
			if tgt.after_speed < 1 then return string.format ("UNREACHED: %s: no question about the target %s followed the changed-demand step (the target was not offered after it)", mode, target) end
			if args.control_signature == nil or args.control_signature == "" then
				return mode .. " needs control_signature=<the moving_query_speed_control run's query_signature>: a completed changed-demand query is not evidence until its physical result is compared with the control's"
			end
			if args.control_signature == table.concat (signature, ";") then
				return string.format ("UNCHANGED: %s: the changed-demand query's physical result is exactly the control's (after_speed=%d, after_speed_test=%d): the replaced speed demand changed no output", mode, tgt.after_speed, tgt.after_speed_test)
			end
			return nil
		end
		if plan.lift then
			-- B: the SAME latched segments asked after the real lifted host step. The shared broad phase (radius + 10 m, before the moving-vector read) may legitimately skip the lifted target without a
			-- NeedUpdated or a narrow-phase test, so none is required: what is required is that the target was actually OFFERED and ANSWERED after the lift in the query phase (`tgt.after_lift`: only
			-- armed questions about the target's own identity after the lift step are counted, never warmup or global counts), that every request and reply was the contract's (no fault above) and that
			-- the engine completed (the C offer-conformance assertion stops the engine on a mirror that differs from the host's record), and that the physical query result CHANGED from the control's.
			if tgt.after_lift < 1 then return string.format ("UNREACHED: %s: no question about the target %s followed the lifted step (the target was not offered after the lift)", mode, target) end
			if args.control_signature == nil or args.control_signature == "" then
				return mode .. " needs control_signature=<the moving_query_control run's query_signature>: a completed lifted query is not changed-pose authority until its physical result is compared with the control's"
			end
			if args.control_signature == table.concat (signature, ";") then
				return string.format ("UNCHANGED: %s: the lifted query's physical result is exactly the control's (after_lift=%d, after_lift_test=%d): the lift changed no output, so no changed-pose authority is claimed", mode, tgt.after_lift, tgt.after_lift_test)
			end
			return nil
		end
		-- A: the strict moving path within the controlled query phase, from the target's own questions (never a frame, warmup or global count)
		if tgt.need_updated < 1 then return string.format ("UNREACHED: %s: the target %s was never asked need_updated in %d questions (the shared C asks it only of a candidate whose motion vector is above 1), so it was not a moving candidate at the query", mode, target, tgt.questions) end
		if tgt.test < 1 then return string.format ("UNREACHED: %s: no question about the target %s reached the narrow phase (%d questions, %d skipped)", mode, target, tgt.questions, tgt.skip) end
		if tgt.test_moving16 < 1 then return string.format ("UNREACHED: %s: no test decision about the target %s carried the 16-word moving pose", mode, target) end
		return nil
	end
	local pair_list = {}
	for index, id in ipairs (ordered) do
		local s = alive [id]
		if s then
			local best, best_distance
			for _, other in ipairs (ordered) do
				local o = alive [other]
				if o and other ~= id and o.side ~= s.side and (o.side == "blue" or o.side == "red") and descriptors [other].kind ~= PERSON then
					local dx, dy, dz = o.position [1] - s.position [1], o.position [2] - s.position [2], o.position [3] - s.position [3]
					local distance = math.sqrt (dx * dx + dy * dy + dz * dz)
					if not best_distance or distance < best_distance then best, best_distance = other, distance end
				end
			end
			if best then pair_list [#pair_list + 1] = { launcher = id, target = best, distance = best_distance, order = index } end
		end
	end
	table.sort (pair_list, function (a, b)
		if a.distance ~= b.distance then return a.distance < b.distance end
		return a.order < b.order
	end)
	local launcher, target, weapon, distance
	local considered = #pair_list
	if plan.moving_target then
		-- the campaign's own engagements of the final warmup frame, in identity order (never by distance)
		considered = 0
		for _, id in ipairs (ordered) do
			local aimed = last_vectors [id]
			if alive [id] and aimed ~= nil and aimed.frame == frames - 1 then
				considered = considered + 1
				local o = alive [aimed.target]
				if o and descriptors [aimed.target] and o.side ~= alive [id].side and (o.side == "blue" or o.side == "red") and moving_routed (aimed.target)
					and aimed.weapon == math.floor (aimed.weapon) and aimed.weapon >= 1 and aimed.weapon <= 127 then
					local q = state (id, aimed.weapon)
					if q and q.weapon_count and q.weapon_count > 0 and q.guidance == 0 and q.aiming ~= 0 and q.decoy == 0 then
						launcher, target, weapon = id, aimed.target, aimed.weapon
						break
					end
				end
			end
		end
	else
		for _, pair in ipairs (pair_list) do
			for w = 1, 127 do
				local q = state (pair.launcher, w)
				if q and q.weapon_count and q.weapon_count > 0 and q.guidance == 0 and q.aiming ~= 0 and q.decoy == 0 then
					launcher, target, weapon, distance = pair.launcher, pair.target, w, pair.distance
					break
				end
			end
			if launcher then break end
		end
	end
	emit { event = "case_selected", launcher = launcher or "none", target = target or "none", weapon = weapon or -1, distance = distance or -1, pairs_considered = considered,
		selection = plan.moving_target and "campaign_engagement_of_a_moving_routed_enemy" or "nearest_non_person_enemy_of_a_launcher_with_an_unguided_ballistic_weapon" }
	if not launcher and plan.moving_target then hard ("UNREACHED: no campaign engagement of a moving routed enemy in the final warmup frame (" .. considered .. " launchers aimed a target that frame): the warmup, seed and distances are not tuned and no other target is taken") end
	if not launcher then hard ("SETUP FAILURE: no launcher with an unguided ballistic weapon and a non-person enemy was found") end

	watched = target
	if plan.moving_target then
		local last = last_move [target]
		local aimed = last_vectors [launcher]
		emit { event = "moving_target", launcher = launcher, target = target, weapon = weapon, motion = table.concat (last.motion, ","), frame = last.frame, position = table.concat (last.position or {}, ","),
			vectors_frame = aimed.frame, aim_target_motion = type (aimed.motion) == "table" and table.concat (aimed.motion, ",") or "", heading_evidence = "unavailable_aim_step_carries_no_weapon_identity" }
	end
	events () -- everything the natural war logged before the launch is taken first, so what is read after is the launch's own
	local any_tests_before = counter ("contact_any_tests")
	local any_hits_before = counter ("contact_any_hits")
	local precision_before = counter ("precision_high")
	armed = true
	local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
	local failure
	if not launched then
		if string.sub (first_line (launch_text), 1, #NAME + 1) == NAME .. ":" then error (tostring (launch_text), 0) end
		local marker = engine_marker (launch_text)
		if marker == nil then hard ("the launch call failed, but not with the engine's known fatal at the model boundary: " .. first_line (launch_text)) end
		failure = { marker = marker, line = first_line (launch_text) }
	end

	-- the launch report joined with the births the launch call itself logged: the cohort is frozen here and never added to
	local cohort, members, report = {}, {}, nil
	if launched then
		report = fields (launch_text)
		if string.sub (launch_text, 1, #"wpn-case-launch ") ~= "wpn-case-launch " then hard ("the launch report is not a wpn-case-launch line: " .. first_line (launch_text)) end
		for _, e in ipairs (events ()) do
			if e.kind == "birth" then
				if e.w == nil or tonumber (e.w) == nil then hard ("an event with no weapon identity: " .. e.raw) end
				if cohort [e.w] then hard ("a weapon born twice in one launch: " .. e.w) end
				cohort [e.w] = true
				members [#members + 1] = e.w
			end
		end
		table.sort (members, function (a, b) return tonumber (a) < tonumber (b) end)
		local problem = launch_report_problem (report, first_line (launch_text), weapon)
		if problem then hard (problem) end
		if plan.moving_target and (whole (report.target_link_before) ~= 1 or whole (report.restored) ~= -1) then
			hard (string.format ("the launch did not use the campaign's own engagement: target_link_before=%s restored=%s (expected 1 and -1, so no link was set): %s", tostring (report.target_link_before), tostring (report.restored), first_line (launch_text)))
		end
		local births = whole (report.births)
		if #members < 1 then hard ("SETUP FAILURE: the launch logged no birth: the controlled cohort is empty") end
		if births ~= #members then hard (string.format ("the launch report says births=%d but the launch logged %d birth events", births, #members)) end
	end
	emit { event = "case_launch", ok = launched and true or false, launcher = launcher, target = target, weapon = weapon, text = first_line (launch_text), births = #members }
	emit { event = "cohort", size = #members, members = table.concat (members, ",") }

	-- ------------------------------------------------------------------------------------------------------------ run until the cohort has ended, the engine stops, or the cap
	local terminal_of, terminals, foreign, duplicates, later_births = {}, {}, {}, {}, {}
	local function take ()
		for _, e in ipairs (events ()) do
			if e.kind == "birth" then
				later_births [#later_births + 1] = e.w or "?"
			elseif e.kind == "terminal" then
				local w = e.w
				if w == nil then hard ("a terminal fact with no weapon identity: " .. e.raw) end
				if not cohort [w] then foreign [#foreign + 1] = w
				elseif terminal_of [w] then duplicates [#duplicates + 1] = w
				else terminal_of [w] = e terminals [#terminals + 1] = e end
			end
		end
	end
	local function pending_members ()
		local n = 0
		for _, w in ipairs (members) do if not terminal_of [w] then n = n + 1 end end
		return n
	end
	local deadline = clock.elapsed_seconds + settle
	while not failure and pending_members () > 0 and clock.elapsed_seconds < deadline do
		failure = guarded (step, 1)
		-- after a stop NOTHING more is asked of the engine
		if failure then break end
		take ()
	end
	local any_tests_delta, any_hits_delta, precision_delta
	if not failure then
		take ()
		any_tests_delta = counter ("contact_any_tests") - any_tests_before
		any_hits_delta = counter ("contact_any_hits") - any_hits_before
		precision_delta = counter ("precision_high") - precision_before
	end

	-- the cohort's PHYSICAL signature: what each case weapon ended as and where, never its ordinal (an ordinal is bookkeeping and differs between runs for no physical reason)
	local signature_parts = {}
	for _, w in ipairs (members) do
		local e = terminal_of [w]
		if e then
			emit { event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime }
			signature_parts [#signature_parts + 1] = table.concat ({ tostring (e.type), tostring (e.code), tostring (e.steps), tostring (e.pos), tostring (e.velocity), tostring (e.lifetime) }, "|")
		end
	end
	for _, w in ipairs (foreign) do emit { event = "foreign_terminal", w = w } end
	for _, w in ipairs (duplicates) do emit { event = "duplicate_terminal", w = w } end
	for _, w in ipairs (later_births) do emit { event = "later_birth", w = w } end
	local incomplete = (not failure) and pending_members () > 0
	local pose_facts_row = pose_facts ()
	emit {
		event = "authority", mode = mode, profile = profile, expected_marker = plan.marker or "", completed = failure == nil and not incomplete, incomplete = incomplete,
		failure_marker = failure and failure.marker or "", failure_line = failure and failure.line or "",
		cohort_size = #members, case_weapons_ended = #terminals, case_weapons_pending = failure and -1 or pending_members (), later_births = #later_births, foreign_terminals = #foreign,
		duplicate_terminals = #duplicates, elapsed = clock.elapsed_seconds, frames = frames, frame_budget = frame_budget,
		contact_any_tests_delta = any_tests_delta or -1, contact_any_hits_delta = any_hits_delta or -1, precision_high_delta = precision_delta or -1, fixture_api = fixture_api,
		view_input = "configured_view_weapon_true_on_armed_weapon_steps", view_steps_seen = view.steps_seen, view_answers_seen = view.answers_seen, view_applied = view.applied,
		view_already_set = view.already_set,
		entity_before = before.entity, entity_after = after.entity, other_ops_before = before.other, other_ops_after = after.other,
		stationary = seen.stationary, moving = seen.moving, real_hit = seen.real_hit, real_miss = seen.real_miss,
		real_mask_63 = seen.real_mask_63, real_mask_56 = seen.real_mask_56, real_mask_other = seen.real_mask_other,
		answered_hit = seen.answered_hit, answered_miss = seen.answered_miss, altered = altered,
		candidate_before = before.candidate, candidate_after = after.candidate, candidate_scene0 = cand.scene0, candidate_mobile = cand.mobile,
		candidate_skip = cand.skip, candidate_need_updated = cand.need_updated, candidate_need_fixed = cand.need_fixed, candidate_need_placement = cand.need_placement,
		candidate_need_normal = cand.need_normal, candidate_buildings = cand.buildings, candidate_test = cand.test,
		candidate_altered = cand_altered, candidate_answered_skip = cand_skipped,
		roster_before = before.roster, roster_after = after.roster, roster_insert = scan.roster.insert, roster_remove = scan.roster.remove, roster_reset = scan.roster.reset,
		roster_placed = scan.roster.placed, roster_removed = scan.roster.removed, roster_cleared = scan.roster.cleared,
		scan_before = before.scan, scan_after = after.scan, scan_phase_begin = scan.phase_begin, scan_phase_entry = scan.phase_entry, scan_phase_end = scan.phase_end,
		scan_phase_resolved = scan.phase_resolved, scan_excluded = scan.excluded, scan_first = scan.first, scan_next = scan.next, scan_offer = scan.offer,
		scan_done_none = scan.done_none, scan_done_hit = scan.done_hit, scan_altered = scan_altered, scan_answered_done_none = scan_none_answered,
		pose_before = before.pose, pose_after = after.pose, pose_stage_offer = pose.stage_offer, pose_stage_placement = pose.stage_placement, pose_named = pose.named,
		pose_recorded = pose.recorded, pose_unrecorded = pose.unrecorded, pose_retired = pose.retired, pose_unknown = pose.unknown, pose_no_id = pose.no_id,
		pose_altered = pose_altered, pose_histogram = pose_histogram_text (pose_histogram),
		candidate_mobile_named = cand.mobile_named, candidate_mobile_unnamed = cand.mobile_unnamed, record_withdrawn = withdrawn, record_displaced = displaced,
			roster_withdrawn = scan.withdrawn, roster_withdrawn_next_phase = scan.withdrawn_next,
			city_before = before.city, city_after = after.city, city_child_insert = scan.city.child_insert, city_child_remove = scan.city.child_remove, city_child_first = scan.city.child_first,
			city_child_next = scan.city.child_next, city_link = scan.city.link, city_name = scan.city.name, city_placed = scan.city.placed, city_removed = scan.city.removed, city_next = scan.city.next,
			city_served = scan.city.served, city_withdrawn = scan.city.withdrawn, city_withdrawn_next = scan.city.withdrawn_next,
			city_withdrawn_cause = (failure and string.find (failure.line, "[roster.child]", 1, true) and string.find (failure.line, "has ended", 1, true)) and "city_child_ended" or "",
			roster_withdrawn_cause = (failure and string.find (failure.line, "[roster.key]", 1, true) and string.find (failure.line, "has ended", 1, true)) and "roster_key_ended" or "",
		target = watched or "", target_questions = tgt.questions, target_skip = tgt.skip, target_need_updated = tgt.need_updated, target_need_fixed = tgt.need_fixed,
		target_need_placement = tgt.need_placement, target_need_normal = tgt.need_normal, target_buildings = tgt.buildings, target_test = tgt.test,
		target_after_lift = tgt.after_lift, target_after_lift_test = tgt.after_lift_test,
		lifted = lifted, lift_refused = lift_refused, lift_constant = LIFT, lift_baseline_elevation = lift_row.baseline, lift_elevation = lift_row.elevation, lift_answered_y = lift_row.answered_y, lift_frame = lift_row.frame,
		pose_pre_total = pose_pre.total, pose_pre_stage_offer = pose_pre.stage_offer, pose_pre_stage_placement = pose_pre.stage_placement, pose_pre_named = pose_pre.named,
		pose_pre_recorded = pose_pre.recorded, pose_pre_unrecorded = pose_pre.unrecorded, pose_pre_retired = pose_pre.retired, pose_pre_unknown = pose_pre.unknown, pose_pre_no_id = pose_pre.no_id,
		pose_pre_histogram = pose_histogram_text (pose_pre_histogram),
		pose_fact_observations = pose_facts_row.observations, pose_fact_offers = pose_facts_row.offers, pose_fact_no_id = pose_facts_row.no_id, pose_fact_unrecorded = pose_facts_row.unrecorded, pose_fact_retired = pose_facts_row.retired,
		pose_fact_unknown = pose_facts_row.unknown, pose_fact_recorded = pose_facts_row.recorded, pose_fact_dead = pose_facts_row.dead, pose_fact_dead_recorded = pose_facts_row.dead_recorded,
		pose_fact_position_differs = pose_facts_row.position_differs, pose_fact_attitude_compared = pose_facts_row.attitude_compared, pose_fact_attitude_differs = pose_facts_row.attitude_differs,
		pose_fact_motion_held = pose_facts_row.motion_held, pose_fact_motion_held_differs = pose_facts_row.motion_held_differs, pose_fact_motion_not_held = pose_facts_row.motion_not_held,
		pose_fact_formula_equal = pose_facts_row.formula_equal, pose_fact_formula_differs = pose_facts_row.formula_differs,
		pose_fact_vehicle_formula_equal = pose_facts_row.vehicle_formula_equal, pose_fact_vehicle_formula_differs = pose_facts_row.vehicle_formula_differs,
		pose_fact_aircraft_motion_differs = pose_facts_row.aircraft_motion_differs, pose_fact_recorded_by_kind = pose_facts_row.recorded_by_kind,
		secondary_input = plan.paired and "entity_collision_answered_hit_mask_63_at_the_request_old_end_whenever_asked_in_both_paired_modes_configured_not_natural" or "",
		request_faults = faults.request, reply_faults = faults.reply, first_fault = first_fault or "",
		cohort_signature = table.concat (signature_parts, ";"),
		attribution = "global_to_the_war_no_per_weapon_identity",
		miss_coverage = (mode == "forced_miss") and "valid_accepted_miss_not_a_measured_physical_change_unless_the_control_had_hits" or "",
	}

	-- ------------------------------------------------------------------------------------------------------------ the verdict (raised after the evidence is closed by the caller)
	if plan.marker then
		-- a refusal mode: the engine must have stopped, at this marker, at an armed entity question the provider altered
		if plan.withdraw_child and scan.city.served < 1 then return "UNREACHED: " .. mode .. ": the real scan served no building of a city (" .. scan.city.child_first .. " first and " .. scan.city.child_next .. " next city questions), so there was no backing to withdraw: the case holds no city traversal" end
		if failure == nil then return "mode " .. mode .. " completed: the engine did not stop at " .. plan.marker end
		if failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [model-provider:" .. failure.marker .. "], not at [model-provider:" .. plan.marker .. "]: " .. failure.line end
		if altered + cand_altered + scan_altered + pose_altered + withdrawn + displaced + math.max (scan.withdrawn, 0) + math.max (scan.city.withdrawn, 0) < 1 then return "mode " .. mode .. " stopped without the provider having altered an entity answer: " .. failure.line end
		if view.applied < 1 then return "mode " .. mode .. " stopped, but the controlled view input never reached a weapon step: the stop is not at an entity question of the configured flight" end
		if mode == "withdrawn_child" then
			-- credited only by the reached withdrawal (acknowledged `removed`), the very next city question being the traversal's next building, and the host's own named refusal of the ended building as the only fault
			if scan.city.withdrawn ~= 1 then return "mode withdrawn_child stopped at its marker, but the host's removal of the served building was not acknowledged as removed (" .. scan.city.withdrawn .. ")" end
			if scan.city.withdrawn_next ~= "child_next" then return "mode withdrawn_child stopped at its marker, but the city question after the removal was not the traversal's next building (" .. scan.city.withdrawn_next .. ")" end
			if not (string.find (failure.line, "[roster.child]", 1, true) and string.find (failure.line, "has ended", 1, true)) then return "mode withdrawn_child stopped at its marker, but not for the ended city child: " .. failure.line end
			if faults.request ~= 0 or faults.reply ~= 1 or first_fault ~= "scan reply: result refused" then return "mode withdrawn_child had a fault other than the host's named refusal (request_faults=" .. faults.request .. " reply_faults=" .. faults.reply .. "): " .. tostring (first_fault) end
		end
		if mode == "withdrawn_backing" then
			-- credited only by the reached withdrawal (acknowledged `removed`), the very next question being the entry continuation, and the host's own named refusal of the ended key as the only fault
			if scan.withdrawn ~= 1 then return "mode withdrawn_backing stopped at its marker, but the host's removal of the served entry was not acknowledged as removed (" .. scan.withdrawn .. ")" end
			if scan.withdrawn_next ~= "phase_entry" then return "mode withdrawn_backing stopped at its marker, but the question after the removal was not the entry continuation (" .. scan.withdrawn_next .. ")" end
			if not (string.find (failure.line, "[roster.key]", 1, true) and string.find (failure.line, "has ended", 1, true)) then return "mode withdrawn_backing stopped at its marker, but not for the ended roster entry: " .. failure.line end
			if faults.request ~= 0 or faults.reply ~= 1 or first_fault ~= "scan reply: result refused" then return "mode withdrawn_backing had a fault other than the host's named refusal (request_faults=" .. faults.request .. " reply_faults=" .. faults.reply .. "): " .. tostring (first_fault) end
		end
		if mode == "withdrawn_record" and not string.find (failure.line, "retired", 1, true) then return "mode withdrawn_record stopped at its marker, but not for the retired identity: " .. failure.line end
		if mode == "displaced_record" and not string.find (failure.line, "differ from the host", 1, true) then return "mode displaced_record stopped at its marker, but not for a mirror that differs from the host's record: " .. failure.line end
		return nil
	end
	if failure ~= nil then return "mode " .. mode .. " failed in the engine at [model-provider:" .. failure.marker .. "]: " .. failure.line end
	if incomplete then
		return string.format ("INCOMPLETE: %d of %d case weapons had no terminal by the settle cap of %g s; an unfinished cohort is never a pass", pending_members (), #members, settle)
	end
	if #duplicates > 0 then return "a case weapon ended twice (" .. #duplicates .. " duplicate terminals)" end
	for _, w in ipairs (members) do
		local e = terminal_of [w]
		local problem = terminal_problem (e)
		if problem then return "the terminal of case weapon " .. w .. " is incomplete (field " .. problem .. " is missing or not in its grammar): " .. e.raw end
	end
	-- the controlled input must have reached the host's weapon step and the step must have taken the high-precision path it opens; otherwise the counters below say nothing
	if view.applied < 1 then return string.format ("UNREACHED: the controlled view input never reached a weapon step (%d weapon steps seen, %d VIEW_WEAPON answers): the flight was not under the configured precondition", view.steps_seen, view.answers_seen) end
	if precision_delta < 1 then return "UNREACHED: the controlled view input was applied but the weapon step never took the high-precision collision path (precision_high did not rise)" end
	-- the ACTUAL caller was reached: the weapon step's entity lookup was made (the fixture's own counter, not the provider's)
	if any_tests_delta < 1 then return "UNREACHED: the weapon step made no entity-collision test (contact_any_tests did not rise): the actual entity caller was not reached" end
	if plan.lift then
		if lifted < 1 then return string.format ("UNREACHED: terrain_lift lifted no step of the target (%d refused by the map ceiling): the host's own routed move of the target never carried a terrain input within the bound", lift_refused) end
		-- the host's physical step answered AT the lifted elevation (eech_convoy_step sets the new y from it): to one single-precision step of it
		local y, e = lift_row.answered_y, lift_row.elevation
		if not finite (y) or not (math.abs (y - e) <= math.abs (e) * 2 ^ -23) then return string.format ("terrain_lift: the host's answer to the lifted step is at y=%s, not at the lifted elevation %s", tostring (y), tostring (e)) end
	end
	if profile == "original" then
		if after.entity ~= 0 or after.candidate ~= 0 or after.scan ~= 0 or after.roster ~= 0 or after.city ~= 0 or after.pose ~= 0 then return "profile=original, but the host was asked " .. after.entity .. " entity_collision, " .. after.candidate .. " entity_candidate, " .. after.scan .. " entity_scan, " .. after.roster .. " roster events, " .. after.city .. " city events and " .. after.pose .. " entity_pose questions: the module is not the original" end
		return "CAPABILITY RED (expected for the original profile): the entity-collision caller ran (contact_any_tests +" .. any_tests_delta .. ", contact_any_hits +" .. any_hits_delta .. ", fixture API present) and the host was asked no entity_collision question, no entity_candidate question and no entity_scan question"
	end
	-- the route is the host's: a candidate that passed the (campaign) broad phase asks the host. None at all is a setup that never reached the narrow phase
	if after.scan < 1 then return string.format ("UNREACHED: the weapon step made %d entity-collision tests but the host was asked no entity_scan question, so no candidate reached the narrow phase and the host was asked no entity_candidate question: the real enumeration did not reach the host", any_tests_delta) end
	if after.candidate < 1 and not plan.no_downstream then return string.format ("UNREACHED: the weapon step made %d entity-collision tests but the host was asked no entity_candidate question, so no candidate reached the narrow phase: the real scan did not reach the host", any_tests_delta) end
	if after.entity < 1 and not plan.skips and not plan.no_downstream then return string.format ("UNREACHED: the weapon step made %d entity-collision tests but no candidate reached the narrow phase, so the host was asked no entity_collision question", any_tests_delta) end
	if faults.request + faults.reply ~= 0 then return string.format ("the entity question or the host's reply is not the contract's (request_faults=%d reply_faults=%d): %s", faults.request, faults.reply, tostring (first_fault)) end
	if plan.moving_target then
		if cand.mobile_unnamed ~= 0 then return string.format ("%s: %d mobile candidate questions carried no host identity", mode, cand.mobile_unnamed) end
		if tgt.need_updated < 1 then return string.format ("UNREACHED: %s: the target %s was never asked need_updated in %d questions (the shared C asks it only of a candidate whose motion vector is above 1), so it was not a moving candidate at the question", mode, tostring (watched), tgt.questions) end
		if tgt.test < 1 then return string.format ("UNREACHED: %s: no question about the target %s reached the narrow phase (%d questions, %d skipped)", mode, tostring (watched), tgt.questions, tgt.skip) end
		if plan.lift and tgt.after_lift < 1 then return string.format ("UNREACHED: terrain_lift: the cohort ended before a lifted step (frame %d) was followed by a question about the target %s", lift_row.frame, tostring (watched)) end
	end
	if seen.stationary + seen.moving ~= after.entity then return string.format ("the provider counted %d entity questions but classified %d", after.entity, seen.stationary + seen.moving) end
	if cand.total ~= after.candidate then return string.format ("the provider counted %d candidate questions but classified %d", after.candidate, cand.total) end
	if scan.total ~= after.scan then return string.format ("the provider counted %d scan questions but classified %d", after.scan, scan.total) end
	if scan.city.total ~= after.city then return string.format ("the provider counted %d city events but classified %d", after.city, scan.city.total) end
	if scan.roster.total ~= after.roster then return string.format ("the provider counted %d roster events but classified %d", after.roster, scan.roster.total) end
	if pose.total ~= after.pose then return string.format ("the provider counted %d pose questions but classified %d", after.pose, pose.total) end
	if pose_pre.total ~= before.pose then return string.format ("the provider counted %d pre-launch pose questions but classified %d", before.pose, pose_pre.total) end
	if mode == "control" or plan.host_side_movement then
		-- the host-side movement modes are the natural chain too: they alter the host's own movement input, never an answer of the provider
		if altered ~= 0 or cand_altered ~= 0 or scan_altered ~= 0 then return "the control altered an answer" end
		-- the natural chain: the enumeration offered candidates and a candidate reached the narrow phase
		if scan.offer < 1 then return "UNREACHED: the real enumeration offered no entry (" .. scan.offer .. " offers), so no candidate dialogue followed from it" end
		if cand.mobile_unnamed > 0 then return string.format ("%d mobile candidate questions carried no host identity: the host holds no record it can decide their pose from", cand.mobile_unnamed) end
		-- the observation is characterisation, not an expectation: it must have been REACHED (a mobile candidate was offered), and every question and reply was well formed (checked above)
		if pose.total + pose_pre.total < 1 then return "UNREACHED: the real enumeration offered " .. scan.offer .. " entries but no mobile candidate was offered, before or after the launch, so the host was asked no entity_pose question and the mobile collaborator contract was not observed" end
		-- every weapon-step entity hit is one narrow-phase hit of the host (the hosted line of sight asks too, so the host may report more)
		if seen.real_hit < any_hits_delta then return string.format ("the weapon step counted %d entity hits but the host's narrow phase reported only %d", any_hits_delta, seen.real_hit) end
		return nil
	end
	if mode == "scan_real" then
		-- the paired control: the real enumeration must have offered entries, the candidate dialogue and the narrow phase must have followed, and the configured hit must have been used by the actual caller
		if scan_altered ~= 0 or cand_altered ~= 0 then return "scan_real altered an answer of the enumeration or of a candidate" end
		if scan.offer < 1 or cand.test < 1 or after.entity < 1 then return "UNREACHED: the real enumeration never led to a narrow phase (" .. scan.offer .. " offers, " .. cand.test .. " test decisions, " .. after.entity .. " entity questions), so the paired variant has nothing to prevent" end
		if altered ~= after.entity or seen.answered_hit ~= after.entity or seen.answered_miss ~= 0 then return string.format ("scan_real answered %d hits and %d misses of %d entity questions", seen.answered_hit, seen.answered_miss, after.entity) end
		if any_hits_delta < 1 then return "scan_real: the configured hit never reached the weapon step's entity hit count (contact_any_hits did not rise): the narrow phase's answer was not used by the actual caller" end
		return nil
	elseif mode == "scan_none" then
		-- the variant: every scan was a valid empty enumeration, so no candidate dialogue, no narrow phase and no entity hit may follow
		if scan_altered ~= after.scan or scan_none_answered ~= after.scan then return string.format ("scan_none answered %d empty enumerations of %d scan questions", scan_none_answered, after.scan) end
		if after.candidate ~= 0 or after.entity ~= 0 or after.pose ~= 0 or altered ~= 0 or cand_altered ~= 0 then return string.format ("scan_none: the host was still asked %d entity_candidate, %d entity_collision and %d entity_pose questions after every enumeration was empty", after.candidate, after.entity, after.pose) end
		if any_hits_delta ~= 0 then return string.format ("scan_none: the weapon step still counted %d entity hits although every enumeration was empty", any_hits_delta) end
		return nil
	end
	if mode == "candidate_real" then
		-- the paired control: the real scan must have reached the narrow phase (a test decision) and the configured hit must have been used by the actual caller
		if cand_altered ~= 0 then return "candidate_real altered a candidate answer" end
		if cand.test < 1 or after.entity < 1 then return "UNREACHED: the real candidate decisions never required a narrow phase (" .. cand.test .. " test decisions, " .. after.entity .. " entity questions), so the paired variant has nothing to prevent" end
		if altered ~= after.entity or seen.answered_hit ~= after.entity or seen.answered_miss ~= 0 then return string.format ("candidate_real answered %d hits and %d misses of %d entity questions", seen.answered_hit, seen.answered_miss, after.entity) end
		if any_hits_delta < 1 then return "candidate_real: the configured hit never reached the weapon step's entity hit count (contact_any_hits did not rise): the narrow phase's answer was not used by the actual caller" end
		return nil
	elseif mode == "candidate_skip" then
		-- the variant: every candidate decision was a valid skip, so no narrow phase may follow and no entity hit may reach the weapon step
		if cand_altered ~= after.candidate or cand_skipped ~= after.candidate then return string.format ("candidate_skip answered %d skips of %d candidate questions", cand_skipped, after.candidate) end
		if after.entity ~= 0 or altered ~= 0 then return string.format ("candidate_skip: the host was still asked %d entity_collision questions after every candidate was skipped", after.entity) end
		if any_hits_delta ~= 0 then return string.format ("candidate_skip: the weapon step still counted %d entity hits although every candidate was skipped", any_hits_delta) end
		return nil
	end
	if altered < 1 then return "mode " .. mode .. " altered no entity answer" end
	if mode == "forced_hit" then
		if seen.answered_hit ~= after.entity or seen.answered_miss ~= 0 then return string.format ("forced_hit answered %d hits and %d misses of %d questions", seen.answered_hit, seen.answered_miss, after.entity) end
		if any_hits_delta < 1 then return "forced_hit: the forced hit never reached the weapon step's entity hit count (contact_any_hits did not rise): the answer was not used by the actual caller" end
	elseif mode == "forced_miss" then
		if seen.answered_miss ~= after.entity or seen.answered_hit ~= 0 then return string.format ("forced_miss answered %d misses and %d hits of %d questions", seen.answered_miss, seen.answered_hit, after.entity) end
		if any_hits_delta ~= 0 then return string.format ("forced_miss: the weapon step still counted %d entity hits, but the host answered every question as a miss", any_hits_delta) end
	end
	return nil
end

-- main runs under xpcall so that the evidence is closed on EVERY path; the first error is the one raised, and a close that fails is never hidden and never follows a success
local ran, verdict = xpcall (main, function (e) return e end)
local closed, close_error = out:close ()
if not ran then
	if not closed then io.stderr:write (NAME .. ": OUTPUT FAILURE while closing after the failure below: " .. tostring (close_error) .. "\n") end
	error (verdict, 0)
end
if not closed then error (NAME .. ": OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
if verdict ~= nil then error (NAME .. ": " .. verdict, 0) end
io.stderr:write (string.format ("%s: mode=%s profile=%s completed entity=%d other_ops=%d altered=%d\n", NAME, mode, profile, after.entity, after.other, altered))
