#!/bin/sh
# The FINAL checks of the PUBLIC WITNESS batch (the host-owned Lua driver of the physical flag store, its scripted-engine tests over the real provider, the exact pattern helpers shared with entity-authority.lua and the one pinned
# script count), one child at a time in the WARM sole volume named by the helper (check-physical-flags-witness-linux-20261107.sh: volume and image are read from it by the freeze), between the PRE and POST input maps (no source write in
# between). The compiled inputs of the product (C and Rust under aphavoc, eech-engine-sys, eech-engine, eech-dc and the eech-world src) are UNCHANGED from the 020 sources: only Lua, tests and docs changed, so the set is the AFFECTED one:
# formatting; the lint of the host, engine and bridge crates (the driver tests are in eech-world); the driver tests alone (so their count is read apart); the host's whole suite (the entity-authority.lua change reaches several binaries and
# host_callbacks.rs pins the script count); the engine layer's two source guards that read Lua text. No hosted or original C control, no matrix, characterisation replay, native or Windows run: nothing they compile changed.
# rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-witness-direct-20261107.py
H=check-physical-flags-witness-linux-20261107.sh
P=hwwit-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >hwwit-final-set-lease-before-20261107.txt 2>&1
run w1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run w2-clippy-world-engine-dc 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-world -p eech-engine -p eech-dc --all-targets -- -D warnings'
run w3-witness-driver 'cargo --version && cargo test --locked -p eech-world --no-fail-fast --test physical_flags_witness_driver'
run w4-world-all 'cargo test --locked -p eech-world --no-fail-fast'
run w5-engine-sys-lua-guards 'cargo test --locked -p eech-engine-sys --no-fail-fast --test entity_roster_guard --test physical_flags_guard'
{ echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >hwwit-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
