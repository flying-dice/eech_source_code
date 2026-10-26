# D2 LOS characterization checkpoint — 26 October 2026

**EEDCS-42 and D2 remain incomplete. No physical solver has moved. No PR or
formal approval exists. No baseline or tolerance was changed.**

The sole Claude Sonnet 5.5 High implementation lane prepared the feature-gated
characterization harness at `a0b6246468a2c10932eccbde0a7eeeda4cbecf56`.
The primary Codex lead performed one independent source review for correctness,
clean code and boundary safety. The author corrected its findings at
`676ab794c024cce0644c2ed7f3a3cd43aedba6b8`; the lead inspected that correction
diff, without repeating the full review or starting additional reviewers.

## Findings and correction inspection

| Finding | Source correction verified |
|---|---|
| The fixture bounded entity indices by MAX_NUM_ENTITIES, but en_heap.c allocates only number_of_entities entries. A Lua index between those values could read outside the heap. | live_entity now checks a non-null heap and the actual allocated length before forming a pointer, and permits only the six supported mobile families. Runtime negative cases remain unrun. |
| The Lua driver could emit an invariant failure or take no requested probes yet exit successfully. | It now asserts the trace invariant, emits and rejects unavailable pairs, and rejects unreached probe times. An explicitly empty probe_at remains the no-probe control. Runtime refusal cases remain unrun. |
| The source notes overstated criteria/threshold coverage and misidentified aircraft criteria. | Notes now identify eight chosen criteria masks; only 200 m thresholds are bracketed. Fine loops make at most seven queries. Vehicle and aircraft callers use ALL for non-AI and COARSE for AI targets; wn_tgt always requests COARSE. |

The unchanged LOS solver arithmetic remains in mb_tgt.c; the gated shim only
records input float bits, criteria and results. Feature-disabled runtime paths
are unmodified by this preparation. The test surface carries entity indices
within the existing fixture model and copied counts/digests; no legacy entity
pointer enters Lua. The 64-bit digest is transported as a hexadecimal string.
This source inspection does not prove observational neutrality or runtime safety.

## Actual checks

- `git diff --check ff285dcab..676ab794c`: no whitespace errors.
- Parsed `crates/eech-world/lua/los-characterization.lua` with the copied bundle's
  `bin/lua.dll`, using `luaL_newstate` and `luaL_loadfile` through Python 3.12
  ctypes, then closing the Lua state. **PASS Lua 5.1 syntax only.** The loaded
  chunk was never executed. The local helper is outside the repository at
  `../check-lua-syntax.py`.
- The existing Docker Desktop Linux daemon remains unavailable while its
  first-run Subscription Service Agreement waits for user handling. No C/Rust
  compilation, formatting/lint execution, runtime characterization, boundary
  execution test, shipping regression pack or D2 Tacview comparison has run.
- Both Claude print-mode processes exited successfully; the actual local
  context transcript was verified before the correction resume. No process was
  restarted after an assumed timeout. Relevant context:
  `ae34a3d7-0931-4638-b06a-a2d399277929`.

The original implementation and correction JSONL logs and prompts are retained
outside the repository in the parent workspace. There were no build caches to
clean. The local regression bundle remains intact, outside Git. Its verified
payload and manifest evidence is in HOST-READINESS-20261025.md. Accepted D1
runtime evidence remains reused at its documented limits.

## Next action and gates

When the existing Docker daemon is available, build the characterization
checkpoint using the repository Dockerfile and Windows cross-build recipe,
with `--features eech-dc/los-fixture` on the cargo build command and a fresh
task target volume. The shipping build script does not currently expose a
features argument; invoke its documented Docker recipe with that explicit
test-only flag rather than enabling the feature in a shipping build.

Run the prepared Lua driver through the real module/host path on the copied
Lebanon and Georgia roots, with probes and the explicit no-probe control.
Record baseline outputs, tested criteria/paths and observational limits; exercise
the invalid-index, unsupported-family, free-slot, same-entity, unavailable-pair
and unreached-time refusals. Fix any harness compile/runtime defects before
moving the solver. Only then implement the bounded host extraction and prove
host observations influence campaign decisions, run the established shipping
pack/full-file comparison, complete independent delivery review and normal merge.

Normal PR publication also needs a distinct authorized author connection:
GitHub CLI and connected GitHub both identify fd-starscream-bot, the mandatory
reviewer. AGENT.md requires a different PR author. No authentication was changed,
no credentials were copied, and no self-approval workaround was attempted.
The questions about the Docker agreement and existing author path remain pending.

Terrain/object queries, entity mirrors, shared RNG, effects and sounds remain
declared coupling. Remaining D2 sensing/combat work is not delivered; D3 has not
started. This checkpoint is preparation and preserved partial work, not D2
acceptance or evidence of unchanged candidate Tacview.
