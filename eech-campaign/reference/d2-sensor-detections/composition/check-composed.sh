#!/bin/sh
# usage: check-composed.sh <logname>  fmt + clippy: default, all fixtures combined, old guard combined; each cargo exit captured separately
W="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections"
LOG="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/$1"
docker run --rm -v "$W:/src" -v eech-d2-sensor-clippy-target:/target -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -w /src/eech-campaign eech-build sh -c '
rustup component add clippy rustfmt > /dev/null 2>&1; echo "RUSTUP_ADD_EXIT=$?"
echo "HEAD_DIRTY_FILES=$(cd /src && git status --porcelain 2>/dev/null | wc -l)"
cargo fmt --all --check; echo "FMT_EXIT=$?"
cargo clippy --locked --all-targets -- -D warnings > /tmp/c1.out 2>&1; echo "CLIPPY_DEFAULT_EXIT=$?"; grep -E "^(error|warning)" -A12 /tmp/c1.out | head -60; tail -2 /tmp/c1.out
cargo clippy --locked -p eech-engine -p eech-dc -p eech-world --all-targets --features eech-dc/dependent-fixture,eech-dc/loss-fixture,eech-dc/los-fixture,eech-dc/sensor-fixture -- -D warnings > /tmp/c2.out 2>&1; echo "CLIPPY_ALL_FIXTURES_EXIT=$?"; grep -E "^(error|warning)" -A12 /tmp/c2.out | head -60; tail -2 /tmp/c2.out
cargo clippy --locked -p eech-engine -p eech-dc -p eech-world --all-targets --features eech-dc/dependent-fixture-old-guard,eech-dc/sensor-fixture -- -D warnings > /tmp/c3.out 2>&1; echo "CLIPPY_OLD_GUARD_SENSOR_EXIT=$?"; grep -E "^(error|warning)" -A12 /tmp/c3.out | head -60; tail -2 /tmp/c3.out
cargo clippy --locked -p eech-engine -p eech-dc -p eech-world --all-targets --features eech-dc/sensor-fixture -- -D warnings > /tmp/c4.out 2>&1; echo "CLIPPY_SENSOR_ONLY_EXIT=$?"; grep -E "^(error|warning)" -A12 /tmp/c4.out | head -60; tail -2 /tmp/c4.out
' > "$LOG" 2>&1
echo "DOCKER_EXIT=$?" >> "$LOG"
