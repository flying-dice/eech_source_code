#!/bin/sh
# usage (inside the EECH-D2-Build distro): run-hostaim-validation.sh <logfile>: compiles and runs hostaim_validation.c against the real mb_hostaim.c in a disposable container
log=$1
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code" || exit 2
docker run --rm -v "$PWD:/src:ro" -w /tmp eech-build sh -c '
R=/src/aphavoc/source/entity/mobile
C=/src/eech-campaign/reference/d2-weapon-characterization/c-checks
gcc -std=gnu99 -Wall -Wno-unused-function -I "$C/stub" -I "$R" -o /tmp/hostaim_validation "$C/hostaim_validation.c" "$R/mb_hostaim.c" -lm
echo COMPILE_EXIT=$?
/tmp/hostaim_validation
echo RUN_EXIT=$?
' > "$log" 2>&1
echo DOCKER_EXIT=$? >> "$log"
