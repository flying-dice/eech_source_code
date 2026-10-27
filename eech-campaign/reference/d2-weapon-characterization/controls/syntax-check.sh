#!/bin/sh
# syntax-only compile of the touched weapon sources and the fixture, feature ON and OFF, against the staged tree of an earlier build (headers only)
set -u
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code' || exit 97
vol=${1:-eech-win-target-d2x-fix-c21852b09}
docker run --rm -v "$PWD:/src:ro" -v "$vol:/target:ro" eech-build sh -eu -c '
tree=$(find /target/x86_64-pc-windows-gnu/release/build -path "*/eech-engine-sys-*/out/tree" -type d | head -1)
test -n "$tree"
flags="-fsyntax-only -std=gnu99 -fcommon -fwrapv -DWIN32 -DEECH_CAMPAIGN_HOST_IDENTITIES -DEECH_CAMPAIGN_HOST_ARRIVALS -DEECH_CAMPAIGN_HOST_MOVEMENT -DEECH_CAMPAIGN_HOST_LOS -Werror=implicit-function-declaration -Werror=implicit-int -Werror=int-conversion -include /src/eech-campaign/crates/eech-engine-sys/csrc/eech_win32_names.h -I /src/eech-campaign/crates/eech-engine-sys/compat -I /src/eech-campaign/crates/eech-engine-sys/csrc -I $tree/modules -I $tree/aphavoc/source -I /tmp/ov"
fail=0
mkdir -p /tmp/ov/entity/mobile
cp /src/aphavoc/source/entity/mobile/mb_wpnfx.h /tmp/ov/entity/mobile/
for def in "" "-DEECH_WEAPON_FIXTURE"; do
  for f in aphavoc/source/entity/mobile/weapon/wn_move.c aphavoc/source/entity/mobile/weapon/wn_creat.c aphavoc/source/entity/mobile/weapon/wn_updt.c aphavoc/source/entity/mobile/weapon/wn_cllsn.c eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c; do
    if x86_64-w64-mingw32-gcc $flags $def -w -iquote "/src/$(dirname "$f")" "/src/$f" > /tmp/out.txt 2>&1; then echo "OK   [$def] $f"; else echo "FAIL [$def] $f"; head -40 /tmp/out.txt; fail=1; fi
  done
done
exit $fail
'
echo "SYNTAX_EXIT=$?"
