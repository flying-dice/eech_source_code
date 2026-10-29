#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-blast-checks.sh <logfile>: compiles and runs blast_checks.c against the REAL aphavoc/source/entity/mobile/mb_hostaim.c in a
# disposable container (AddressSanitizer, UndefinedBehaviorSanitizer, non-PIE, as run-case-checks.sh).  The probe, compile, run and Docker statuses are captured, logged and
# propagated (the run is skipped after a failed compile or probe).  This checks the campaign bridge to the host; it is
# NOT a native run.
set -u
if [ "$#" -ne 1 ]; then echo "usage: run-blast-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -e CH_CC="${EECH_BLAST_CC:-gcc}" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
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
"$CH_CC" -std=gnu99 -Wall -Wno-unused-function -Wno-unused-parameter $san -fno-pie -no-pie -I "$C/forecast-stub" -I "$R" -o /tmp/blast_checks "$C/blast_checks.c" "$R/mb_hostaim.c" -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then echo "RUN_NOT_STARTED"; exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/blast_checks) > /tmp/blast-checks.log 2>&1
run=$?
cat /tmp/blast-checks.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
