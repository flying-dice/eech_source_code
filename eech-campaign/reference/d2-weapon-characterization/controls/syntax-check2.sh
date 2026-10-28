#!/bin/sh
# syntax-only compile of the touched weapon sources and the fixture, feature ON and OFF, against a writable copy of the staged tree of an earlier build with the working tree's
# headers of entity/mobile (and its weapon directory) copied over their staged counterparts, so that the staged copies cannot shadow the new protocol.  The files compiled are
# the weapon-execution sources and the two engine files that include them; init.c and the other callers are NOT in this set (a subset check, not a whole-engine compile).
set -u
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code' || exit 97
vol=${1:-eech-win-target-d2w-s1-abc6d35bd}
docker run --rm -v "$PWD:/src:ro" -v "$vol:/target:ro" eech-build sh -eu -c '
tree=$(find /target/x86_64-pc-windows-gnu/release/build -path "*/eech-engine-sys-*/out/tree" -type d | head -1)
test -n "$tree"
cp -a "$tree" /tmp/tree
cp /src/aphavoc/source/entity/mobile/mb_*.h /tmp/tree/aphavoc/source/entity/mobile/
cp /src/aphavoc/source/entity/mobile/weapon/*.h /tmp/tree/aphavoc/source/entity/mobile/weapon/
flags="-fsyntax-only -std=gnu99 -fcommon -fwrapv -DWIN32 -DEECH_CAMPAIGN_HOST_IDENTITIES -DEECH_CAMPAIGN_HOST_ARRIVALS -DEECH_CAMPAIGN_HOST_MOVEMENT -DEECH_CAMPAIGN_HOST_LOS -Werror=implicit-function-declaration -Werror=implicit-int -Werror=int-conversion -include /src/eech-campaign/crates/eech-engine-sys/csrc/eech_win32_names.h -I /src/eech-campaign/crates/eech-engine-sys/compat -I /src/eech-campaign/crates/eech-engine-sys/csrc -I /tmp/tree/modules -I /tmp/tree/aphavoc/source"
fail=0
for def in "" "-DEECH_WEAPON_FIXTURE"; do
  for f in aphavoc/source/entity/mobile/mb_hostaim.c aphavoc/source/entity/mobile/weapon/wn_move.c aphavoc/source/entity/mobile/weapon/wn_creat.c aphavoc/source/entity/mobile/weapon/wn_updt.c aphavoc/source/entity/mobile/weapon/wn_cllsn.c aphavoc/source/entity/mobile/weapon/wn_anim.c aphavoc/source/entity/mobile/weapon/wn_decoy.c aphavoc/source/entity/mobile/weapon/wn_msgs.c aphavoc/source/entity/mobile/weapon/wn_draw.c eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c; do
    if x86_64-w64-mingw32-gcc $flags $def -Wall -Wno-unused-function -Wno-unused-variable -Wno-parentheses -Wno-unused-but-set-variable -iquote "/src/$(dirname "$f")" "/src/$f" > /tmp/out.txt 2>&1; then echo "OK   [$def] $f"; grep -c "warning:" /tmp/out.txt | sed "s/^/     warnings: /"; else echo "FAIL [$def] $f"; grep -B2 -A6 "error" /tmp/out.txt | head -80; fail=1; fi
  done
done
exit $fail
'
echo "SYNTAX_EXIT=$?"
