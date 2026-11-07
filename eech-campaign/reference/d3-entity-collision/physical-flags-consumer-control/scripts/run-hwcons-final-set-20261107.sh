#!/bin/sh
# The FINAL checks of the consumer batch (the real immediate-consumer control of the UPDATED word + the prose correction), one child at a time in the WARM sole volume named by the helper (check-physical-flags-consumer-linux-20261107.sh: volume and image are read from it by the freeze), between the
# PRE and POST input maps (no source write in between). The compiled inputs gained ONLY test-only additions inert without their feature (a new gated translation unit, comments in the sealed first control, a manifest feature, one define block): formatting, the engine-sys lint with the default
# and with both flags features, the engine-sys default targets (all source guards incl. the census) and the three real-C/census targets with both features. No matrix, characterisation replay, native or Windows run. rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-consumer-direct-20261107.py
H=check-physical-flags-consumer-linux-20261107.sh
P=hwcons-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwcons-final-set-lease-before-20261107.txt 2>&1
run k1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run k2-clippy-engine-sys-default 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run k3-clippy-engine-sys-both-features 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check --all-targets -- -D warnings'
run k4-engine-sys-default 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast'
run k5-flags-controls-and-census 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check --test physical_flags --test physical_flags_consumer --test physical_flags_guard --no-fail-fast -- --nocapture'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwcons-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
