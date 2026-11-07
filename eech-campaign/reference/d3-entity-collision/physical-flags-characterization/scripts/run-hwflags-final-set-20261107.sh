#!/bin/sh
# The FINAL checks of the physical-flags characterisation (ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1), one child at a time in the sole fresh volume (eech-linux-d3-physical-flags-9d247-20261107, image sha256:9d247b4e... named, never substituted), between the PRE and POST input maps
# (no source write in between). The compiled inputs gained ONLY test-only feature-gated additions (one translation unit, a manifest feature, one define block), so the checks that compile the engine C are repeated: formatting, the engine-sys lint with and without the new feature,
# the engine-sys default targets (all source guards incl. the new census) and the new real-C control with the feature. No matrix, characterisation replay, native or Windows run. rustfmt and clippy are installed into the EPHEMERAL container only.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-physical-flags-direct-20261107.py
P=hwflags-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwflags-final-set-lease-before-20261107.txt 2>&1
run f1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run f2-clippy-engine-sys-default 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys --all-targets -- -D warnings'
run f3-clippy-engine-sys-flags-feature 'rustup component add clippy && cargo clippy --locked -p eech-engine-sys --features physical-flags-check --all-targets -- -D warnings'
run f4-engine-sys-default 'cargo --version && cargo test --locked -p eech-engine-sys --no-fail-fast'
run f5-flags-control-and-guard 'cargo test --locked -p eech-engine-sys --features physical-flags-check --test physical_flags --test physical_flags_guard --no-fail-fast -- --nocapture'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwflags-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
