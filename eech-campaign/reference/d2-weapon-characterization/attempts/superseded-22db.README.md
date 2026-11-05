# SUPERSEDED original-observer build 22db40e5b (27 Oct 2026) — not the final original characterization

Source/fixture review-1 corrections were confirmed sound (decoy per weapon, empty-link setter/restore, launcher/query guards). Two blockers remained (revisions 3-5): the runner's single-backslash
regex (found by the w2-case-a attempt, `../failed-setup`, no native started) and the overbroad shared `supported_kind` that refused a SITE as a TARGET. Preserved:
* `out/w2-case-b` + `roots/w2-case-b`: controlled witness, native PID 25016, ran to completion, actual script/native exit **1** after 422 s because ONE check failed
  (`case_weapons_reach_terminal`: the case's smoke grenade rests on the ground; see the final witness). Its `rejects_unsupported_target` check PASSED on a refusal that was the false expectation (a SITE is a valid target).
  Its trace also holds JSON-invalid `check` lines (Lua 5.1 `%q` writes a newline as backslash-newline): an observer-driver defect, corrected afterwards.
* `bin/`: the exported fixture binaries (all 32 BUILD-INFO manifest entries independently matched by root); `build-fixture-*.log`, static checks (fmt exit 1 on v2: one-line format,
  fixed before the commit; fmt exit 0 on v3), the strict feature-off proof (all four sources match, feature-on differs) of this source.
