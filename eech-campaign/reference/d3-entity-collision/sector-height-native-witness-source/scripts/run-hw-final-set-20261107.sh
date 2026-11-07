#!/bin/sh
# The FINAL checks of the M5/M6 witness batch, one child at a time in the sole leased volume (eech-linux-d3-height-witnesses-20261107, image sha256:810e0121... named, never substituted), between the PRE and POST
# input maps (no source write in between). Each child is a direct runner invocation whose recorded actual_exit is the container's own exit; the progress file lists the real exits. rustfmt and clippy are not in the
# leased image: `rustup component add` installs them into the EPHEMERAL container only (no image, volume or global change; recorded by the child's own log).
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-height-witness-direct-20261107.py
P=hw-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
# the lease, read-only: the image id and no container before the first child
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' sha256:810e0121c7081fc7f4b51d8973beec085759e659be46e241e0bb8864245b13c2; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hw-final-set-lease-before-20261107.txt 2>&1
run v1-fmt-check 'rustup component add rustfmt && cargo fmt --check'
run v2-clippy-workspace 'rustup component add clippy && cargo clippy --locked --all-targets -- -D warnings'
run v3-clippy-witness-features 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc --features eech-engine-sys/sector-height-witness,eech-engine/sector-height-witness,eech-dc/sector-height-witness --all-targets -- -D warnings'
run v4-world-driver-tests 'cargo test --locked -p eech-world --test sector_height_witness_driver --test entity_authority --test sector_height_baseline_driver --test host_callbacks --no-fail-fast'
run v5-engine-sys-default 'cargo test --locked -p eech-engine-sys --no-fail-fast'
run v6-control-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture'
run v7-control-original-solver 'cargo test --locked -p eech-engine-sys --features sector-height-check,collision-fixture --test sector_height -- --nocapture'
run v8-witness-c-refusals-and-guard 'cargo test --locked -p eech-engine-sys --features sector-height-witness --test sector_height_witness --test sector_height_witness_guard --no-fail-fast'
run v9-engine-dc-default 'cargo test --locked -p eech-engine -p eech-dc --no-fail-fast'
run v10-engine-dc-witness 'cargo test --locked -p eech-engine -p eech-dc --features eech-engine/sector-height-witness,eech-dc/sector-height-witness --no-fail-fast'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hw-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
