#!/bin/sh
# The FINAL checks of the consumer-reference correction (review 507f49b7 residual of ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1), one child at a time in the FRESH leased volume (eech-linux-d3-height-reference-9d247-20261107, image
# sha256:9d247b4e... named by Root's explicit lease Root-height-reference-reader-9d247-lease-20261107.json, never substituted; the old 810e image is unresident), between the PRE and POST input maps (no source write in between), and ONLY after Root's released cache marker. Only the CHANGED inputs are checked: formatting, the World lint, and the affected
# World driver targets (the witness driver tests and the census/authority/baseline targets that read the Lua). No unchanged C check, matrix, native or Windows run is repeated: the compiled inputs are shown identical to e42bb19a0/17f by the join.
# rustfmt and clippy are not in the leased image: `rustup component add` installs them into the EPHEMERAL container only (no image, volume or global change).
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
D=run-height-reference-direct-20261107.py
P=hwref-final-set-20261107.progress
test ! -e "$P" || exit 2
: >"$P"
run() { "$PY" -I -B -X utf8 "$D" "$1" "$2" >/dev/null 2>&1; echo "$1 exit=$?" >>"$P"; }
{ wsl -d EECH-D2-Build --cd / -- docker image inspect --format '{{.Id}}' sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3; echo "containers_before=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')"; } >hwref-final-set-lease-before-20261107.txt 2>&1
run z1-fmt-check 'rustup component add rustfmt && cargo fmt --version && cargo fmt --check'
run z2-clippy-world 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-world --all-targets -- -D warnings'
run z3-world-driver-tests 'cargo --version && cargo test --locked -p eech-world --test sector_height_witness_driver --test entity_authority --test sector_height_baseline_driver --test host_callbacks --no-fail-fast'
echo "containers_after=$(wsl -d EECH-D2-Build --cd / -- docker ps -q | wc -l | tr -d ' ')" >hwref-final-set-lease-after-20261107.txt 2>&1
echo DONE >>"$P"
