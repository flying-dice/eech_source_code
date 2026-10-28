#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-bridge-checks-pre-guard.sh <logfile> <git revision>: the RED control of the destructive-target guard (EEDCS-46, R5).  Compiles and runs the CURRENT
# bridge_checks.c against the mb_hostaim.c and mb_hostaim.h of an EARLIER revision (before the guard), with -DBRIDGE_CHECKS_PRE_GUARD_DRIVER so that the checks do not assign the walk member that
# revision lacks, in a disposable container (AddressSanitizer, UndefinedBehaviorSanitizer, non-PIE).  The behavioural checks of the guard are EXPECTED to fail there (exit 1 and a `FAIL` line); a
# compile failure, a sanitizer report or a timeout is a failure of the control, never the red.  The probe, compile, run and Docker statuses are captured, logged and propagated.  NOT a native run.
set -u
if [ "$#" -ne 2 ]; then echo "usage: run-bridge-checks-pre-guard.sh <logfile> <git revision>" >&2; exit 2; fi
log=$1; rev=$2
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
old=$(mktemp -d) || exit 2
if ! git show "$rev:aphavoc/source/entity/mobile/mb_hostaim.c" > "$old/mb_hostaim.c" || ! git show "$rev:aphavoc/source/entity/mobile/mb_hostaim.h" > "$old/mb_hostaim.h"; then echo "cannot extract $rev" > "$log"; exit 2; fi
docker run --rm -v "$PWD:/src:ro" -v "$old:/old:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
echo "SANITIZERS=$san"
gcc -std=gnu99 -Wall -Wno-unused-function -Wno-unused-parameter $san -fno-pie -no-pie -DBRIDGE_CHECKS_PRE_GUARD_DRIVER -I "$C/forecast-stub" -I /old -I "$R" -o /tmp/bridge_checks "$C/bridge_checks.c" /old/mb_hostaim.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then echo "RUN_NOT_STARTED"; exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/bridge_checks) > /tmp/bridge-checks.log 2>&1
run=$?
cat /tmp/bridge-checks.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
rm -rf "$old"
exit "$docker_exit"
