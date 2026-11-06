#!/bin/sh
# Compiles tree_step.c with the flags of crates/eech-world/build.rs for 32-bit x86 (gcc-12, -m32) and counts the x87 float loads/stores in begin_body and finish's inlined copies.
# arg1: source dir (csrc) arg2: label. Runs inside the task-local image; read-only source.
set -u
src="$1"; label="$2"
for o in 1 2; do
  gcc-12 -m32 -O$o -g -std=gnu99 -w -fno-strict-aliasing -fwrapv -ftrivial-auto-var-init=zero -fPIC -c "$src/tree_step.c" -o /tmp/t$o.o || { echo "compile failed O$o"; continue; }
  echo "== $label -O$o: x87 flds/fstps/fsts in begin_body:"
  objdump -d --no-show-raw-insn /tmp/t$o.o | awk '/<begin_body>:/{f=1} f{print} /^$/{if(f)exit}' | grep -c "flds\|fstps\|fsts"
  echo "== $label -O$o: x87 flds/fstps/fsts in the whole object:"
  objdump -d --no-show-raw-insn /tmp/t$o.o | grep -c "flds\|fstps\|fsts"
done
