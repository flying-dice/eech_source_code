#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-hostaim-validation.sh <logfile>
# Compiles and runs hostaim_validation.c against real mb_hostaim.c. An optional
# EECH_HOSTAIM_VALIDATION_CC override supports isolated failure-status probes.
set -u
if [ "$#" -ne 1 ]; then
    echo "usage: run-hostaim-validation.sh <logfile>" >&2
    exit 2
fi
log=$1
validation_cc=${EECH_HOSTAIM_VALIDATION_CC:-gcc}
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code" || exit 2
docker run --rm -v "$PWD:/src:ro" -e EECH_HOSTAIM_VALIDATION_CC="$validation_cc" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-weapon-characterization/c-checks
"$EECH_HOSTAIM_VALIDATION_CC" -std=gnu99 -Wall -Wno-unused-function -I "$C/stub" -I "$R" -o /tmp/hostaim_validation "$C/hostaim_validation.c" "$R/mb_hostaim.c" -lm
compile_exit=$?
printf "COMPILE_EXIT=%s\n" "$compile_exit"
if [ "$compile_exit" -ne 0 ]; then exit "$compile_exit"; fi
/tmp/hostaim_validation
run_exit=$?
printf "RUN_EXIT=%s\n" "$run_exit"
exit "$run_exit"
' > "$log" 2>&1
docker_exit=$?
printf "DOCKER_EXIT=%s\n" "$docker_exit" >> "$log"
exit "$docker_exit"
