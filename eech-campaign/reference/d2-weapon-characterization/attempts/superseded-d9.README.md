# d9d6930b7 controlled witness — actual exit 1 (retained as evidence, not accepted)

`out/w3-case-a`: native PID 19580, queue run-queue.ps1, completed 21:11:39 UTC, script/native exit **1** after 419 s; 11 checks, 1 failed: `case_weapons_reach_terminal`.
The unmatched case weapon is ordinal **117 CRATE** (launched by a fixed-wing at the case's `unguided_other` step): wpn-live at 301.5754 s: steps 2415, on_ground 0, lifetime -241.487854,
cruise_time 511, cruise_error 0, expires_in_at_least 269.512146, terminal_seen 0 — still IN FLIGHT inside its own lifetime. weapon_movement ends it only at lifetime <= -(cruise_time + error),
so the 240 s wait was shorter than the weapon's own end. The d9 "resting smoke grenade" assumption was wrong for this weapon (run-wide ground_survived 1 described another object).
All other 10 checks passed, including accepts_site_target (a SITE target: link set, weapon born with target ENTITY_TYPE_SITE, previous link restored) and the three empty-link checks.
Corrected afterwards in the Lua witness (3c7b5ef40): the wait is computed from each case weapon's own lifetime and cruise time; the check stays strict.
