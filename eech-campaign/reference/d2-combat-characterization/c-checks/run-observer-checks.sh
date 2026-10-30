#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-observer-checks.sh <logfile>: compiles and runs observer_checks.c against the REAL csrc/eech_cmb_fixture.c in a disposable
# container, with AddressSanitizer and UndefinedBehaviorSanitizer when the compiler has them.  The compile, run and Docker statuses are captured, logged and propagated
# (the run is skipped after a failed compile); bounded raw output is kept. Test-only non-PIE avoids the reproduced empty-program ASAN startup flood; shipping flags are unchanged.
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
O=/src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_fixture.c
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
echo "TEST_LINK_FLAGS=-fno-pie -no-pie"
echo "SANITIZERS=$san"
echo "int main(void){return 0;}" > /tmp/probe.c
gcc $san -fno-pie -no-pie /tmp/probe.c -o /tmp/probe
probe_compile=$?
echo "SANITIZER_PROBE_COMPILE_EXIT=$probe_compile"
if [ "$probe_compile" -ne 0 ]; then exit "$probe_compile"; fi
(ulimit -f 128; timeout -k 2s 10s /tmp/probe) > /tmp/probe.log 2>&1
probe_run=$?
cat /tmp/probe.log
echo "SANITIZER_PROBE_RUN_EXIT=$probe_run"
if [ "$probe_run" -ne 0 ]; then exit "$probe_run"; fi
mkdir -p /tmp/obs && cp "$O" /tmp/obs/eech_cmb_fixture.c && cp "$C/observer_checks.c" /tmp/obs/observer_checks.c
gcc -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -fno-pie -no-pie -I "$C/stub" -I /src/aphavoc/source -o /tmp/observer_checks /tmp/obs/observer_checks.c /tmp/obs/eech_cmb_fixture.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/observer_checks) > /tmp/observer-checks.log 2>&1
run=$?
cat /tmp/observer-checks.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
