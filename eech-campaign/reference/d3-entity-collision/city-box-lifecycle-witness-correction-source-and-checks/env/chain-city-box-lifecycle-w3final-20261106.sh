#!/bin/sh
PY="/c/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
cd /r/d3-evidence
R() { "$PY" -I -B -X utf8 run-city-box-lifecycle-direct-20261106.py "$1" "$2" < /dev/null; }
R w3f-fmt-world 'cargo fmt -p eech-world -- --check'
R w3f-fmt-engine-sys 'cargo fmt -p eech-engine-sys -- --check'
R w3f-fmt-engine 'cargo fmt -p eech-engine -- --check'
R w3f-clippy-engine-ordinary 'cargo clippy --locked --all-targets -p eech-engine -- -D warnings'
R w3f-clippy-world 'cargo clippy --locked --all-targets -p eech-world -- -D warnings'
R w3f-clippy-hosted 'cargo clippy --locked --all-targets -p eech-engine-sys --features weapon-fixture -- -D warnings'
R w3f-clippy-original 'cargo clippy --locked --all-targets -p eech-engine-sys --features collision-fixture,weapon-fixture -- -D warnings'
R w3f-clippy-dc 'cargo clippy --locked --all-targets -p eech-dc --features weapon-fixture -- -D warnings'
R w3f-test-engine-sys 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_roster_guard'
R w3f-test-world 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --test entity_authority --test city_box_lifecycle'
