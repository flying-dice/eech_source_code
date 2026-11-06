#!/bin/sh
# Compiles and runs the scratch control in the task-local image. /orig = the repository root (read-only), /work = this directory (output).
set -u
cd /work
ORIG=/orig/modules/maths
{
 awk '/^void get_identity_matrix3x3 *\(/,/^}/' $ORIG/matrix.c
 awk '/^float normalise_any_3d_vector *\(/,/^}/' $ORIG/vector.c
 echo 'float normalise_3d_vector(vec3d *vector) { return normalise_any_3d_vector(vector); }'
 awk '/^void get_3d_vector_cross_product *\(/,/^}/' $ORIG/vector.c
 awk '/^void get_3d_transformation_matrix_from_face_normal_and_heading *\(/,/^}/' $ORIG/matrix.c
} > original_extract.h
sha256sum original_extract.h > original_extract.sha256
INC="-I/orig/eech-campaign/crates/eech-world/csrc"
COMMON="-std=gnu99 -w -fno-strict-aliasing -fwrapv -ftrivial-auto-var-init=zero"
# V: label, flags
run() { label=$1; shift; gcc-12 "$@" $COMMON pose_main.c -lm -o pose_$label 2> pose_$label.cc.err && ./pose_$label 2000 > pose_$label.out; echo "$label exit=$? lines=$(wc -l < pose_$label.out)"; }
run x64_O1_original -m64 -O1
run i686_O1_original -m32 -O1
run i686_O0_original -m32 -O0
run i686_O2_original -m32 -O2
run i686_O1_standard_original -m32 -O1 -fexcess-precision=standard
run i686_O0_standard_original -m32 -O0 -fexcess-precision=standard
run x64_O1_production -m64 -O1 -DPRODUCTION_TEXT $INC
run i686_O1_production -m32 -O1 -DPRODUCTION_TEXT $INC
run i686_O1_standard_production -m32 -O1 -fexcess-precision=standard -DPRODUCTION_TEXT $INC
for f in pose_*.out; do
  n=$(diff pose_x64_O1_original.out $f | grep -c '^<')
  echo "differs from x64_O1_original: $f lines=$n"
done
for f in pose_*.out; do
  a=$(cut -d' ' -f1-4 pose_x64_O1_original.out | diff - $(echo) /dev/null >/dev/null 2>&1; cut -d' ' -f1-4 pose_x64_O1_original.out > /tmp/in_ref; cut -d' ' -f1-4 $f > /tmp/in_f; diff /tmp/in_ref /tmp/in_f | grep -c '^<')
  cut -d' ' -f5-13 pose_x64_O1_original.out > /tmp/out_ref; cut -d' ' -f5-13 $f > /tmp/out_f
  b=$(diff /tmp/out_ref /tmp/out_f | grep -c '^<')
  echo "words vs x64_O1_original: $f input_lines_differing=$a output_lines_differing=$b"
done
