#!/bin/sh
# The FINAL checks of the KEY-FIX batch (the two source findings of the consolidated review V2 of the production candidate 0d0c2545b: ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1 and ENV-PHYSICAL-FLAGS-ISSUED-GROUP-ALIAS-1, with the
# collision consumer control in the true original profile and the observed hosted block), one child at a time in the WARM sole volume named by the helper (check-physical-flags-keyfix-linux-20261107.sh: volume and image are read from it by the
# freeze), between the PRE and POST input maps (no source write in between). The compiled inputs CHANGED (the campaign's flag key domain and its group binding, the host's key bound, the bridge-side tests, the public driver, the controls), so the set
# is: formatting; the lint of the engine layer in the default profile, with the flag, collision and dialogue features and with the original profile; the lint of the host, engine and bridge crates; the engine layer's default targets (every source guard);
# the HOSTED flag controls (the 199 and 56 scenarios, the negative controls, the identity control, the collision control's observed block, the dialogue and the census); the TRUE ORIGINAL profile controls (arrays; the collision consumer's 15
# expectations); the sector height controls in both profiles (mb_hostmodel.c is compiled by them); the full production module build; the host's whole suite and the engine and bridge suites. No matrix, characterisation replay, native or Windows
# run. rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-keyfix-direct-20261107.py
H=check-physical-flags-keyfix-linux-20261107.sh
P=hwkey-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
IMAGE=$(sed -n 's/^image=//p' "$H")
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' "$IMAGE"; echo "image_line=$IMAGE"; echo "volume_line=$(sed -n 's/^volume=//p' "$H")"; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwkey-final-set-lease-before-20261107.txt 2>&1
run q1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run q2-clippy-engine-sys-default 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run q3-clippy-engine-sys-flag-features 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check,entity-dialogue-check --all-targets -- -D warnings'
run q4-clippy-engine-sys-original 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys --features physical-flags-check,physical-flags-collision-check,collision-fixture --all-targets -- -D warnings'
run q5-clippy-world-engine-dc 'rustup component add clippy && cargo clippy --locked -p eech-world -p eech-engine -p eech-dc --all-targets -- -D warnings'
run q6-engine-sys-default 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast'
run q7-flags-hosted-controls 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check,entity-dialogue-check --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_hosted_faults --test physical_flags_identity --test physical_flags_collision --test physical_flags_guard --test entity_dialogue -- --nocapture'
run q8-flags-original-profile 'cargo test --locked -p eech-engine-sys --features physical-flags-check,physical-flags-consumer-check,physical-flags-collision-check,collision-fixture --no-fail-fast --test physical_flags --test physical_flags_consumer --test physical_flags_collision --test physical_flags_guard -- --nocapture'
run q9-sector-height-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --no-fail-fast --test sector_height --test sector_height_guard -- --nocapture'
run q10-sector-height-original 'cargo test --locked -p eech-engine-sys --features sector-height-check,collision-fixture --no-fail-fast --test sector_height --test sector_height_guard -- --nocapture'
run q11-build-dc 'cargo build --locked -p eech-dc'
run q12-world-all 'cargo test --locked -p eech-world --no-fail-fast'
run q13-engine-dc 'cargo test --locked -p eech-engine -p eech-dc --no-fail-fast'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwkey-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
