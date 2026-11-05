#!/bin/sh
# usage (inside EECH-D2-Build): case-helper-status-controls.sh <output directory>: bounded controls of the STATUS HONESTY of the two controlled-case helpers, run, never read
# off the source: each helper is run normally and again with the task-scoped compiler override EECH_CMB_GUARD_CC=false (a forced compile failure), and its actual exit status and
# log are checked.  The helpers themselves propagate a nonzero status; THIS controller exits 0 only after it has asserted every expected status, and says so line by line.
#   run-case-checks.sh                normal: probe/compile/run/Docker 0, CASE_CHECKS=ok.   forced compile failure: COMPILE_EXIT=1, RUN_NOT_STARTED, no RUN_EXIT, DOCKER_EXIT=1, exit 1
#   case-mutation-previous-driver.sh  normal: compile 0, the previous driver fails the checks and aborts at the off-map setter, so a NONZERO run status that is also the Docker status
#                                     and the helper's exit status.   forced compile failure: COMPILE_EXIT=1, RUN_NOT_STARTED, no RUN_EXIT, DOCKER_EXIT=1, exit 1
set -u
if [ "$#" -ne 1 ]; then echo "usage: case-helper-status-controls.sh <output directory>" >&2; exit 2; fi
out=$1
mkdir -p "$out" || exit 2
here=$(cd "$(dirname "$0")" && pwd)
failures=0

expect() {
	if [ "$2" = ok ]; then echo "ok   $1"; else echo "FAIL $1"; failures=$((failures + 1)); fi
}

has() { grep -q "$2" "$1" && echo ok || echo no; }
lacks() { grep -q "$2" "$1" && echo no || echo ok; }

# ---- run-case-checks.sh, normal
sh "$here/run-case-checks.sh" "$out/helper-run-normal.log"; code=$?
expect "run-case-checks.sh normal: exit 0" "$([ "$code" -eq 0 ] && echo ok || echo no)"
expect "  log: probe, compile, run and Docker status 0 and CASE_CHECKS=ok" "$( [ "$(has "$out/helper-run-normal.log" 'COMPILE_EXIT=0')" = ok ] && [ "$(has "$out/helper-run-normal.log" 'RUN_EXIT=0')" = ok ] && [ "$(has "$out/helper-run-normal.log" 'DOCKER_EXIT=0')" = ok ] && [ "$(has "$out/helper-run-normal.log" 'CASE_CHECKS=ok')" = ok ] && echo ok || echo no)"

# ---- run-case-checks.sh, forced compile failure
EECH_CMB_GUARD_CC=false sh "$here/run-case-checks.sh" "$out/helper-run-forced-compile-failure.log"; code=$?
log=$out/helper-run-forced-compile-failure.log
expect "run-case-checks.sh forced compile failure: exit 1 (the outer status)" "$([ "$code" -eq 1 ] && echo ok || echo no)"
expect "  log: COMPILE_EXIT=1, RUN_NOT_STARTED, no RUN_EXIT line, DOCKER_EXIT=1" "$( [ "$(has "$log" 'COMPILE_EXIT=1')" = ok ] && [ "$(has "$log" 'RUN_NOT_STARTED')" = ok ] && [ "$(lacks "$log" '^RUN_EXIT=')" = ok ] && [ "$(has "$log" '^DOCKER_EXIT=1')" = ok ] && echo ok || echo no)"

# ---- case-mutation-previous-driver.sh, normal: the previous driver must fail, and the failure must reach the exit status
sh "$here/case-mutation-previous-driver.sh" "$out/helper-mutant-normal.log"; code=$?
log=$out/helper-mutant-normal.log
run_status=$(sed -n 's/^RUN_EXIT=//p' "$log" | head -1)
docker_status=$(sed -n 's/^DOCKER_EXIT=//p' "$log" | head -1)
expect "case-mutation-previous-driver.sh normal: a NONZERO exit (the mutant's own: $code)" "$([ "$code" -ne 0 ] && echo ok || echo no)"
expect "  log: COMPILE_EXIT=0, checks FAIL, the off-map engine stop, no CASE_CHECKS=ok" "$( [ "$(has "$log" 'COMPILE_EXIT=0')" = ok ] && [ "$(has "$log" '^FAIL ')" = ok ] && [ "$(has "$log" 'ENGINE STOP: position off map')" = ok ] && [ "$(lacks "$log" 'CASE_CHECKS=ok')" = ok ] && echo ok || echo no)"
expect "  RUN_EXIT ($run_status), DOCKER_EXIT ($docker_status) and the helper's exit ($code) are the same nonzero status" "$([ -n "$run_status" ] && [ "$run_status" = "$docker_status" ] && [ "$docker_status" = "$code" ] && [ "$code" -ne 0 ] && echo ok || echo no)"

# ---- case-mutation-previous-driver.sh, forced compile failure
EECH_CMB_GUARD_CC=false sh "$here/case-mutation-previous-driver.sh" "$out/helper-mutant-forced-compile-failure.log"; code=$?
log=$out/helper-mutant-forced-compile-failure.log
expect "case-mutation-previous-driver.sh forced compile failure: exit 1 (the outer status)" "$([ "$code" -eq 1 ] && echo ok || echo no)"
expect "  log: COMPILE_EXIT=1, RUN_NOT_STARTED, no RUN_EXIT line, DOCKER_EXIT=1" "$( [ "$(has "$log" 'COMPILE_EXIT=1')" = ok ] && [ "$(has "$log" 'RUN_NOT_STARTED')" = ok ] && [ "$(lacks "$log" '^RUN_EXIT=')" = ok ] && [ "$(has "$log" '^DOCKER_EXIT=1')" = ok ] && echo ok || echo no)"

if [ "$failures" -eq 0 ]; then
	echo "CASE_HELPER_STATUS_CONTROLS=ok: every expected status was asserted (this controller's exit 0 is that assertion; the helpers propagate their own nonzero statuses)"
	exit 0
fi
echo "CASE_HELPER_STATUS_CONTROLS=FAILED: $failures assertions"
exit 1
