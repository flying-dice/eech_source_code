#!/bin/sh
# compile-check (syntax only) of the historical harness's extracted unit with the harness's own flags, for the extractor as of 90b (red) and as fixed (green).
# Read-only mounts, no cache volume. -m32 is the harness's platform; the image may not carry a 32-bit libc, which the run reports.
repo="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/collision-characterization-20261029"
gen="/mnt/c/Users/starscream/AppData/Local/Temp/cref"
image=sha256:4a5618cbc1e13e96caa4cb1ff0076c4c24dc883876d0bdec1398024f3a2d6c51
test -z "$(docker ps -q)"
for arch in -m32 -m64; do
  for v in red green; do
    docker run --rm -v "$repo:/src:ro" -v "$gen:/gen:ro" "$image" sh -c "cc -std=gnu99 $arch -msse2 -mfpmath=sse -O0 -ffp-contract=off -fno-fast-math -I /gen/$v -I /src/eech-core-ts/c-reference -I /src/aphavoc/source -I /src/modules -Werror=implicit-function-declaration -Werror=incompatible-pointer-types -Werror=int-conversion -Werror=return-type -fsyntax-only /gen/$v/eech_extracted.c" > "$gen/check$arch-$v.out" 2>&1
    echo "$arch $v exit=$?" >> "$gen/check-exits.txt"
  done
done
