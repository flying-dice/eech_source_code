#!/bin/sh
# EEDCS-43 review finding P1: does an edit under the original EECH tree (aphavoc/, modules/) reach a REUSED cargo target without cleaning?
# Runs in a disposable eech-build container on a private scratch copy of the tree (the worktree is never edited), with the OLD and then the
# NEW build/main.rs, on one cargo target volume that is never cleaned. usage: p1-incremental-check.sh <log>
LOG="$1"
W="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-sensor-detections"
LD="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/sensor-characterization-baseline-20261026/logs"
docker run --rm -v "$W:/src:ro" -v "$LD:/logs:ro" -v eech-d2-sensor-p1-target:/target -v eech-cargo:/usr/local/cargo/registry -e CARGO_TARGET_DIR=/target eech-build sh -c '
set -u
rm -rf /scratch && mkdir -p /scratch/ec
cp -r /src/modules /src/aphavoc /scratch/
cd /src/eech-campaign && tar cf - --exclude=./target --exclude=./reference . | (cd /scratch/ec && tar xf -)
cd /scratch/ec
export EECH_SOURCE_ROOT=/scratch
SRC=/scratch/aphavoc/source/entity/mobile/mb_hostsense.c
HDR=/scratch/aphavoc/source/entity/mobile/mb_hostsense.h
TXT=/scratch/modules/3d/textname.h
MAIN=crates/eech-engine-sys/build/main.rs
staged() { f=$(ls -t /target/debug/build/eech-engine-sys-*/out/tree/$1 2>/dev/null | head -1); sha256sum "$f" | cut -c1-16; }
run() { cargo check --locked -p eech-engine-sys > /tmp/c.out 2>&1; rc=$?; if grep -q "Compiling eech-engine-sys" /tmp/c.out; then r=RERAN; else r=not-rerun; fi; echo "$1: cargo exit $rc, build script $r"; }
echo "scratch tree: $(sha256sum $SRC | cut -c1-16) mb_hostsense.c at start"
echo "== OLD build/main.rs (the reviewed 37989fa5f)"
cp /logs/main.rs.p1-old $MAIN
run "A1 first build"
echo "   staged mb_hostsense.c $(staged aphavoc/source/entity/mobile/mb_hostsense.c)  scratch $(sha256sum $SRC | cut -c1-16)"
printf "\n/* edit A: an original-input edit only */\n" >> $SRC
run "A2 after editing the original mb_hostsense.c"
echo "   staged mb_hostsense.c $(staged aphavoc/source/entity/mobile/mb_hostsense.c)  scratch $(sha256sum $SRC | cut -c1-16)   (equal = the edit reached the build)"
echo "== NEW build/main.rs"
cp /logs/main.rs.p1-new $MAIN
run "B1 first build with the new script"
echo "   staged mb_hostsense.c $(staged aphavoc/source/entity/mobile/mb_hostsense.c)  scratch $(sha256sum $SRC | cut -c1-16)"
run "B2 nothing changed"
printf "\n/* edit B */\n" >> $SRC
run "B3 after editing the original mb_hostsense.c"
echo "   staged mb_hostsense.c $(staged aphavoc/source/entity/mobile/mb_hostsense.c)  scratch $(sha256sum $SRC | cut -c1-16)"
printf "\n/* edit C */\n" >> $HDR
run "B4 after editing the original header mb_hostsense.h"
echo "   staged mb_hostsense.h $(staged aphavoc/source/entity/mobile/mb_hostsense.h)  scratch $(sha256sum $HDR | cut -c1-16)"
printf "\n/* edit D */\n" >> $TXT
run "B5 after editing modules/3d/textname.h (feeds the generated texture names)"
echo "   staged textname.h $(staged modules/3d/textname.h)  scratch $(sha256sum $TXT | cut -c1-16)"
run "B6 nothing changed again"
' > "$LOG" 2>&1
echo "DOCKER_EXIT=$?" >> "$LOG"
