#!/bin/sh
# usage: run-build.sh <out dir under target/sensor-fixture> <log>
W="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections/eech-campaign"
cd "$W"
EECH_WIN_TARGET=eech-win-target-d2-sensor-fixture-2 sh tools/build-windows-sensor-fixture.sh "$1" > "$2" 2>&1
echo "BUILD_EXIT=$?" >> "$2"
