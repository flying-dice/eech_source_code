#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-bridge-checks.sh <logfile>: the mutation controls of bridge_checks.c (EEDCS-46, R5).  Each control removes ONE refusal or ordering rule of a COPY of the
# real mb_hostaim.c (the checkout is mounted read-only) and runs the same checks against it.  A mutant is CAUGHT only if it COMPILED and the run ended with the intended outcome: a failed
# check (exit 1 and a `FAIL` line) or a sanitizer report.  A mutant that does not compile (BUILD_FAILURE), times out (TIMEOUT), ends otherwise (UNEXPECTED) or passes (PASSED) is a FAILURE
# of the controller, never a catch.  Two setup controls prove that: a copy that cannot compile and a copy that never ends must be classified BUILD_FAILURE and TIMEOUT, and are counted apart.
# The unmutated copy must pass.  Compile and run statuses are recorded separately and the script exits 0 only if everything above holds.
set -u
if [ "$#" -ne 1 ]; then echo "usage: mutate-bridge-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/m
fresh() { cp "$R/mb_hostaim.c" "$R/mb_hostaim.h" /tmp/m/ || { echo "SETUP_FAILURE: cannot copy the sources"; exit 3; }; }
# build_and_run <run timeout seconds>: sets COMPILE_RC, RUN_RC and CLASS
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/forecast-stub" -I /tmp/m -o /tmp/m/mutant "$C/bridge_checks.c" /tmp/m/mb_hostaim.c -lm > /tmp/m/compile.log 2>&1
    COMPILE_RC=$?
    RUN_RC=-
    if [ "$COMPILE_RC" -ne 0 ]; then CLASS=BUILD_FAILURE; head -3 /tmp/m/compile.log; return; fi
    (ulimit -f 128; timeout -k 2s "$1" /tmp/m/mutant) > /tmp/m/mutant.log 2>&1
    RUN_RC=$?
    tail -1 /tmp/m/mutant.log | cut -c1-160
    if [ "$RUN_RC" -eq 124 ] || [ "$RUN_RC" -eq 137 ]; then CLASS=TIMEOUT
    elif [ "$RUN_RC" -eq 1 ] && grep -q "^FAIL" /tmp/m/mutant.log; then CLASS=CAUGHT_ASSERTION
    elif grep -qE "ERROR: AddressSanitizer|runtime error:" /tmp/m/mutant.log; then CLASS=CAUGHT_SANITIZER
    elif [ "$RUN_RC" -eq 0 ]; then CLASS=PASSED
    else CLASS=UNEXPECTED; fi
}
fresh
echo "== control (unmutated copy)"
build_and_run 60
echo "CONTROL compile=$COMPILE_RC run=$RUN_RC class=$CLASS"
control_ok=0; [ "$COMPILE_RC" -eq 0 ] && [ "$RUN_RC" -eq 0 ] && control_ok=1
caught=0; failed=0; controls_ok=0; controls_bad=0
mutate() {
    name=$1; pattern=$2; replacement=$3
    fresh
    sed -i "s#$pattern#$replacement#" /tmp/m/mb_hostaim.c
    if cmp -s "$R/mb_hostaim.c" /tmp/m/mb_hostaim.c; then echo "MUTANT $name: PATTERN NOT FOUND (a failure)"; failed=$((failed+1)); return; fi
    echo "== mutant: $name"
    build_and_run 60
    case "$CLASS" in
        CAUGHT_ASSERTION|CAUGHT_SANITIZER) echo "MUTANT $name: CAUGHT class=$CLASS compile=$COMPILE_RC run=$RUN_RC"; caught=$((caught+1));;
        *) echo "MUTANT $name: FAILURE class=$CLASS compile=$COMPILE_RC run=$RUN_RC (not a catch)"; failed=$((failed+1));;
    esac
}
# a setup control must be classified as the failure it is, and is never counted as a catch
setup_control() {
    name=$1; expected=$2; seconds=$3; pattern=$4; replacement=$5
    fresh
    sed -i "s@$pattern@$replacement@" /tmp/m/mb_hostaim.c
    if cmp -s "$R/mb_hostaim.c" /tmp/m/mb_hostaim.c; then echo "SETUP CONTROL $name: PATTERN NOT FOUND (a failure)"; controls_bad=$((controls_bad+1)); return; fi
    echo "== setup control: $name"
    build_and_run "$seconds"
    if [ "$CLASS" = "$expected" ]; then echo "SETUP CONTROL $name: class=$CLASS as expected (compile=$COMPILE_RC run=$RUN_RC), not a catch"; controls_ok=$((controls_ok+1)); else echo "SETUP CONTROL $name: class=$CLASS, expected $expected"; controls_bad=$((controls_bad+1)); fi
}
mutate kill_needs_alive_and_server "(answer->kill \&\& (!request->alive || !request->server))" "0"
mutate neighbour_must_be_present "(answer->notify \&\& !request->neighbour_present)" "0"
mutate variant_range "(answer->variant >= CAMPAIGN_BRIDGE_VARIANTS)" "0"
mutate landed_needs_fell "(answer->landed \&\& !answer->fell)" "0"
mutate terrain_asked_once "(answer->need_terrain != 1) || request->terrain_known" "(answer->need_terrain != 1)"
mutate vehicle_must_be_living "!step.bridge.window\[index\].readable || !step.bridge.window\[index\].alive" "0"
mutate vehicles_sector_started_once "if ((visited_sx \[i\] == sx) \&\& (visited_sz \[i\] == sz))" "if (0)"
mutate vehicles_sector_bound "if (visited_count >= CAMPAIGN_BLAST_SECTORS_MAX)" "if (0)"
mutate vehicles_continuation_after "(after_handle != step.bridge.window\[step.bridge.window_count - 1\].handle)" "0"
mutate fall_figures_finite "!isfinite (answer->dropped_y) ||" ""
mutate rest_living_segment_not_moved "(answer->landed \&\& request->alive)" "0"
mutate rest_landed_height_agrees "(!answer->landed \&\& answer->landed_y != 0.0f) || (answer->landed \&\& request->alive)" "(answer->landed \&\& request->alive)"
mutate vehicles_destructive_target_guard "if (walk->resumable \&\& !walk->resumable (walk->context, object))" "if (0)"
mutate identity_of_the_object_named "(strcmp (offered, claimed) != 0)" "0"
mutate identity_beside_no_object "(answer->target_host_id\[0\] \&\& ((request->op != CAMPAIGN_BRIDGE_VEHICLES) || !answer->action))" "0"
setup_control cannot_compile BUILD_FAILURE 60 "#include <limits.h>" "#include <limits.h>\n#error a copy that cannot compile"
setup_control never_ends TIMEOUT 5 "step.mode = CAMPAIGN_AIM_BRIDGE;" "step.mode = CAMPAIGN_AIM_BRIDGE; { volatile int spin = 1; while (spin) { } }"
echo "CAUGHT=$caught FAILED=$failed SETUP_CONTROLS_AS_EXPECTED=$controls_ok SETUP_CONTROLS_WRONG=$controls_bad CONTROL_OK=$control_ok"
if [ "$control_ok" -ne 1 ] || [ "$failed" -ne 0 ] || [ "$controls_bad" -ne 0 ] || [ "$controls_ok" -ne 2 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
