#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-blast-differential.sh <logfile>: the mutation controls of blast_differential.c (EEDCS-46).  Each control changes ONE rule of a COPY of the real
# campaign driver (mb_hostaim.c) or of the reference kernel (cmb_step.c) and runs the same differential against the original.  A mutant is CAUGHT only if it COMPILED and the run ended with the
# intended outcome: the paths differ (exit 1 and a `FAIL` line) or a sanitizer reports.  A mutant that does not compile (BUILD_FAILURE), times out (TIMEOUT), ends otherwise (UNEXPECTED) or
# passes (PASSED) is a FAILURE of the controller, never a catch.  Two setup controls prove that: a copy that cannot compile and a copy that never ends must be classified BUILD_FAILURE and
# TIMEOUT, and are counted apart.  The unmutated copies must pass.  Compile and run statuses are recorded separately; the script exits 0 only if everything above holds.
set -u
if [ "$#" -ne 1 ]; then echo "usage: mutate-blast-differential.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
K=/src/eech-campaign/crates/eech-world/csrc
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/m
fresh() { cp "$R/mb_hostaim.c" "$R/mb_hostaim.h" "$K/cmb_step.c" /tmp/m/ || { echo "SETUP_FAILURE: cannot copy the sources"; exit 3; }; }
# build_and_run <run timeout seconds>: sets COMPILE_RC, RUN_RC and CLASS
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/forecast-stub" -I /tmp/m -I "$K" -o /tmp/m/diff "$C/blast_differential.c" /tmp/m/mb_hostaim.c /tmp/m/cmb_step.c -lm > /tmp/m/compile.log 2>&1
    COMPILE_RC=$?
    RUN_RC=-
    if [ "$COMPILE_RC" -ne 0 ]; then CLASS=BUILD_FAILURE; head -3 /tmp/m/compile.log; return; fi
    (ulimit -f 128; timeout -k 2s "$1" /tmp/m/diff) > /tmp/m/diff.log 2>&1
    RUN_RC=$?
    tail -2 /tmp/m/diff.log | head -1 | cut -c1-200
    if [ "$RUN_RC" -eq 124 ] || [ "$RUN_RC" -eq 137 ]; then CLASS=TIMEOUT
    elif [ "$RUN_RC" -eq 1 ] && grep -q "^FAIL" /tmp/m/diff.log; then CLASS=CAUGHT_ASSERTION
    elif grep -qE "ERROR: AddressSanitizer|runtime error:" /tmp/m/diff.log; then CLASS=CAUGHT_SANITIZER
    elif [ "$RUN_RC" -eq 0 ]; then CLASS=PASSED
    else CLASS=UNEXPECTED; fi
}
fresh
echo "== control (unmutated copies)"
build_and_run 300
echo "CONTROL compile=$COMPILE_RC run=$RUN_RC class=$CLASS"
control_ok=0; [ "$COMPILE_RC" -eq 0 ] && [ "$RUN_RC" -eq 0 ] && control_ok=1
caught=0; failed=0; controls_ok=0; controls_bad=0
mutate() {
    name=$1; file=$2; pattern=$3; replacement=$4
    fresh
    sed -i "s#$pattern#$replacement#" "/tmp/m/$file"
    case "$file" in mb_hostaim.c) orig="$R/mb_hostaim.c";; *) orig="$K/cmb_step.c";; esac
    if cmp -s "$orig" "/tmp/m/$file"; then echo "MUTANT $name: PATTERN NOT FOUND (a failure)"; failed=$((failed+1)); return; fi
    echo "== mutant: $name"
    build_and_run 300
    case "$CLASS" in
        CAUGHT_ASSERTION|CAUGHT_SANITIZER) echo "MUTANT $name: CAUGHT class=$CLASS compile=$COMPILE_RC run=$RUN_RC"; caught=$((caught+1));;
        *) echo "MUTANT $name: FAILURE class=$CLASS compile=$COMPILE_RC run=$RUN_RC (not a catch)"; failed=$((failed+1));;
    esac
}
# a setup control must be classified as the failure it is, and is never counted as a catch
setup_control() {
    name=$1; expected=$2; seconds=$3; file=$4; pattern=$5; replacement=$6
    fresh
    sed -i "s@$pattern@$replacement@" "/tmp/m/$file"
    case "$file" in mb_hostaim.c) orig="$R/mb_hostaim.c";; *) orig="$K/cmb_step.c";; esac
    if cmp -s "$orig" "/tmp/m/$file"; then echo "SETUP CONTROL $name: PATTERN NOT FOUND (a failure)"; controls_bad=$((controls_bad+1)); return; fi
    echo "== setup control: $name"
    build_and_run "$seconds"
    if [ "$CLASS" = "$expected" ]; then echo "SETUP CONTROL $name: class=$CLASS as expected (compile=$COMPILE_RC run=$RUN_RC), not a catch"; controls_ok=$((controls_ok+1)); else echo "SETUP CONTROL $name: class=$CLASS, expected $expected"; controls_bad=$((controls_bad+1)); fi
}
mutate in_range_is_inclusive cmb_step.c "in_range = (range \&\& (range < damage_radius)) ? 1 : 0;" "in_range = (range \&\& (range <= damage_radius)) ? 1 : 0;"
mutate zero_range_counts cmb_step.c "in_range = (range \&\& (range < damage_radius)) ? 1 : 0;" "in_range = (range < damage_radius) ? 1 : 0;"
mutate rocket_threshold cmb_step.c "> 0.75 \&\& object->rocket" "> 0.70 \&\& object->rocket"
mutate rocket_needs_no_flag cmb_step.c "> 0.75 \&\& object->rocket" "> 0.75"
mutate target_not_skipped cmb_step.c "if (object->excluded) // skip target" "if (0) // skip target"
mutate dead_objects_are_candidates cmb_step.c "|| !object->alive || !object->has_position" "|| !object->has_position"
mutate sector_rows_wrong_order cmb_step.c "out->need_sz = in->window_sz + 1;" "out->need_sz = in->window_sz + 1; out->need_sx = max_sector_x;"
mutate sector_rect_not_bounded cmb_step.c "min_sector_x = bound(min_sector_x, in->min_x_sector, in->max_x_sector);" "min_sector_x = min_sector_x;"
mutate weapon_sent_collision cmb_step.c "if (object->type != CF_ET_WEAPON)" "if (1)"
mutate successor_after_stale_window mb_hostaim.c "step.blast.window_sx, step.blast.window_sz, object, handle);" "step.blast.window_sx, step.blast.window_sz, objects[step.blast.window_count - 1], step.blast.window[step.blast.window_count - 1].handle);"
setup_control cannot_compile BUILD_FAILURE 300 mb_hostaim.c "#include <limits.h>" "#include <limits.h>\n#error a copy that cannot compile"
setup_control never_ends TIMEOUT 5 mb_hostaim.c "step.mode = CAMPAIGN_AIM_BLAST;" "step.mode = CAMPAIGN_AIM_BLAST; { volatile int spin = 1; while (spin) { } }"
echo "CAUGHT=$caught FAILED=$failed SETUP_CONTROLS_AS_EXPECTED=$controls_ok SETUP_CONTROLS_WRONG=$controls_bad CONTROL_OK=$control_ok"
if [ "$control_ok" -ne 1 ] || [ "$failed" -ne 0 ] || [ "$controls_bad" -ne 0 ] || [ "$controls_ok" -ne 2 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
