#!/bin/sh
# Copy of check-pose-motion-diag-linux-20261105.sh with the task-local derivative image (i686 target, i386 liblua5.1) and the same single cache volume.
set -eu
repo="$1"; shift
volume=eech-linux-d3-collision-driver-5416adb3b-20261029
test -z "$(docker ps -q --filter volume="$volume")"
image=eech-d3-tree-i686-20261106:2
docker run --rm -i -v "$repo:/src" -v "$volume:/target" -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -w /src/eech-campaign "$image" sh -c "$*"
