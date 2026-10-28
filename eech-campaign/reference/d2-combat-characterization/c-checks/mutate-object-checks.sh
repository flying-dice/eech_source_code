#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-object-checks.sh <logfile>: the mutation controls of object_checks.c (EEDCS-46, R3).  Each control removes ONE rule of a COPY of the real mb_hostobj.c or
# mb_hostaim.c (the checkout is mounted read-only) and runs the same checks against it.  A mutant is CAUGHT only if it COMPILED and the run ended with the intended outcome: a failed check (exit 1 and a
# `FAIL` line) or a sanitizer report.  A mutant that does not compile (BUILD_FAILURE), times out (TIMEOUT), ends otherwise (UNEXPECTED) or passes (PASSED) is a FAILURE of the controller, never a catch.
# Two setup controls prove that: a copy that cannot compile and a copy that never ends must be classified BUILD_FAILURE and TIMEOUT, and are counted apart.  The unmutated copy must pass.
set -u
if [ "$#" -ne 1 ]; then echo "usage: mutate-object-checks.sh <logfile>" >&2; exit 2; fi
log=$1
root=${EECH_CHECKOUT:-/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections}
cd "$root" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/m
fresh() { cp "$R/mb_hostaim.c" "$R/mb_hostaim.h" "$R/mb_hostobj.c" "$R/mb_hostobj.h" /tmp/m/ || { echo "SETUP_FAILURE: cannot copy the sources"; exit 3; }; }
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/object-stub" -I /tmp/m -I /src/aphavoc/source -o /tmp/m/mutant "$C/object_checks.c" /tmp/m/mb_hostaim.c /tmp/m/mb_hostobj.c -lm > /tmp/m/compile.log 2>&1
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
    name=$1; file=$2; pattern=$3; replacement=$4
    fresh
    sed -i "s#$pattern#$replacement#" "/tmp/m/$file"
    if cmp -s "$R/$file" "/tmp/m/$file"; then echo "MUTANT $name: PATTERN NOT FOUND (a failure)"; failed=$((failed+1)); return; fi
    echo "== mutant: $name"
    build_and_run 60
    case "$CLASS" in
        CAUGHT_ASSERTION|CAUGHT_SANITIZER) echo "MUTANT $name: CAUGHT class=$CLASS compile=$COMPILE_RC run=$RUN_RC"; caught=$((caught+1));;
        *) echo "MUTANT $name: FAILURE class=$CLASS compile=$COMPILE_RC run=$RUN_RC (not a catch)"; failed=$((failed+1));;
    esac
}
setup_control() {
    name=$1; expected=$2; seconds=$3; pattern=$4; replacement=$5
    fresh
    sed -i "s@$pattern@$replacement@" /tmp/m/mb_hostobj.c
    if cmp -s "$R/mb_hostobj.c" /tmp/m/mb_hostobj.c; then echo "SETUP CONTROL $name: PATTERN NOT FOUND (a failure)"; controls_bad=$((controls_bad+1)); return; fi
    echo "== setup control: $name"
    build_and_run "$seconds"
    if [ "$CLASS" = "$expected" ]; then echo "SETUP CONTROL $name: class=$CLASS as expected (compile=$COMPILE_RC run=$RUN_RC), not a catch"; controls_ok=$((controls_ok+1)); else echo "SETUP CONTROL $name: class=$CLASS, expected $expected"; controls_bad=$((controls_bad+1)); fi
}
mutate identity_ends_with_the_entity mb_hostobj.c "records\[record\].entity_index = -1;" "records[record].entity_index = records[record].entity_index;"
mutate slot_forgets_the_identity mb_hostobj.c "record_of_entity\[index\] = 0;" "record_of_entity[index] = record_of_entity[index];"
mutate no_duplicate_identity mb_hostobj.c "if (find_record (id) >= 0)" "if (0)"
mutate no_roster_collision mb_hostobj.c "if (campaign_roster_find_host_id (id, NULL) != 0)" "if (0)"
mutate no_control_characters mb_hostobj.c "if ((\*p < 0x20) || (\*p == 0x7f))" "if (0)"
mutate fact_applied_once mb_hostobj.c "if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))" "if (0)"
mutate no_repair_by_report mb_hostobj.c "(report->damage_level < 0) || (report->damage_level > level_before)" "(report->damage_level < 0)"
mutate no_negative_level mb_hostobj.c "(report->damage_level < 0) || (report->damage_level > level_before)" "(report->damage_level > level_before)"
mutate server_only mb_hostobj.c "if (get_comms_model () != COMMS_MODEL_SERVER)" "if (0)"
mutate zero_level_kills mb_hostobj.c "(report->has_damage_level \&\& (level_after <= 0))" "0"
mutate destruction_leaves_level mb_hostobj.c "if (level_after != level_before)" "if (1)"
mutate stale_roster_identity mb_hostobj.c "case 2:" "case 3:"
mutate identity_issued_once mb_hostobj.c "if (id)\$" "if (0)"
mutate empty_report mb_hostobj.c "(!report->has_damage_level \&\& !report->destroyed)" "0"
mutate identity_needs_a_body mb_hostaim.c "|| (id\[0\] == " "|| (0 \&\& id[0] == "
setup_control cannot_compile BUILD_FAILURE 60 "#include \"project.h\"" "#include \"project.h\"\n#error a copy that cannot compile"
setup_control never_ends TIMEOUT 5 "free (records);" "free (records); { volatile int spin = 1; while (spin) { } }"
echo "CAUGHT=$caught FAILED=$failed SETUP_CONTROLS_AS_EXPECTED=$controls_ok SETUP_CONTROLS_WRONG=$controls_bad CONTROL_OK=$control_ok"
if [ "$control_ok" -ne 1 ] || [ "$failed" -ne 0 ] || [ "$controls_bad" -ne 0 ] || [ "$controls_ok" -ne 2 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
