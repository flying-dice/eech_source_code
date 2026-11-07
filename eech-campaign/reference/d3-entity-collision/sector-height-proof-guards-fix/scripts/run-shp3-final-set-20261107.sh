#!/bin/sh
# the FINAL set of AFFECTED checks of the proof-guards correction (two Lua diagnostics and their driver tests), one child at a time, between its PRE and POST input maps (no source write in between); the 0be1dd846
# matrices and the 5aae2256f u1-u11 are reused ONLY through the explicit join. No C, x87, legacy workspace or whole-World rerun is made for these Lua/driver guards.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-sector-height-production-direct-20261107.py
P=shp3-final-set-20261107.progress
run() { "$PY" -I -B -X utf8 "$1" "$2" "$3" > /dev/null 2>&1; echo "$2 exit=$?" >> $P; }
: > $P
run $D v1-fmt-check 'cargo fmt --check'
run $D v2-clippy-world 'cargo clippy --locked -p eech-world --all-targets -- -D warnings'
run $D v3-driver-tests 'cargo test --locked -p eech-world --test entity_authority --test sector_height_baseline_driver --no-fail-fast'
run $D v4-host-callbacks 'cargo test --locked -p eech-world --test host_callbacks --no-fail-fast'
# the REAL accepted original report (private, outside Git, immutable) against the same rules as the Lua validation (an independent port); only safe aggregates are printed
ACC="D:/EECH-Migration-Evidence/sector-height-capacity-f01-baseline-20261107/sector-height-lebanon_retail-seed1.txt"
"$PY" -I -B -X utf8 check-real-accepted-shape-20261107.py "$ACC" > shp3-real-accepted-shape-20261107.txt 2> shp3-real-accepted-shape-20261107.err.txt
echo "v5-real-accepted-shape exit=$?" >> $P
sha256sum "$ACC" > shp3-real-accepted-sha-20261107.txt
echo DONE >> $P
