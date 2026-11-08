#!/bin/sh
# The FINAL checks of the MOTION-RESPONSE FLAG GROUP batch (family mfr332; derived from the tvr69c final set, then rewritten by hand). This batch changes: one feature-gated C fixture, its test, the feature's manifest entry
# and build define, the gate of two test-only helpers in the product C mb_hostmodel.c (a preprocessor condition), the source guard and the contract. So the set is the AFFECTED one: formatting; the lint of the crate in its
# default build and with the new feature in the hosted and the original profile; the new control itself in both profiles (counts and host traffic are read from the logs after they complete); the source guard (it now pins
# the six set sites and the feature's confinement); the three existing flag controls in both profiles (mb_hostmodel.c is compiled by them: the gate edit must not have moved them); and the default shipping module build
# (no feature leaked, mb_hostmodel.c compiles with the edited gate). No matrix, replay of an accepted proof, native, Windows or full-pack run.
# One child at a time in the WARM sole volume named by the helper (check-physical-flags-mfr332-linux-20261107.sh: volume and image are read from it by the freeze), between the PRE and POST input maps (no source write in
# between). rustfmt and clippy are installed into the EPHEMERAL container only, in the SAME invocation as the command.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-mfr332-direct-20261107.py
H=check-physical-flags-mfr332-linux-20261107.sh
P=mfr332-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >mfr332-final-set-lease-before-20261107.txt 2>&1
run s1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run s2-clippy-default 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run s3-clippy-motion-hosted 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --features physical-flags-motion-check --all-targets -- -D warnings'
run s4-clippy-motion-original 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --features physical-flags-motion-check,collision-fixture --all-targets -- -D warnings'
run s5-motion-hosted 'cargo --version && cargo test --locked -p eech-engine-sys --features physical-flags-motion-check --test physical_flags_motion -- --nocapture'
run s6-motion-original 'cargo test --locked -p eech-engine-sys --features physical-flags-motion-check,collision-fixture --test physical_flags_motion -- --nocapture'
run s7-flags-guard 'cargo test --locked -p eech-engine-sys --no-fail-fast --test physical_flags_guard'
run s8-flags-controls-hosted 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_collision -- --nocapture'
run s9-flags-controls-original 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check,collision-fixture --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_collision -- --nocapture'
run s10-build-dc 'cargo build --locked -p eech-dc'
{ echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >mfr332-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
