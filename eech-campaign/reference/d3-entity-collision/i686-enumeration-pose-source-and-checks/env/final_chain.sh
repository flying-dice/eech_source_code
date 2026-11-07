#!/bin/sh
# final checks on the clean committed code 681ccfe54: map before the FIRST check, ONE scoped runner child at a time (sole cache owner), map after; stdin closed
ENV=/r/d3-evidence/i686-enumeration-pose-20261106-env
cd $ENV || exit 9
P=/c/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe
REPO=R:/d3-evidence/collision-characterization-20261029
HEAD=e442946e5589ae9b91a1522bf4b35afec490cc63
MAP=/r/d3-evidence/sector-roster-inputmap-batched-20261106.py
RUNNER=R:/d3-evidence/run-d3-pose-focused-direct-20261106.py
run() { label=$1; shift; "$P" -I -B -X utf8 $RUNNER "$label" "$@" < /dev/null >/dev/null 2>&1; echo "$label exit=$?" >> chain-final.progress; }
git -C $REPO rev-parse HEAD > final-head-before.txt
git -C $REPO status --short > final-status-before.txt
"$P" -I -B -X utf8 $MAP $REPO R:/d3-evidence/i686-enumeration-pose-inputmap-before-20261106.json $HEAD < /dev/null > inputmap-before.stdout.log 2> inputmap-before.stderr.log
echo "map-before exit=$?" > chain-final.progress
run pose-final-i686-default 'cargo test --locked --target i686-unknown-linux-gnu -p eech-world --test entity_candidate_host'
run pose-final-i686-stackattr 'env EECH_ORIGINAL_STACK_ATTRIBUTES=1 cargo test --locked --target i686-unknown-linux-gnu -p eech-world --test entity_candidate_host'
run pose-final-x64-default 'cargo test --locked --target x86_64-unknown-linux-gnu -p eech-world --test entity_candidate_host'
run pose-final-fmt-check 'cargo fmt -p eech-world -- --check'
run pose-final-clippy 'cargo clippy --locked --all-targets -p eech-world -- -D warnings'
"$P" -I -B -X utf8 $MAP $REPO R:/d3-evidence/i686-enumeration-pose-inputmap-after-20261106.json $HEAD < /dev/null > inputmap-after.stdout.log 2> inputmap-after.stderr.log
echo "map-after exit=$?" >> chain-final.progress
git -C $REPO rev-parse HEAD > final-head-after.txt
git -C $REPO status --short > final-status-after.txt
echo done > chain-final.marker
