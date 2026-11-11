#!/bin/sh
# THIS BATCH (tvr69c) is the CORRECTION of the Tacview disposition (guard enclosure predicate and identity-scope prose): one guard test and the contract changed, no production source; the set is the same shape as the previous batch's, whose comment follows.
# The FINAL checks of the TACVIEW-CONSUMER DISPOSITION batch (family tvr69c; derived from the sb120 final set, then rewritten by hand). This batch changes NO production source: one source-guard test (physical_flags_guard.rs, text only),
# the contract (section 13) and the packet. So the set is the AFFECTED one and small: formatting, the lint of the crate whose test changed (all targets), and the guard itself. No control, build, matrix, replay, native, Windows or
# full-pack run: nothing they compile or run changed since the characterised and reviewed sources (the product and crate sources equal 3abf03e51's; the retained 3abf reports are the compiled evidence of the disposition).
# One child at a time in the WARM sole volume named by the helper (check-physical-flags-tvr69c-linux-20261107.sh: volume and image are read from it by the freeze), between the PRE and POST input maps (no source write in between).
# rustfmt and clippy are installed into the EPHEMERAL container only, in the SAME invocation as the command.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-tvr69c-direct-20261107.py
H=check-physical-flags-tvr69c-linux-20261107.sh
P=tvr69c-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >tvr69c-final-set-lease-before-20261107.txt 2>&1
run s1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run s2-clippy-engine-sys 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run s3-flags-guard 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast --test physical_flags_guard'
{ echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >tvr69c-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
