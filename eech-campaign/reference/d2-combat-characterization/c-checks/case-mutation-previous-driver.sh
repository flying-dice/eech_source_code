#!/bin/sh
# usage (inside EECH-D2-Build): case-mutation-previous-driver.sh <logfile>: the SAME case_checks.c against a PREVIOUS controlled-case driver (default commit 450ecf60a, or the commit in EECH_PREVIOUS_DRIVER_COMMIT; read from git), compiled and
# run with the test-only flags of run-case-checks.sh.  Expected: the checks FAIL and the old off-map position aborts, so the helper returns a NONZERO status (the mutant's own); a
# control, never a product build.
# Status honesty (the pattern of run-observer-checks.sh at f5430bf1e): the compile status is captured and logged and the run is SKIPPED when it is nonzero; the container's status
# (the run's, or the compile's) is the Docker status, which is printed to the log and is also this helper's exit status.  Nothing is masked by a final echo.
# EECH_CMB_GUARD_CC (default gcc) is a TASK-SCOPED compiler override for the bounded status controls only (c-checks/case-helper-status-controls.sh).
set -u
if [ "$#" -ne 1 ]; then echo "usage: case-mutation-previous-driver.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
prev=$(mktemp -d) || exit 2
trap 'rm -rf "$prev"' EXIT
if ! git show "${EECH_PREVIOUS_DRIVER_COMMIT:-450ecf60a}:eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_case.c" > "$prev/eech_cmb_case.c"; then
	echo "PREVIOUS_DRIVER_NOT_READABLE" > "$log"
	echo "DOCKER_EXIT=2" >> "$log"
	exit 2
fi
docker run --rm -v "$PWD:/src:ro" -v "$prev:/prev:ro" -e CMB_GUARD_CC="${EECH_CMB_GUARD_CC:-gcc}" -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
echo "TEST_LINK_FLAGS=-fno-pie -no-pie"
echo "SANITIZERS=$san"
mkdir -p /tmp/cs && cp /prev/eech_cmb_case.c /tmp/cs/eech_cmb_case.c && cp "$C/case_checks.c" /tmp/cs/case_checks.c
"$CMB_GUARD_CC" -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -fno-pie -no-pie -I "$C/case-stub" -o /tmp/case_mutant /tmp/cs/case_checks.c /tmp/cs/eech_cmb_case.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then echo "RUN_NOT_STARTED"; exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/case_mutant) > /tmp/mutant.log 2>&1
run=$?
cat /tmp/mutant.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
