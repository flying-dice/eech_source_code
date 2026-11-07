#!/bin/sh
# The FINAL checks of the Core-load loader fix (ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1), one child at a time in the leased fresh volume (eech-linux-d3-height-reference-9d247-20261107, image sha256:9d247b4e... named by Root's explicit lease, never substituted),
# between the PRE and POST input maps (no source write in between). Product C changed (parsgen.c, the saved-campaign loader), so EVERY check that compiles the engine C is repeated: formatting, the engine-sys lint and default targets (all source guards, the
# new loader guard), the witness-feature C refusals and guards, the hosted and original-solver controls, engine/dc default and witness. No matrix, characterisation replay, native or Windows run: those are Root's with NEW Windows binaries from this new source.
# rustfmt and clippy are not in the leased image: `rustup component add` installs them into the EPHEMERAL container only (no image, volume or global change).
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-height-reference-direct-20261107.py
P=hwcore-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwcore-final-set-lease-before-20261107.txt 2>&1
run s1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run s2-clippy-workspace 'rustup component add clippy && cargo clippy --version && cargo clippy --locked --all-targets -- -D warnings'
run s3-clippy-witness-features 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc --features eech-engine-sys/sector-height-witness,eech-engine/sector-height-witness,eech-dc/sector-height-witness --all-targets -- -D warnings'
run s4-engine-sys-default 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast'
run s5-control-hosted 'cargo test --locked -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture'
run s6-control-original-solver 'cargo test --locked -p eech-engine-sys --features sector-height-check,collision-fixture --test sector_height -- --nocapture'
run s7-witness-c-and-guards 'cargo test --locked -p eech-engine-sys --features sector-height-witness --test sector_height_witness --test sector_height_witness_guard --test saved_campaign_load_guard --no-fail-fast'
run s8-engine-dc-default 'cargo test --locked -p eech-engine -p eech-dc --no-fail-fast'
run s9-engine-dc-witness 'cargo test --locked -p eech-engine -p eech-dc --features eech-engine/sector-height-witness,eech-dc/sector-height-witness --no-fail-fast'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwcore-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
