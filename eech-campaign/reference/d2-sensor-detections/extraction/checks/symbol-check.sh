#!/bin/sh
# usage: symbol-check.sh <log>   symbols of the shipping candidate's scan objects, from the private shipping cargo volume
LOG="$1"
docker run --rm -v eech-win-target-d2-sensor-shipping-2:/target eech-build sh -c '
NM=$(command -v x86_64-w64-mingw32-nm || command -v nm)
echo "nm: $NM"
for f in ac_tgt vh_tgt mb_hostsense; do
  o=$(ls /target/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/*-$f.o 2>/dev/null | head -1)
  echo "=== $f object: $o"
  echo "--- defined (T/t text):"; $NM "$o" | grep -E " [Tt] " | sed "s/^[0-9a-f]* //"
  echo "--- undefined (U):"; $NM "$o" | grep " U " | sed "s/^ *U //" | tr "\n" " "; echo
done
echo "=== any fixture or sensor-observer symbol in the shipping objects"
cat /target/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/*.o | strings | grep -c -E "eech_sensor_(begin|end|skip|candidate|verdict|floor|tolerance|probe)|EECH_SENSOR_FIXTURE" || true
echo "=== the physical sensing arithmetic in the shipping scan objects"
for f in ac_tgt vh_tgt; do
  o=$(ls /target/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/*-$f.o | head -1)
  echo "$f references get_approx_3d_range: $($NM $o | grep -c -E "get_approx_3d_range")  get_sqr_2d_range: $($NM $o | grep -c get_sqr_2d_range)  frand1x/rand: $($NM $o | grep -c -E "rand")"
done' > "$LOG" 2>&1
echo "DOCKER_EXIT=$?" >> "$LOG"
