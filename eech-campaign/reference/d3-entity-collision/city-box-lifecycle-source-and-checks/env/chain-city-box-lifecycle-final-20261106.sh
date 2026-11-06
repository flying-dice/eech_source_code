#!/bin/sh
PY="/c/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
cd /r/d3-evidence
R() { "$PY" -I -B -X utf8 run-city-box-lifecycle-direct-20261106.py "$1" "$2" < /dev/null; }
R final-fmt-world 'cargo fmt -p eech-world -- --check'
R final-fmt-engine 'cargo fmt -p eech-engine-sys -- --check'
R final-clippy-world 'cargo clippy --locked --all-targets -p eech-world -- -D warnings'
R final-clippy-engine 'cargo clippy --locked --all-targets -p eech-engine-sys -- -D warnings'
R final-test-engine 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue --test entity_roster_guard --test entity_host_guard'
R final-test-world 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --lib --test entity_roster_host --test city_box_lifecycle --test entity_authority --test entity_candidate_host'
R final-copt2-engine 'env EECH_C_OPT=2 cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue --test entity_roster_guard'
R final-copt2-world 'env EECH_C_OPT=2 cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --lib --test entity_roster_host --test city_box_lifecycle --test entity_authority'
