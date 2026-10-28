#!/bin/sh
# usage (inside EECH-D2-Build): case-mutation-450.sh <logfile>: the SAME case_checks.c against the PREVIOUS driver (450ecf60a, a copy kept in the private tree), same test-only flags.
# Expected: failures (the checks are about the guards 450 lacked).  A control, never a product build.
log=$1
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections' || exit 2
docker run --rm -v "$PWD:/src:ro" -v '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/weapon-characterization-20261027:/ev:ro' -w /tmp eech-build sh -c '
C=/src/eech-campaign/reference/d2-combat-characterization/c-checks
san="-fsanitize=address,undefined -fno-sanitize-recover=undefined"
mkdir -p /tmp/cs && cp /ev/eech_cmb_case.450.c /tmp/cs/eech_cmb_case.c && cp "$C/case_checks.c" /tmp/cs/case_checks.c
gcc -std=gnu99 -Wall -Wno-unused-parameter -DEECH_WEAPON_FIXTURE $san -fno-pie -no-pie -I "$C/case-stub" -o /tmp/case_mutant /tmp/cs/case_checks.c /tmp/cs/eech_cmb_case.c -lm
echo "COMPILE_EXIT=$?"
(ulimit -f 128; timeout -k 2s 120s /tmp/case_mutant) > /tmp/mutant.log 2>&1
run=$?
cat /tmp/mutant.log
echo "RUN_EXIT=$run"
exit "$run"
' > "$log" 2>&1
echo "DOCKER_EXIT=$?" >> "$log"
