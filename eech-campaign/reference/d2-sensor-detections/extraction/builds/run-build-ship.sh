#!/bin/sh
# usage: run-build-ship.sh <out dir under eech-campaign/> <log>   the unchanged shipping recipe (tools/build-windows.sh, default features) in a private cargo volume
W="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections/eech-campaign"
cd "$W"
EECH_WIN_TARGET=eech-win-target-d2-sensor-shipping-2 sh tools/build-windows.sh "$1" > "$2" 2>&1
echo "BUILD_EXIT=$?" >> "$2"
