#!/bin/sh
# Assembles an installation root for a native Comanche vs Hokum campaign
# (map4 Taiwan, map5 Lebanon, map6 Yemen) from a retail install, which is not
# in the repository. These maps ship complete in the install (terrain, roads,
# population with producers), unlike map1-3 (see retail-map3.sh):
#
#   <install>/cohokum/3ddata/          3dobjs.*, 3dobjdb.bin, textures.*, ...
#   <install>/common/maps/<map>/       camp01, route, terrain, graphics, ...
#
# plus the repository's own data (setup/common/data, setup/cohokum tables and
# its 3ddata/objects).
#
# The retail campaign scripts (*.SCR) end the campaign as a FAIL after 30
# minutes (a TIME_DURATION trigger); the installed copies drop that trigger,
# and the originals are kept as *.retail.
#
# Usage: tools/retail-cvh.sh <install> <map> <root>      e.g. <map> = map5
#   then: eech-world lua/campaign.lua root=<root> scenario=lebanon_retail hours=12 acmi=out.acmi
set -eu
install=$1 mapdir=$2 root=$3
repo=$(cd "$(dirname "$0")/../.." && pwd)
mkdir -p "$root/cohokum" "$root/common/data" "$root/common/maps"
cp -r "$install/cohokum/3ddata" "$root/cohokum/"
cp -r "$repo/setup/cohokum/3ddata/objects" "$root/cohokum/3ddata/"
for t in GWUT1162.CSV EXPLOS.CSV METASMOK.CSV SMOKES.CSV; do cp "$repo/setup/cohokum/$t" "$root/cohokum/"; done
cp -r "$repo/setup/common/data/." "$root/common/data/"
cp -r "$install/common/maps/$mapdir" "$root/common/maps/"
for scr in "$root/common/maps/$mapdir"/camp01/*.[Ss][Cc][Rr]; do
	[ -f "$scr" ] || continue
	cp "$scr" "$scr.retail"
	python3 - "$scr" <<'PY'
import sys
p = sys.argv[1]
s = open(p, 'rb').read().decode('latin1')
if ':CREATE_EVENT time_duration_event' in s:
    a = s.index(':CREATE_EVENT time_duration_event')
    b = s.index(':EVENT time_duration_event') + len(':EVENT time_duration_event')
    s = s[:a] + '// EECH headless: the retail 30-minute END_CAMPAIGN FAIL trigger is removed (.retail has it)\r\n' + s[b:]
    open(p, 'wb').write(s.encode('latin1'))
PY
done
echo "retail $mapdir installation at $root"
