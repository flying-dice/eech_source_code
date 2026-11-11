#!/bin/sh
# Diagnostic only: the same scratch control compiled for 32-bit with SSE float math (flags NOT those of production), against the x64 words already written by pose-control.sh.
set -u
cd /work
INC="-I/orig/eech-campaign/crates/eech-world/csrc"
COMMON="-std=gnu99 -w -fno-strict-aliasing -fwrapv -ftrivial-auto-var-init=zero"
run() { label=$1; shift; gcc-12 "$@" $COMMON pose_main.c -lm -o pose_$label 2> pose_$label.cc.err && ./pose_$label 2000 > pose_$label.out; echo "$label exit=$? lines=$(wc -l < pose_$label.out)"; }
run i686_O0_sse_original -m32 -O0 -msse2 -mfpmath=sse
run i686_O1_sse_original -m32 -O1 -msse2 -mfpmath=sse
run i686_O1_sse_production -m32 -O1 -msse2 -mfpmath=sse -DPRODUCTION_TEXT $INC
run i686_O2_sse_production -m32 -O2 -msse2 -mfpmath=sse -DPRODUCTION_TEXT $INC
for f in pose_i686_*sse*.out; do
  cut -d' ' -f5-13 pose_x64_O1_original.out > /tmp/out_ref; cut -d' ' -f5-13 $f > /tmp/out_f
  echo "words vs x64_O1_original: $f output_lines_differing=$(diff /tmp/out_ref /tmp/out_f | grep -c '^<')"
done
