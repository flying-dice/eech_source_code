#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-case-checks.sh <logfile>: compiles and runs case_checks.c against the REAL csrc/eech_cmb_case.c in a disposable container, with the
# same test-only flags as run-observer-checks.sh (AddressSanitizer, UndefinedBehaviorSanitizer, non-PIE).  The probe, compile, run and Docker statuses are captured, logged and
# propagated (the run is skipped after a failed compile or probe); the run is bounded by a timeout and an output size cap.  This checks the driver's guards over a stand-in entity
# list; it is NOT a native run, and the real launch helper (which needs the whole game) is not linked: the stand-in answers with its shapes, pinned to csrc/eech_wpn_fixture.c.  The container's status (the probe's, the compile's or the run's) is the Docker status, printed to the log and also this helper's exit status.
# EECH_CMB_GUARD_CC (default gcc) is a TASK-SCOPED compiler override for the bounded status controls only (case-helper-status-controls.sh).
set -u
if [ "$#" -ne 1 ]; then echo "usage: run-case-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -e CMB_GUARD_CC="${EECH_CMB_GUARD_CC:-gcc}" -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
O=/src/eech-campaign/crates/eech-engine-sys/csrc/eech_cmb_case.c
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
# the stand-in query and launch answer with the shapes of the REAL helper; they are pinned to the real source (a missing pin fails the whole check before anything is compiled)
H=/src/eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c
pins_missing=0
for lit in "reason=subject_kind_unsupported" "reason=launcher_kind_unsupported" "reason=launcher_target_or_weapon_not_valid" "reason=target_link_not_set" "reason=no_such_host_id" "alive=%d position=" "weapon_count=%d warhead=%d" "valid_sub_type (weapon) ? (int) weapon_database[weapon].warhead_type : -1" "!get_local_entity_int_value (launcher, INT_TYPE_ALIVE)"; do
	if grep -Fq -- "$lit" "$H"; then echo "REAL_HELPER_PIN_OK $lit"; else echo "REAL_HELPER_PIN_MISSING $lit"; pins_missing=1; fi
done
echo "REAL_HELPER_PINS_MISSING=$pins_missing"
if [ "$pins_missing" -ne 0 ]; then exit 3; fi
mkdir -p /tmp/cs && cp "$O" /tmp/cs/eech_cmb_case.c && cp "$C/case_checks.c" /tmp/cs/case_checks.c
"$CMB_GUARD_CC" -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -fno-pie -no-pie -I "$C/case-stub" -o /tmp/case_checks /tmp/cs/case_checks.c /tmp/cs/eech_cmb_case.c -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then echo "RUN_NOT_STARTED"; exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/case_checks) > /tmp/case-checks.log 2>&1
run=$?
cat /tmp/case-checks.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
