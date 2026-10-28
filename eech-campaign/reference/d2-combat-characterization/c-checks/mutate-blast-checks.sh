#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-blast-checks.sh <logfile>: the mutation controls of blast_checks.c (EEDCS-46).  Each control removes ONE refusal or ordering rule of a COPY of the
# real mb_hostaim.c (the checkout is mounted read-only) and runs the same checks against it: every control must make at least one check fail, which is what shows the checks exercise
# that rule rather than pass whatever the campaign does.  Statuses are captured and propagated: the script exits 0 only if every mutant was caught (and the unmutated copy passed).
set -u
if [ "$#" -ne 1 ]; then echo "usage: mutate-blast-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
cp "$R/mb_hostaim.c" /tmp/mb_hostaim.c
cp "$R/mb_hostaim.h" /tmp/mb_hostaim.h
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/forecast-stub" -I /tmp -o /tmp/mutant "$C/blast_checks.c" /tmp/mb_hostaim.c -lm || return 99
    (ulimit -f 128; timeout -k 2s 60s /tmp/mutant) > /tmp/mutant.log 2>&1
    run=$?
    tail -1 /tmp/mutant.log
    return $run
}
echo "== control (unmutated copy)"
build_and_run
control=$?
echo "CONTROL_EXIT=$control"
caught=0; missed=0
mutate() {
    name=$1; pattern=$2; replacement=$3
    cp "$R/mb_hostaim.c" /tmp/mb_hostaim.c
    sed -i "s#$pattern#$replacement#" /tmp/mb_hostaim.c
    if cmp -s "$R/mb_hostaim.c" /tmp/mb_hostaim.c; then echo "MUTANT $name: PATTERN NOT FOUND"; missed=$((missed+1)); return; fi
    echo "== mutant: $name"
    build_and_run
    rc=$?
    if [ "$rc" -ne 0 ]; then echo "MUTANT $name: CAUGHT (exit $rc)"; caught=$((caught+1)); else echo "MUTANT $name: MISSED"; missed=$((missed+1)); fi
}
mutate continuation_after "(after_handle != step.blast.window\[step.blast.window_count - 1\].handle)" "0"
mutate modifier_upper_bound "(answer->modifier > 1.0f)" "(answer->modifier > 100.0f)"
mutate radius_must_not_change "else if (result.blast.radius != radius)" "else if (0)"
mutate crater_needs_terrain "(plan->crater \&\& !step->impact.terrain_known)" "0"
mutate sector_order "((sz == last_sz) \&\& (sx <= last_sx))" "0"
mutate blasts_at_most_two "(plan->blasts > 2)" "(plan->blasts > 9)"
mutate direct_hit_needs_target "(plan->direct_hit \&\& !step->impact.has_target)" "0"
mutate window_sector_in_map "(answer->need_sx > blast->max_x_sector)" "0"
mutate rocket_must_be_known "if ((result.blast.action == CAMPAIGN_BLAST_INTERCEPT) \&\& !step.blast.window\[index\].rocket)" "if (0)"
mutate terrain_asked_once "|| step->impact.terrain_known || plan->invalid" "|| plan->invalid"
echo "CAUGHT=$caught MISSED=$missed"
if [ "$control" -ne 0 ] || [ "$missed" -ne 0 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
