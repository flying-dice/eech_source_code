#!/bin/sh
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs"
sh run-build-ship5.sh target/f-ship "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/f-build-shipping.log"
sh symbol-check5.sh "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/f-symbol-check.log"
sh run-build5.sh target/sensor-fixture/f "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/f-build-fixture.log"
sh quick3.sh f-fmt.log fmt --all --check
sh quick3.sh f-clippy-default.log clippy --locked --all-targets -- -D warnings
sh quick3.sh f-clippy-all-fixtures.log clippy --locked --all-targets --features eech-dc/dependent-fixture,eech-dc/loss-fixture,eech-dc/los-fixture,eech-dc/sensor-fixture -- -D warnings
sh quick3.sh f-clippy-old-guard-sensor.log clippy --locked -p eech-engine -p eech-dc --all-targets --features eech-dc/dependent-fixture-old-guard,eech-dc/sensor-fixture -- -D warnings
sh quick3.sh f-clippy-sensor-only.log clippy --locked -p eech-engine -p eech-dc -p eech-world --all-targets --features eech-dc/sensor-fixture -- -D warnings
echo ALLDONE > "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/f-all.done"
