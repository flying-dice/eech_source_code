#!/bin/sh
PY="/c/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
cd /r/d3-evidence
"$PY" -I -B -X utf8 run-city-box-lifecycle-direct-20261106.py base-world 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --lib --test entity_roster_host' < /dev/null
"$PY" -I -B -X utf8 run-city-box-lifecycle-direct-20261106.py base-engine 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-engine-sys --features entity-dialogue-check --test entity_dialogue --test entity_roster_guard' < /dev/null
