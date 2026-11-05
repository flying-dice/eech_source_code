#!/bin/sh
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs"
sh run-build-ship4.sh target/p2-ship "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/p2-build-shipping.log"
sh symbol-check4.sh "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/p2-symbol-check.log"
sh run-build4.sh target/sensor-fixture/p2 "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs/p2-build-fixture.log"
