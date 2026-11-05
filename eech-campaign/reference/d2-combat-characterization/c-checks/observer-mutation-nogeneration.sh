#!/bin/sh
# usage (inside EECH-D2-Build): observer-mutation-nogeneration.sh <logfile>: a SCRATCH copy of eech_cmb_fixture.c with the slot-generation comparison removed (the receiver is then
# identified by type and data alone, the old rule), compiled and run with the same test-only flags as run-observer-checks.sh.  Expected: the same-kind reuse checks FAIL, the
# different-kind reuse check still passes. The helper returns the actual mutant/compile/Docker status (normally expected1), never echo0. A control, never a product build.
set -u
if [ "$#" -ne 1 ]; then echo "usage: observer-mutation-nogeneration.sh <logfile>" >&2; exit 2; fi
log=$1
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections' || exit 2
docker run --rm -v "$PWD:/src:ro" -e CMB_GUARD_CC="${EECH_CMB_GUARD_CC:-gcc}" -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/obs && cp "$C/observer_checks.c" /tmp/obs/observer_checks.c
sed "s/(generation\[frame->slot\] != frame->generation) ||/0 ||/" /src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_fixture.c > /tmp/obs/eech_cmb_fixture.c
cmp -s /tmp/obs/eech_cmb_fixture.c /src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_fixture.c && { echo "MUTATION_NOT_APPLIED"; exit 3; }
diff /src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_fixture.c /tmp/obs/eech_cmb_fixture.c
"$CMB_GUARD_CC" -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -fno-pie -no-pie -I "$C/stub" -I /src/aphavoc/source -o /tmp/observer_mutant /tmp/obs/observer_checks.c /tmp/obs/eech_cmb_fixture.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/observer_mutant) > /tmp/mutant.log 2>&1
run=$?
cat /tmp/mutant.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
