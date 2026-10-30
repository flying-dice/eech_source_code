#!/bin/sh
# usage (inside the EECH-D2-Build distro): static-checks.sh <logfile> <clippy target volume>
# Disposable container of the repository's eech-build image on a fresh target volume. cargo's own exit status is captured per step (no pipeline).
log=$1; vol=$2
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code" || exit 2
if docker volume ls --format '{{.Name}}' | grep -Fx "$vol" >/dev/null; then echo "Refusing to reuse volume $vol" >&2; exit 2; fi
docker run --rm -v "$PWD:/src" -v "$vol:/target" -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target -w /src/eech-campaign eech-build sh -c '
rustup component add clippy rustfmt >/tmp/rustup.out 2>&1; echo RUSTUP_ADD_EXIT=$?
cargo fmt --all --check >/tmp/fmt.out 2>&1; echo FMT_EXIT=$?; tail -30 /tmp/fmt.out
cargo clippy --locked --all-targets -- -D warnings >/tmp/c1.out 2>&1; echo CLIPPY_DEFAULT_EXIT=$?; tail -8 /tmp/c1.out
cargo clippy --locked -p eech-engine -p eech-dc --features eech-dc/weapon-fixture --all-targets -- -D warnings >/tmp/c2.out 2>&1; echo CLIPPY_WEAPON_EXIT=$?; tail -8 /tmp/c2.out
' > "$log" 2>&1
echo DOCKER_EXIT=$? >> "$log"
