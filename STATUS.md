# D3 ground-contact on G1 — private test-only composition checkpoint

29 October 2026. This is an ISOLATED composition checkout, not a delivery branch. Authority: M5 goal 11534337 v3, migration ledger 14548994.
D3/EEDCS-21 is In Progress; D4/D5 have not started. Campaign geography, routes, timers, scheduling and consequences stay in EECH_DC. Other D3 spatial, model,
environment and host-time groups remain in scope. The finite D3 scope is docs/handover/D3-SCOPE-AUDIT.md.

## What is composed, and what is not

- Parent G1 (Task47 / EEDCS-47, PR107, branch codex/d3-terrain-services) at pinned f03a680c9625b89ac966322faf0dd96460a3e03d. **G1 is NOT accepted or delivered.**
  Its frozen immutable-06 ordinary shipping pack is a separate live run and is not touched here. Point, fast-elevation and map-fact queries dispatch to the host provider; the
  DLL keeps semantic value records and optional opaque hints. The sector-maximum getter and the physical terrain sector grid are still the campaign's own (G2 has not moved).
- Task48 / EEDCS-48 original characterization at pinned d44a19612d014fa25cddb8e455dbe145f3549e41 (original fixture source from c119a7370, plus the strict JSON encoder and
  comment-only wording). It is merged normally into the G1 head in this isolated checkout. Conflicts were only STATUS.md and
  eech-campaign/crates/eech-dc/src/module.rs, resolved additively: the terrain-fixture methods and the ground_probe method are both kept.
- **This lane is test-only.** It adapts the ORIGINAL ground fixture so the same original queries (point_above_ground, point_below_ground, get_ground_impact_point), the same
  case generator (including the physical terrain grid), seed and draw order, and the same `ground-physical` fold can run against G1's host-terrain module. It does not move G2,
  does not cut over production, adds no host API and changes no ordinary shipping source. The original query expressions (en_world.h, en_cllsn.c) and terrdata.c are byte-identical to the base.

## Original evidence (preserved, never rewritten)

- Oracles: R:/d3-evidence/ground-original-c119a7370-{georgia,lebanon,lebanon-repeat}/ground.jsonl (fixture build c119a7370, 43 entries verified). Georgia, Lebanon and a separate
  Lebanon repeat each ran native exit 0 with 20,000 point cases and 5,000 impacts, twice, campaign seed 1, private seed 20261029; 117 rows per file pass strict JSON; reports are
  in-process identical; the separate Lebanon raw files are byte identical; all three post-input checks pass. These are the ORIGINAL profile (legacy record diagnostics).
- The first 9e607637f runs were refused before any case (campaign grid conflated with the physical grid) and their raw failures stay preserved.
- Original probe/no-probe control (300 s before, 60 s after, 100 ms frames, two verified roots): both exited 0 with 3600 frames, 1726 objects, 1712 alive, digest 2c4dce1c and
  elapsed 359.96115112305, identical except the probed flag; strict JSON 117 and 3 rows, post-inputs 0. The digest is coarse; it is not a full Tacview or all-state proof.
- Source-level Linux evidence for the original driver (11 real-Lua tests, Clippy 0, formatting and Lua parses 0) is from the original branch, not from this composition.

## Composition validation and remaining gates

1. Root source/driver checks PASS: all40 real Linux host_callbacks guards (not the shipping pack), affected Clippy0, fmt/check0, both Lua5.1 parses0 and diffcheck0. Exact tested hashes retained. Selecting the four non-host branches reproduces the original C token stream; original query/extent files are unchanged. This does not prove hosted physical equality.
2. Root: commit the clean composition; independent review of the new adaptation and completed original-control evidence is pending.
3. Root: clean fixture build of the composition, then the SAME 20k points / 5k impacts / 2 repeats probes on fresh manifest-verified roots through the real G1 module and
   `host.terrain_provider`, and exact comparison against the original oracles: `ground-physical`, the counters and the samples' physical words are the common comparison; the diagnostic
   rows are NOT comparable and must be classified, never normalized. Real mismatches are investigated from raw cases. Cache, short-circuit and restoration measurements are compared
   separately with their limits; differences are preserved.
4. After G1 delivery and integration re-verification: G2 production extraction (not started), unchanged shipping pack, authority and refusal proof, normal review and delivery.

No full 40-check, Tacview, Task47 or epic acceptance is claimed. Details: eech-campaign/docs/m5-ground-characterization.md (section 10).
