#!/bin/sh
repo="/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/collision-characterization-20261029"
gen="/mnt/c/Users/starscream/AppData/Local/Temp/cref"
image=sha256:4a5618cbc1e13e96caa4cb1ff0076c4c24dc883876d0bdec1398024f3a2d6c51
test -z "$(docker ps -q)"
docker run --rm -v "$repo:/src:ro" -v "$gen:/gen:ro" "$image" sh -c '
bad=0
while read u; do
  cc -std=gnu99 -m64 -msse2 -mfpmath=sse -O0 -ffp-contract=off -fno-fast-math -I /gen/green -I /src/eech-core-ts/c-reference -I /src/aphavoc/source -I /src/modules -Werror=implicit-function-declaration -Werror=incompatible-pointer-types -Werror=int-conversion -Werror=return-type -fsyntax-only "$u" > /tmp/o.txt 2>&1
  code=$?
  echo "$code $u"
  if [ $code -ne 0 ]; then bad=$((bad+1)); grep -E "error" /tmp/o.txt | head -3; fi
done < /gen/units.txt
echo "units-failed=$bad"
' > "$gen/cref-all.out" 2>&1
echo "docker-exit=$?" >> "$gen/cref-all.out"
