#!/bin/sh
# The FINAL checks of the PRODUCTION batch (the complete physical UPDATED / MOVED / ROTATED store moves from EECH_DC into its host; Task49 candidate), one child at a time in the WARM sole volume named by the helper (check-physical-flags-production-linux-20261107.sh: volume and image are read from it by
# the freeze), between the PRE and POST input maps (no source write in between). The compiled inputs CHANGED (the production move: see the join), so the set is: formatting; the lint of the engine layer in the default profile and with the flag/dialogue features; the lint of the host,
# engine and bridge crates; the engine layer's default targets (every source guard); the HOSTED flag controls (the real getter, setter and reset over the REAL host store, the negative controls, the dialogue and the census); the TRUE ORIGINAL profile (collision-fixture) controls with the
# arrays; the sector height controls in both profiles (the phase/state enums grew); the full production module build (the default build at its new identity); the host's whole suite (the public driver and provider) and the engine and bridge suites. No matrix, characterisation replay, native or
# Windows run. rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-production-direct-20261107.py
H=check-physical-flags-production-linux-20261107.sh
P=hwprod-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwprod-final-set-lease-before-20261107.txt 2>&1
run p1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run p2-clippy-engine-sys-default 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run p3-clippy-engine-sys-flag-features 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,entity-dialogue-check --all-targets -- -D warnings'
run p4-clippy-world-engine-dc 'rustup component add clippy && cargo clippy --locked -p eech-world -p eech-engine -p eech-dc --all-targets -- -D warnings'
run p5-engine-sys-default 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast'
run p6-flags-hosted-controls 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,entity-dialogue-check --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_hosted_faults --test physical_flags_guard --test entity_dialogue -- --nocapture'
run p7-flags-original-profile 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,collision-fixture --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_guard -- --nocapture'
run p8-sector-height-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --no-fail-fast --test sector_height --test sector_height_guard -- --nocapture'
run p9-sector-height-original 'cargo test --locked -p eech-engine-sys --features sector-height-check,collision-fixture --no-fail-fast --test sector_height --test sector_height_guard -- --nocapture'
run p10-build-dc 'cargo build --locked -p eech-dc'
run p11-world-all 'cargo test --locked -p eech-world --no-fail-fast'
run p12-engine-dc 'cargo test --locked -p eech-engine -p eech-dc --no-fail-fast'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwprod-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
