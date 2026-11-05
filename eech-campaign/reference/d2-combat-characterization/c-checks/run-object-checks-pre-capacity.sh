#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-object-checks-pre-capacity.sh <logfile> <git revision>: the RED control of the heap-capacity boundary (EEDCS-46, R3).  The six Windows runs of the shipping witness read
# the entity heap up to MAX_NUM_ENTITIES (the compile-time ceiling) when the run had allocated number_of_entities slots (far fewer): the faulting read was the entity type in campaign_object_structures.  This script
# compiles the CURRENT object_checks.c, whose heap is allocated EXACTLY number_of_entities slots (below the ceiling) and whose accessor counts any slot index outside it, against the mb_hostobj.c of an EARLIER
# revision (the one that iterated to the ceiling), with the current mb_hostaim.c and gp_rosterid.c.  The capacity checks are EXPECTED to fail there (exit 1 and a `FAIL` line): they find the reads outside the
# allocated capacity.  A compile failure, a sanitizer report that stops the run before the checks report, or a timeout is a failure of the control, never the red.  The probe, compile, run and Docker statuses are
# captured, logged and propagated.  NOT a native run.
set -u
if [ "$#" -ne 2 ]; then echo "usage: run-object-checks-pre-capacity.sh <logfile> <git revision>" >&2; exit 2; fi
log=$1; rev=$2
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
old=$(mktemp -d) || exit 2
if ! git show "$rev:aphavoc/source/entity/mobile/mb_hostobj.c" > "$old/mb_hostobj.c"; then echo "cannot extract $rev" > "$log"; exit 2; fi
docker run --rm -v "$PWD:/src:ro" -v "$old:/old:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
G=/src/aphavoc/source/entity/special/group
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
echo "SANITIZERS=$san"
gcc -std=gnu99 -Wall -Wno-unused-function -Wno-unused-parameter $san -fno-pie -no-pie -I "$C/object-stub" -I "$R" -I "$G" -I /src/aphavoc/source -o /tmp/object_checks "$C/object_checks.c" "$R/mb_hostaim.c" /old/mb_hostobj.c "$G/gp_rosterid.c" -lm
compile=$?
echo "COMPILE_EXIT=$compile"
if [ "$compile" -ne 0 ]; then echo "RUN_NOT_STARTED"; exit "$compile"; fi
(ulimit -f 128; timeout -k 2s 120s /tmp/object_checks) > /tmp/object-checks.log 2>&1
run=$?
cat /tmp/object-checks.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
rm -rf "$old"
exit "$docker_exit"
