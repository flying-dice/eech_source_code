#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-subject-checks.sh <logfile>: the mutation controls of the subject identity of the DAMAGE, HEALTH and IMPACT questions (EEDCS-46, R3).  Each control removes ONE line of a
# COPY of the real mb_hostaim.c (the checkout is mounted read-only) and runs the harness that asks the REAL campaign_aim_damage / campaign_aim_health / campaign_aim_impact about a subject.  A mutant is CAUGHT only
# if it COMPILED and the run ended with a failed check (exit 1 and a `FAIL` line) or a sanitizer report; a mutant that does not compile (BUILD_FAILURE), times out (TIMEOUT), ends otherwise (UNEXPECTED) or passes
# (PASSED) is a FAILURE of the controller, never a catch.  The unmutated copy must pass for each harness; a pattern that is not found is a failure.  NOT a native run.
set -u
if [ "$#" -ne 1 ]; then echo "usage: mutate-subject-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/m
fresh() { cp "$R/mb_hostaim.c" "$R/mb_hostaim.h" "$R/mb_hostobj.h" /tmp/m/ || { echo "SETUP_FAILURE: cannot copy the sources"; exit 3; }; }
# build_and_run <harness> <timeout seconds>: sets COMPILE_RC, RUN_RC and CLASS
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/forecast-stub" -I /tmp/m -o /tmp/m/mutant "$C/$1" /tmp/m/mb_hostaim.c -lm > /tmp/m/compile.log 2>&1
    COMPILE_RC=$?
    RUN_RC=-
    if [ "$COMPILE_RC" -ne 0 ]; then CLASS=BUILD_FAILURE; head -3 /tmp/m/compile.log; return; fi
    (ulimit -f 128; timeout -k 2s "$2" /tmp/m/mutant) > /tmp/m/mutant.log 2>&1
    RUN_RC=$?
    tail -1 /tmp/m/mutant.log | cut -c1-160
    if [ "$RUN_RC" -eq 124 ] || [ "$RUN_RC" -eq 137 ]; then CLASS=TIMEOUT
    elif [ "$RUN_RC" -eq 1 ] && grep -q "FAIL" /tmp/m/mutant.log; then CLASS=CAUGHT_ASSERTION
    elif grep -qE "ERROR: AddressSanitizer|runtime error:" /tmp/m/mutant.log; then CLASS=CAUGHT_SANITIZER
    elif [ "$RUN_RC" -eq 0 ]; then CLASS=PASSED
    else CLASS=UNEXPECTED; fi
}
controls_ok=0; controls_bad=0
for h in damage_checks.c health_checks.c blast_checks.c; do
    fresh
    echo "== control (unmutated copy) $h"
    build_and_run $h 60
    echo "CONTROL $h compile=$COMPILE_RC run=$RUN_RC class=$CLASS"
    if [ "$COMPILE_RC" -eq 0 ] && [ "$RUN_RC" -eq 0 ]; then controls_ok=$((controls_ok+1)); else controls_bad=$((controls_bad+1)); fi
done
caught=0; failed=0
mutate() {
    name=$1; harness=$2; pattern=$3; replacement=$4
    fresh
    sed -i "s#$pattern#$replacement#" /tmp/m/mb_hostaim.c
    if cmp -s "$R/mb_hostaim.c" /tmp/m/mb_hostaim.c; then echo "MUTANT $name: PATTERN NOT FOUND (a failure)"; failed=$((failed+1)); return; fi
    echo "== mutant: $name"
    build_and_run $harness 60
    case "$CLASS" in
        CAUGHT_ASSERTION|CAUGHT_SANITIZER) echo "MUTANT $name: CAUGHT class=$CLASS compile=$COMPILE_RC run=$RUN_RC"; caught=$((caught+1));;
        *) echo "MUTANT $name: FAILURE class=$CLASS compile=$COMPILE_RC run=$RUN_RC (not a catch)"; failed=$((failed+1));;
    esac
}
mutate damage_names_no_subject damage_checks.c "campaign_aim_identify (&step, NULL, subject);" "(void) subject;"
mutate health_names_no_subject health_checks.c "campaign_aim_identify (&step, NULL, subject);" "(void) subject;"
mutate impact_names_no_target blast_checks.c "campaign_aim_identify (&step, NULL, target);" "(void) target;"
mutate envelope_ignores_the_registry damage_checks.c "host_id = campaign_object_identify (en);" "host_id = NULL;"
mutate envelope_ignores_the_registry_for_impact blast_checks.c "host_id = campaign_object_identify (en);" "host_id = NULL;"
mutate envelope_length_unchecked health_checks.c "if (strlen (host_id) > CAMPAIGN_AIM_HOST_ID_MAX)" "if (0)"
echo "CAUGHT=$caught FAILED=$failed CONTROLS_OK=$controls_ok CONTROLS_BAD=$controls_bad"
if [ "$controls_ok" -ne 3 ] || [ "$controls_bad" -ne 0 ] || [ "$failed" -ne 0 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
