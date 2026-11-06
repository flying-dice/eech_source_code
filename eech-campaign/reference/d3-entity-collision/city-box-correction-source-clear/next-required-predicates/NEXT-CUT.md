# Next required fixed predicate cut: committed source facts

This is a read-only handoff for the sole next implementation lane, not an approved design or a second implementation. Source identities and the fifteen declared fixed subtypes are in SOURCE-FACTS.json. Facts are pinned to e49; constructor edits were not inspected as final source.

The physical responsibility still in Core is fx_int.c:238/254, deriving align_with_terrain and collision_test_fixed from fixed_database[raw->sub_type]. The canonical host reporter mb_hostplace.c:84/86 currently takes these values from those Core getters. Moving only their storage did not move this derivation. Trace initial creation, subtype changes, restoration and re-placement before choosing the handoff; preserve current semantic-ID timing and one state owner.

The database also contains default shape and ground-radar clutter. Establish their active consumers separately; do not move the whole mixed table merely to move two physical columns. Core constructors read default shape; renderer draw files read align directly, but a lexical match does not prove supported headless runtime reachability. No caller was declared dormant here.

INT_TYPE_IDENTIFY_FIXED is a separate constant classification in fx_int.c:310. Campaign engage.c:126 uses it in task decisions, while sc_msgs.c:128 uses it in the still-required tallest-structure physical computation. Preserve campaign classification and consequences; move the needed physical computation. Keysite ks_int.c:253 derives the identically named ALIGN_WITH_TERRAIN enum from a different database. Do not conflate domains by enum spelling.

For the eventual implementation, prove current actual family/type authority and supported subtype bounds, complete lifecycle/restore behavior, no query-time Core adoption, named unknown/refusal, and that missing/changed host physical input is used through the public native path. Characterization must include original FP/RNG/order and immediate consumers as applicable. Reuse closed constructor/backing evidence and use the same independent reviewer. These notes neither remove another D3 requirement nor authorize D4/D5.

Reviewer precision: ENTITY_SUB_TYPE_FIXED_UNKNOWN is itself a declared valid fixed subtype. Any future unknown refusal means an actually unsupported or out-of-domain value, never that legitimate enum simply because its name is UNKNOWN.
