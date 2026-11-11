#!/bin/sh
# TEST-ONLY (EEDCS-49, D3): builds the Windows binaries with the READ-ONLY sector tallest-structure observer
# (`--features eech-dc/sector-height-observer`; docs/d3-sector-height-public-baseline-plan.md), through the unchanged tools/build-windows.sh and its Dockerfile image, in a cargo
# target volume of its own. The shipping build, its flags and its volume are untouched; the observer DLL is never a shipping artifact. The only difference from build-windows.sh is the
# one cargo argument added to its build line, which this wrapper checks is the single change before it runs a temporary copy kept in the ignored target/ directory (one level below
# eech-campaign, so its paths resolve, and the build's uncommitted-files count stays honest).
#
# Usage (Git Bash, Docker Desktop with Linux containers): tools/build-windows-sector-height-observer.sh target/sector-height-observer/<dir>
#   The output must be under target/sector-height-observer/. BUILD-INFO.txt in it names the commit, the image and the SHA-256 of each file and says FIXTURE BUILD. Run from a clean
#   checkout of the commit to be observed. The observer changes no campaign behaviour; the baseline run is lua/sector-height-baseline.lua on this DLL.
set -eu
out=${1:?output directory under target/sector-height-observer/}
case $out in target/sector-height-observer/*) ;; *) echo "the output must be under target/sector-height-observer/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
src=$here/tools/build-windows.sh
mkdir -p "$here/target"
tmp=$here/target/.build-windows-sector-height-observer.tmp.sh
old='-p eech-dc -p eech-world > /target/build.log'
new='-p eech-dc -p eech-world --features eech-dc/sector-height-observer > /target/build.log'
n=$(grep -cF -e "$old" "$src" || true)
if [ "$n" != 1 ]; then echo "tools/build-windows.sh: expected exactly one cargo build line to adapt, found $n" >&2; exit 1; fi
trap 'rm -f "$tmp"' EXIT
sed "s|$old|$new|; s|echo \"target: x86_64-pc-windows-gnu, release\"|echo \"target: x86_64-pc-windows-gnu, release, FIXTURE BUILD (eech-dc/sector-height-observer): test-only, not a shipping artifact\"|" "$src" > "$tmp"
if [ "$(diff "$src" "$tmp" | grep -c '^>')" != 2 ]; then echo "the adapted copy differs in more than the two intended lines" >&2; exit 1; fi
EECH_WIN_TARGET=${EECH_WIN_TARGET:-eech-win-target-sector-height-observer} sh "$tmp" "$out"
