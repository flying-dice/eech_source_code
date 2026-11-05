#!/bin/sh
# Assembles the retail Georgia campaign (map3) root from two installs, as
# they ship today:
#
#   <cvh>  Comanche vs Hokum (GOG): cohokum/3ddata, common/maps/map3/camp01 and
#          route/POPNAME.DAT, BRIDGE.POP (no roads or terrain for map3)
#   <avh>  Apache vs Havoc (Steam): common/maps/map3/route (ROADDATA.*) and terrain
#
# into the layout tools/retail-map3.sh takes, then runs it.
#
# Usage: tools/retail-map3-installs.sh <cvh install> <avh install> <root>
set -eu
cvh=$1 avh=$2 root=$3
data=$(mktemp -d)
trap 'rm -rf "$data"' EXIT
mkdir -p "$data/map3/route"
cp -r "$cvh/cohokum/3ddata" "$data/"
cp -r "$cvh/common/maps/map3/camp01" "$data/map3/"
cp -r "$avh/common/maps/map3/route/." "$data/map3/route/"
cp "$cvh/common/maps/map3/route/"* "$data/map3/route/"
cp -r "$avh/common/maps/map3/terrain" "$data/map3/"
sh "$(dirname "$0")/retail-map3.sh" "$data" "$root"
