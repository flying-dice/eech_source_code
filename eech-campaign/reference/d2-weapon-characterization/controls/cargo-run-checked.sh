#!/bin/sh
# One Cargo command. Preserve complete output and propagate every actual failure.
set -u
if [ "$#" -lt 3 ]; then
    echo "usage: cargo-run.sh <logfile> <volume> <cargo args...>" >&2
    exit 2
fi
log=$1
vol=$2
shift 2
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code" || exit 2
docker run --rm -v "$PWD:/src" -v "$vol:/target" -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -w /src/eech-campaign eech-build sh -c '
rustup component add clippy rustfmt
component_exit=$?
printf "RUSTUP_ADD_EXIT=%s\n" "$component_exit"
if [ "$component_exit" -ne 0 ]; then exit "$component_exit"; fi
cargo "$@"
cargo_exit=$?
printf "CARGO_EXIT=%s\n" "$cargo_exit"
exit "$cargo_exit"
' sh "$@" > "$log" 2>&1
docker_exit=$?
printf "DOCKER_EXIT=%s\n" "$docker_exit" >> "$log"
exit "$docker_exit"
