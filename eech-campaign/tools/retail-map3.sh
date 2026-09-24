#!/bin/sh
# Assembles an installation root for the retail Georgia campaign (EECH map3,
# "Caspian Black Gold") from retail data, which is not in the repository:
#
#   <data>/3ddata/                     cohokum/3ddata of a retail install (3dobjs.*, 3dobjdb.bin, textures.*, ...)
#   <data>/map3/camp01/                GEORGIA.CHC, GEORGIA.SCR, GEORGIA.SID, GEORGIA.BIN
#   <data>/map3/route/                 POPNAME.DAT, BRIDGE.POP, ROADDATA.*, rivdata.*, ...
#   <data>/map3/terrain/               terrain.ffp, default.sec, default.rgb
#   <data>/map3/campaign/, graphics/   optional, copied as they are
#
# plus the repository's own data (setup/common/data, setup/cohokum tables).
#
# The retail GEORGIA.SCR ends the campaign as a FAIL after 30 minutes
# (a TIME_DURATION trigger); the installed copy drops that trigger, and the
# original is kept as GEORGIA.SCR.retail.
#
# Usage: tools/retail-map3.sh <data> <root>
#   then: eech-world lua/campaign.lua root=<root> scenario=georgia_retail hours=12 acmi=out.acmi
set -eu
data=$1 root=$2
repo=$(cd "$(dirname "$0")/../.." && pwd)
map=$root/common/maps/map3
mkdir -p "$root/cohokum" "$root/common/data" "$map"
cp -r "$data/3ddata" "$root/cohokum/"
for t in GWUT1162.CSV EXPLOS.CSV METASMOK.CSV SMOKES.CSV; do cp "$repo/setup/cohokum/$t" "$root/cohokum/"; done
cp -r "$repo/setup/common/data/." "$root/common/data/"
for d in camp01 route terrain campaign graphics; do
	[ -d "$data/map3/$d" ] && cp -r "$data/map3/$d" "$map/"
done
scr=$(ls "$map/camp01/" | grep -i '^georgia\.scr$' | head -1)
cp "$map/camp01/$scr" "$map/camp01/$scr.retail"
python3 - "$map/camp01/$scr" <<'PY'
import sys
p = sys.argv[1]
s = open(p, 'rb').read().decode('latin1')
if ':CREATE_EVENT time_duration_event' in s:
    a = s.index(':CREATE_EVENT time_duration_event')
    b = s.index(':EVENT time_duration_event') + len(':EVENT time_duration_event')
    s = s[:a] + '// EECH headless: the retail 30-minute END_CAMPAIGN FAIL trigger is removed (.retail has it)\r\n' + s[b:]
    open(p, 'wb').write(s.encode('latin1'))
PY
echo "retail map3 installation at $root"
