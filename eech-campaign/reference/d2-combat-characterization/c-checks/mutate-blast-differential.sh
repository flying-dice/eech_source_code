#!/bin/sh
# usage (inside the EECH-D2-Build distro): mutate-blast-differential.sh <logfile>: the mutation controls of blast_differential.c (EEDCS-46).  Each control changes ONE rule of a COPY of the real
# campaign driver (mb_hostaim.c) or of the reference kernel (cmb_step.c) and runs the same differential against the original: every control must make the paths differ, which is what
# shows the comparison would notice that rule being wrong.  The unmutated copies must pass.  Statuses are captured and propagated.
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
build_and_run() {
    gcc -std=gnu99 -w $san -fno-pie -no-pie -I "$C/forecast-stub" -I /tmp/m -I "$K" -o /tmp/m/diff "$C/blast_differential.c" /tmp/m/mb_hostaim.c /tmp/m/cmb_step.c -lm || return 99
    (ulimit -f 128; timeout -k 2s 300s /tmp/m/diff) > /tmp/m/diff.log 2>&1
    run=$?
    tail -2 /tmp/m/diff.log | head -1 | cut -c1-200
    return $run
}
fresh() { cp "$R/mb_hostaim.c" "$R/mb_hostaim.h" "$K/cmb_step.c" /tmp/m/; }
fresh
echo "== control (unmutated copies)"
build_and_run
control=$?
echo "CONTROL_EXIT=$control"
caught=0; missed=0
mutate() {
    name=$1; file=$2; pattern=$3; replacement=$4
    fresh
    sed -i "s#$pattern#$replacement#" "/tmp/m/$file"
    case "$file" in mb_hostaim.c) orig="$R/mb_hostaim.c";; *) orig="$K/cmb_step.c";; esac
    if cmp -s "$orig" "/tmp/m/$file"; then echo "MUTANT $name: PATTERN NOT FOUND"; missed=$((missed+1)); return; fi
    echo "== mutant: $name"
    build_and_run
    rc=$?
    if [ "$rc" -ne 0 ]; then echo "MUTANT $name: CAUGHT (exit $rc)"; caught=$((caught+1)); else echo "MUTANT $name: MISSED"; missed=$((missed+1)); fi
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
echo "CAUGHT=$caught MISSED=$missed"
if [ "$control" -ne 0 ] || [ "$missed" -ne 0 ]; then exit 1; fi
exit 0
' > "$log" 2>&1
docker_exit=$?
echo "DOCKER_EXIT=$docker_exit" >> "$log"
exit "$docker_exit"
