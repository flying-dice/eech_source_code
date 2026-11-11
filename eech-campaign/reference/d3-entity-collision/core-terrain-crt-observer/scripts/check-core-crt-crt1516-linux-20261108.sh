#!/bin/sh
# One Linux check of the CORE LOADER CRT OBSERVER batch (family crt1516) in the SOLE leased volume of the physical-flags lane (cache lease released by Root's literal physical-flags-020-WINDOWS-CACHE-RELEASE-20261107.json, SHA-256 2272fd77...) (Root lease: sole fresh volume eech-linux-d3-physical-flags-9d247-20261107, exact image 9d247). Usage: <repo-in-/mnt/c-path> '<shell command>'
# The image and volume are named, never substituted; the volume is refused if any container uses it. The shared cargo registry volume is mounted READ-ONLY and only to seed a registry copy INSIDE the leased volume
# (CARGO_HOME=/target/cargo-home), so no other volume is written. No image build, no process or volume deletion.
set -eu
repo="$1"; shift
volume=eech-linux-d3-physical-flags-9d247-20261107
image=sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3
test "$(docker image inspect --format '{{.Id}}' "$image")" = "$image"
if docker volume inspect "$volume" >/dev/null 2>&1; then test -z "$(docker ps -q --filter volume="$volume")"; fi
exec docker run --rm -i -v "$repo:/src" -v "$volume:/target" -v eech-cargo:/seed-registry:ro -e CARGO_TARGET_DIR=/target/t -e CARGO_HOME=/target/cargo-home -w /src/eech-campaign "$image" \
  sh -c 'test -d /target/cargo-home/registry || { mkdir -p /target/cargo-home && cp -a /seed-registry /target/cargo-home/registry; }; '"$*"
