#!/bin/sh
# TEST-ONLY (EEDCS-50, D3): builds the Windows binaries with the original airfield route-parser observer
# (`--features eech-dc/route-fixture`, plus `eech-dc/route-original` when EECH_ROUTE_PROFILE=original keeps the unmoved parser), through the unchanged tools/build-windows.sh and its Dockerfile image, in a cargo
# target volume of its own. The shipping build, its flags and its volume are untouched; the fixture DLL is never a
# shipping artifact. The only difference from build-windows.sh is the one cargo argument added to its build line, which this
# wrapper checks is the single change before it runs a temporary copy kept in the ignored target/ directory (one level below eech-campaign, so its paths resolve, and the build's uncommitted-files count stays honest).
#
# Usage (Git Bash, Docker Desktop with Linux containers): tools/build-windows-route-fixture.sh target/route-fixture/<dir>
#   The output must be under target/route-fixture/. BUILD-INFO.txt in it names the commit, the image and the SHA-256 of each file
#   and says FIXTURE BUILD. Run from a clean checkout of the commit to be characterised.
set -eu
out=${1:?output directory under target/route-fixture/}
case $out in target/route-fixture/*) ;; *) echo "the output must be under target/route-fixture/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
src=$here/tools/build-windows.sh
mkdir -p "$here/target"
tmp=$here/target/.build-windows-route-fixture.tmp.sh
old='-p eech-dc -p eech-world > /target/build.log'
features='eech-dc/route-fixture'
case ${EECH_ROUTE_PROFILE:-host} in
	host) ;;
	original) features='eech-dc/route-fixture,eech-dc/route-original' ;;
	*) echo "EECH_ROUTE_PROFILE must be host (the default: the parser takes its route from the host) or original (the unmoved parser, the original characterisation)" >&2; exit 2 ;;
esac
new="-p eech-dc -p eech-world --features $features > /target/build.log"
n=$(grep -cF -e "$old" "$src" || true)
if [ "$n" != 1 ]; then echo "tools/build-windows.sh: expected exactly one cargo build line to adapt, found $n" >&2; exit 1; fi
trap 'rm -f "$tmp"' EXIT
sed "s|$old|$new|; s|echo \"target: x86_64-pc-windows-gnu, release\"|echo \"target: x86_64-pc-windows-gnu, release, FIXTURE BUILD (eech-dc/route-fixture): test-only, not a shipping artifact\"|" "$src" > "$tmp"
if [ "$(diff "$src" "$tmp" | grep -c '^>')" != 2 ]; then echo "the adapted copy differs in more than the two intended lines" >&2; exit 1; fi
EECH_WIN_TARGET=${EECH_WIN_TARGET:-eech-win-target-route-fixture} sh "$tmp" "$out"
