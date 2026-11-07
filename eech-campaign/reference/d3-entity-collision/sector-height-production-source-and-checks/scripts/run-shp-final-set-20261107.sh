#!/bin/sh
# the FINAL set of affected checks, one runner child at a time, between the PRE and POST input maps (no source write in between)
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-sector-height-production-direct-20261107.py
I=run-sector-height-production-i686-20261107.py
run() { "$PY" -I -B -X utf8 "$1" "$2" "$3" > /dev/null 2>&1; echo "$2 exit=$?" >> shp-final-set-20261107.progress; }
: > shp-final-set-20261107.progress
run $D t1-fmt-check 'cargo fmt --check'
run $D t2-clippy-workspace 'cargo clippy --locked --all-targets -- -D warnings'
run $D t3-clippy-sys-features 'cargo clippy --locked -p eech-engine-sys --features sector-height-check,sector-height-observer,entity-dialogue-check,collision-fixture --all-targets -- -D warnings'
run $D t4-clippy-engine-funnel 'cargo clippy --locked -p eech-engine --features sector-height-funnel-check --all-targets -- -D warnings'
run $D t5-workspace-tests-x64 'cargo test --locked --target x86_64-unknown-linux-gnu --no-fail-fast'
run $D t6-control-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture'
run $D t7-control-legacy 'cargo test --locked -p eech-engine-sys --features sector-height-check,collision-fixture --test sector_height -- --nocapture'
run $D t8-observer-control-legacy 'cargo test --locked -p eech-engine-sys --features sector-height-check,sector-height-observer,collision-fixture --test sector_height_observer'
run $D t9-dialogue-x64 'cargo test --locked -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue'
run $D t10-observer-chain-engine 'cargo test --locked -p eech-engine --features sector-height-funnel-check --test sector_height_funnel --test sector_height_observer_chain --test sector_height_observer_cap'
run $D t11-workspace-tests-x64-copt2 'env EECH_C_OPT=2 cargo test --locked --target x86_64-unknown-linux-gnu --no-fail-fast'
run $I t12-control-hosted-i686 'cargo test --locked --target i686-unknown-linux-gnu -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture'
run $I t13-control-legacy-i686 'cargo test --locked --target i686-unknown-linux-gnu -p eech-engine-sys --features sector-height-check,collision-fixture --test sector_height -- --nocapture'
run $I t14-dialogue-i686 'cargo test --locked --target i686-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue'
run $I t15-workspace-tests-i686 'cargo test --locked --target i686-unknown-linux-gnu --no-fail-fast'
run $I t16-workspace-tests-i686-stack 'env EECH_ORIGINAL_STACK_ATTRIBUTES=1 cargo test --locked --target i686-unknown-linux-gnu --no-fail-fast'
echo DONE >> shp-final-set-20261107.progress
