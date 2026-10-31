#!/bin/sh
set -eu
repo='/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/ground-host-preparation-20261029'
volume=eech-linux-tests-d3-ground-host-lua-working-20261029
image=sha256:4a5618cbc1e13e96caa4cb1ff0076c4c24dc883876d0bdec1398024f3a2d6c51
if docker volume inspect "$volume" >/dev/null 2>&1; then echo 'Refusing existing G2 Lua test target'; exit 2; fi
docker run --rm -v "$repo:/src" -v "$volume:/target" -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -w /src/eech-campaign "$image" sh -c '
set -eu
cargo fmt --all
cargo fmt --all --check
cargo test --release --locked -p eech-world --lib terrain::tests:: -- --nocapture
cargo test --release --locked -p eech-world --test ground_provider --test ground_handoff -- --nocapture
cargo clippy --locked -p eech-world --all-targets -- -D warnings
'
