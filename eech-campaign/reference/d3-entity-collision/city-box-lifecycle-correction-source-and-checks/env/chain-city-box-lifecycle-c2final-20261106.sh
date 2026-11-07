#!/bin/sh
PY="/c/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
cd /r/d3-evidence
R() { "$PY" -I -B -X utf8 run-city-box-lifecycle-direct-20261106.py "$1" "$2" < /dev/null; }
R c2f-fmt-world 'cargo fmt -p eech-world -- --check'
R c2f-fmt-engine 'cargo fmt -p eech-engine-sys -- --check'
R c2f-fmt-dc 'cargo fmt -p eech-dc -- --check'
R c2f-clippy-world 'cargo clippy --locked --all-targets -p eech-world -- -D warnings'
R c2f-clippy-engine 'cargo clippy --locked --all-targets -p eech-engine-sys -- -D warnings'
R c2f-clippy-hosted 'cargo clippy --locked --all-targets -p eech-engine-sys --features weapon-fixture -- -D warnings'
R c2f-clippy-original 'cargo clippy --locked --all-targets -p eech-engine-sys --features collision-fixture,weapon-fixture -- -D warnings'
R c2f-clippy-dc 'cargo clippy --locked --all-targets -p eech-dc --features weapon-fixture -- -D warnings'
R c2f-test-engine 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue --test entity_roster_guard --test entity_host_guard'
R c2f-test-world 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --lib --test entity_roster_host --test city_box_lifecycle --test entity_authority --test entity_candidate_host'
R c2f-copt2-engine 'env EECH_C_OPT=2 cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue --test entity_roster_guard'
R c2f-copt2-world 'env EECH_C_OPT=2 cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --lib --test entity_roster_host --test city_box_lifecycle --test entity_authority'
