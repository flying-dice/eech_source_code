#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-object-checks-pre-reverse.sh <logfile> <git revision>: the RED control of the reverse namespace check (EEDCS-46, R3).  The roster's acceptance of a new member's identity
# lived inline in campaign_roster_link_member until it was moved, unchanged, into gp_rosterid.c together with the check that the identity was never issued for a structure.  This script extracts those inline
# lines AS THEY WERE at <git revision> (a length of 1 to 128 bytes, UTF-8, not an identity of the roster) and the valid_utf8 they call, wraps them in the same function, and runs the CURRENT object_checks.c
# against that: the checks of the reverse direction are EXPECTED to fail there (exit 1 and a `FAIL` line), the rest to pass.  A compile failure, a sanitizer report or a timeout is a failure of the control, never
# the red.  The probe, compile, run and Docker statuses are captured, logged and propagated.  NOT a native run.
set -u
if [ "$#" -ne 2 ]; then echo "usage: run-object-checks-pre-reverse.sh <logfile> <git revision>" >&2; exit 2; fi
log=$1; rev=$2
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
old=$(mktemp -d) || exit 2
src="aphavoc/source/entity/special/group/gp_roster.c"
if ! git show "$rev:$src" > "$old/gp_roster.old.c"; then echo "cannot extract $rev:$src" > "$log"; exit 2; fi
{
  echo '#include "project.h"'
  echo '#include <string.h>'
  echo '#include "gp_roster.h"'
  awk '/^static int valid_utf8/ {f=1} f {print} f && /^}/ {exit}' "$old/gp_roster.old.c"
  echo 'void campaign_roster_accept_host_id (const char *host_id, int (*roster_has) (const char *host_id))'
  echo '{'
  echo '	size_t length;'
  awk '/length = strnlen \(host_id, CAMPAIGN_HOST_ID_MAX \+ 1\);/ {f=1} f {print} f && /was reused for a new physical incarnation/ {g=1} g && /^\t\t}/ {exit}' "$old/gp_roster.old.c" | sed 's/find_duplicate_host_id (host_id)/roster_has (host_id)/'
  echo '}'
} > "$old/gp_rosterid.c"
if ! grep -q "was reused for a new physical incarnation" "$old/gp_rosterid.c" || ! grep -q "valid_utf8 ((const" "$old/gp_rosterid.c"; then echo "cannot rebuild the acceptance from $rev" > "$log"; exit 2; fi
docker run --rm -v "$PWD:/src:ro" -v "$old:/old:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
G=/src/aphavoc/source/entity/special/group
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
echo "SANITIZERS=$san"
echo "---- the acceptance as it was:"
cat /old/gp_rosterid.c | tail -n 30
gcc -std=gnu99 -Wall -Wno-unused-function -Wno-unused-parameter $san -fno-pie -no-pie -I "$C/object-stub" -I "$R" -I "$G" -I /src/aphavoc/source -o /tmp/object_checks "$C/object_checks.c" "$R/mb_hostaim.c" "$R/mb_hostobj.c" /old/gp_rosterid.c -lm
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
