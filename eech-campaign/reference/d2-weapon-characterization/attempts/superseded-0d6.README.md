# SUPERSEDED / INCOMPLETE: first fixture build 0d6fa8a94 (27 Oct 2026)

Not an accepted characterization and not a baseline. The independent review found fixture defects (WEAPON-LEAD-FEEDBACK.md revision 1) in this source
before any trace was accepted. Preserved as evidence of what actually ran:

* `out/smoke-1`: 30 s run, driver exit 0 (301 frames: the first clock reaching 30 s). Complete but short; shows boot-time table facts only.
* `out/l600-a`: Lebanon 600 s, native PID 18876, INTENTIONALLY STOPPED by root at the review checkpoint (trace reaches checkpoint 300 s; no `run` event). Not a crash.
* `out/l600-b`: started by the private queue after l600-a's script exit 1; INTENTIONALLY STOPPED by root (trace holds the boot report only). Not a crash.
* `bin/`: the exported fixture binaries of 0d6fa8a94 (BUILD-INFO there); the build log, static checks and the strict feature-off proof of that source are beside it.
* `roots/`: the private roots those runs wrote into (not reused).
Root's own hash records: ../superseded-0d6-l600-{a,b}-partial-hashes.json.
