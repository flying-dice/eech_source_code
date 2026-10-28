#!/bin/sh
# syntax-only compile of the touched weapon sources and the fixture, feature ON and OFF, against the staged headers of an earlier build, with the working tree's changed
# headers laid over them (an overlay, so the staged copies cannot shadow the new protocol)
set -u
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code' || exit 97
vol=${1:-eech-win-target-d2w-s1-abc6d35bd}
docker run --rm -v "$PWD:/src:ro" -v "$vol:/target:ro" eech-build sh -eu -c '
tree=$(find /target/x86_64-pc-windows-gnu/release/build -path "*/eech-engine-sys-*/out/tree" -type d | head -1)
test -n "$tree"
flags="-fsyntax-only -std=gnu99 -fcommon -fwrapv -DWIN32 -DEECH_CAMPAIGN_HOST_IDENTITIES -DEECH_CAMPAIGN_HOST_ARRIVALS -DEECH_CAMPAIGN_HOST_MOVEMENT -DEECH_CAMPAIGN_HOST_LOS -Werror=implicit-function-declaration -Werror=implicit-int -Werror=int-conversion -include /src/eech-campaign/crates/eech-engine-sys/csrc/eech_win32_names.h -I /src/eech-campaign/crates/eech-engine-sys/compat -I /src/eech-campaign/crates/eech-engine-sys/csrc -I /tmp/ov -I $tree/modules -I $tree/aphavoc/source"
fail=0
mkdir -p /tmp/ov/entity/mobile/weapon
cp /src/aphavoc/source/entity/mobile/mb_wpnfx.h /src/aphavoc/source/entity/mobile/mb_hostaim.h /tmp/ov/entity/mobile/
cp /src/aphavoc/source/entity/mobile/weapon/wn_move.h /src/aphavoc/source/entity/mobile/weapon/wn_cllsn.h /tmp/ov/entity/mobile/weapon/
for def in "" "-DEECH_WEAPON_FIXTURE"; do
  for f in aphavoc/source/entity/mobile/mb_hostaim.c aphavoc/source/entity/mobile/weapon/wn_move.c aphavoc/source/entity/mobile/weapon/wn_creat.c aphavoc/source/entity/mobile/weapon/wn_updt.c aphavoc/source/entity/mobile/weapon/wn_cllsn.c eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c; do
    if x86_64-w64-mingw32-gcc $flags $def -Wall -Wno-unused-function -Wno-unused-variable -Wno-parentheses -Wno-unused-but-set-variable -iquote "/src/$(dirname "$f")" "/src/$f" > /tmp/out.txt 2>&1; then echo "OK   [$def] $f"; grep -c "warning:" /tmp/out.txt | sed "s/^/     warnings: /"; else echo "FAIL [$def] $f"; grep -B2 -A6 "error" /tmp/out.txt | head -80; fail=1; fi
  done
done
exit $fail
'
echo "SYNTAX_EXIT=$?"
