#!/bin/sh
# the FINAL set of AFFECTED checks of the public-proof-plan correction, one runner child at a time, between its PRE and POST input maps (no source write in between); the unchanged 0be1dd846 matrices are reused by the explicit join
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-sector-height-production-direct-20261107.py
I=run-sector-height-production-i686-20261107.py
P=shp2-final-set-20261107.progress
run() { "$PY" -I -B -X utf8 "$1" "$2" "$3" > /dev/null 2>&1; echo "$2 exit=$?" >> $P; }
: > $P
run $D u1-fmt-check 'cargo fmt --check'
run $D u2-clippy-workspace 'cargo clippy --locked --all-targets -- -D warnings'
run $D u3-clippy-sys-hosted 'cargo clippy --locked -p eech-engine-sys --features sector-height-check,sector-height-observer --all-targets -- -D warnings'
run $D u4-clippy-sys-legacy 'cargo clippy --locked -p eech-engine-sys --features sector-height-check,sector-height-observer,entity-dialogue-check,collision-fixture --all-targets -- -D warnings'
run $D u5-world-tests 'cargo test --locked -p eech-world --no-fail-fast'
run $D u6-control-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture'
run $D u7-hosted-clearance 'cargo test --locked -p eech-engine-sys --features sector-height-check,sector-height-observer --test sector_height_hosted_clearance'
run $D u8-observer-control-legacy 'cargo test --locked -p eech-engine-sys --features sector-height-check,sector-height-observer,collision-fixture --test sector_height_observer --test sector_height'
run $D u9-observer-chain-engine 'cargo test --locked -p eech-engine --features sector-height-funnel-check --test sector_height_funnel --test sector_height_observer_chain --test sector_height_observer_cap'
run $I u10-hosted-i686 'cargo test --locked --target i686-unknown-linux-gnu -p eech-engine-sys --features sector-height-check,sector-height-observer --test sector_height --test sector_height_hosted_clearance'
run $I u11-observer-legacy-i686 'cargo test --locked --target i686-unknown-linux-gnu -p eech-engine-sys --features sector-height-check,sector-height-observer,collision-fixture --test sector_height_observer --test sector_height'
echo DONE >> $P
