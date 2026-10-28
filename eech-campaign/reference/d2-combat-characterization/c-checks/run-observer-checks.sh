#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-observer-checks.sh <logfile>: compiles and runs observer_checks.c against the REAL csrc/eech_cmb_fixture.c in a disposable
# container, with AddressSanitizer and UndefinedBehaviorSanitizer when the compiler has them.  The compile, run and Docker statuses are captured, logged and propagated
# (the run is skipped after a failed compile); the complete raw output is kept.
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
O=/src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_fixture.c
san=""
if echo "int main(void){return 0;}" > /tmp/probe.c && gcc -fsanitize=address,undefined /tmp/probe.c -o /tmp/probe 2>/dev/null && /tmp/probe; then san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"; fi
echo "SANITIZERS=${san:-none}"
mkdir -p /tmp/obs && cp "$O" /tmp/obs/eech_cmb_fixture.c && cp "$C/observer_checks.c" /tmp/obs/observer_checks.c
gcc -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -I "$C/stub" -I /src/aphavoc/source -o /tmp/observer_checks /tmp/obs/observer_checks.c /tmp/obs/eech_cmb_fixture.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then exit "$compile"; fi
timeout 120 /tmp/observer_checks
run=$?
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
