#!/bin/sh
# usage: quick.sh <logname> <cargo args...>   one cargo command in a disposable eech-build container on the sensor clippy volume; cargo exit captured; full output in <logname>.full
W="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections"
LD="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs"
NAME=$1
LOG="$LD/$NAME"
shift
docker run --rm -v "$W:/src" -v "$LD:/logs" -v eech-d2-sensor-clippy-target-2:/target -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -e NAME="$NAME" -w /src/eech-campaign eech-build sh -c '
rustup component add clippy rustfmt > /dev/null 2>&1
cargo "$@" > /tmp/q.out 2>&1; rc=$?
cp /tmp/q.out "/logs/$NAME.full"
grep -E "^(error|warning)" -A14 /tmp/q.out | head -150
grep -E "^test result|^running|Finished" /tmp/q.out
echo "CARGO_EXIT=$rc"' sh "$@" > "$LOG" 2>&1
echo "DOCKER_EXIT=$?" >> "$LOG"
