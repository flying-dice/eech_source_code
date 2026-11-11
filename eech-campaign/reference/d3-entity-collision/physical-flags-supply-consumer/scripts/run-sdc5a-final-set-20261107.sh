#!/bin/sh
# The FINAL checks of the SUPPLY-DESCRIPTOR batch (family sdc5a): the third real immediate-consumer control of the per-entity UPDATED word, the shipping supply arrival descriptor against the engine's own Rust boundary, in BOTH profiles,
# one child at a time in the WARM sole volume named by the helper (check-physical-flags-sdc5a-linux-20261107.sh: volume and image are read from it by the freeze), between the PRE and POST input maps (no source write in between).
# The inputs that changed are: one C fixture world (behind its own feature), one test-only seam of eech-engine (behind the same feature), the gate of two test-only helpers of mb_hostmodel.c, two Cargo manifests (and the lock), the
# guard and the contract, so the set is the AFFECTED one: formatting; the lint of the engine in the hosted and in the original profile with the control's feature, and the lint of the default builds of the engine layer and its
# dependants; the control itself in the hosted and in the original profile (the descriptor counts are read from the logs); the source guard of the flag census (it now pins the seam and the consumer's read order); the existing flag controls
# in both profiles (mb_hostmodel.c is compiled by them: the gate edit must not have moved them); the engine's default tests and the default shipping module build (no feature leaked). No matrix, characterisation replay, native, Windows
# or full-pack run. rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-sdc5a-direct-20261107.py
H=check-physical-flags-sdc5a-linux-20261107.sh
P=sdc5a-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >sdc5a-final-set-lease-before-20261107.txt 2>&1
run s1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run s2-clippy-engine-hosted 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine --features physical-flags-supply-check --all-targets -- -D warnings'
run s3-clippy-engine-original 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine --features physical-flags-supply-check,collision-fixture --all-targets -- -D warnings'
run s4-clippy-defaults 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world --all-targets -- -D warnings'
run s5-supply-hosted 'cargo --version && cargo test --locked -p eech-engine --features physical-flags-supply-check --test physical_flags_supply -- --nocapture'
run s6-supply-original 'cargo test --locked -p eech-engine --features physical-flags-supply-check,collision-fixture --test physical_flags_supply -- --nocapture'
run s7-flags-guard 'cargo test --locked -p eech-engine-sys --no-fail-fast --test physical_flags_guard'
run s8-flags-controls-hosted 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_collision -- --nocapture'
run s9-flags-controls-original 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check,collision-fixture --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_collision -- --nocapture'
run s10-engine-default 'cargo test --locked -p eech-engine --no-fail-fast'
run s11-build-dc 'cargo build --locked -p eech-dc'
{ echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; date -u +%Y-%m-%dT%H:%M:%SZ; } >sdc5a-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
