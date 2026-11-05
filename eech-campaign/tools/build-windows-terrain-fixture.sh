#!/bin/sh
# TEST-ONLY (EEDCS-47): builds the Windows binaries with the terrain surface characterisation fixture
# (`--features eech-dc/terrain-fixture`), through the unchanged tools/build-windows.sh and its Dockerfile image, in a cargo
# target volume of its own. The shipping build, its flags and its volume are untouched; the fixture DLL is never a
# shipping artifact. The only difference from build-windows.sh is the one cargo argument added to its build line, which this
# wrapper checks is the single change before it runs a temporary copy kept in the ignored target/ directory (one level below eech-campaign, so its paths resolve, and the build's uncommitted-files count stays honest).
#
# Usage (Git Bash, Docker with Linux containers): tools/build-windows-terrain-fixture.sh target/terrain-fixture/<dir>
#   The output must be under target/terrain-fixture/. BUILD-INFO.txt in it names the commit, the image and the SHA-256 of each file
#   and says FIXTURE BUILD. Run from a clean checkout of the commit to be characterised. EECH_WIN_TARGET names the cargo target
#   volume (default eech-win-target-terrain-fixture); use a fresh name for a clean build.
set -eu
out=${1:?output directory under target/terrain-fixture/}
case $out in target/terrain-fixture/*) ;; *) echo "the output must be under target/terrain-fixture/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
src=$here/tools/build-windows.sh
mkdir -p "$here/target"
tmp=$here/target/.build-windows-terrain-fixture.tmp.sh
old='-p eech-dc -p eech-world > /target/build.log'
new='-p eech-dc -p eech-world --features eech-dc/terrain-fixture > /target/build.log'
n=$(grep -cF -e "$old" "$src" || true)
if [ "$n" != 1 ]; then echo "tools/build-windows.sh: expected exactly one cargo build line to adapt, found $n" >&2; exit 1; fi
trap 'rm -f "$tmp"' EXIT
sed "s|$old|$new|; s|echo \"target: x86_64-pc-windows-gnu, release\"|echo \"target: x86_64-pc-windows-gnu, release, FIXTURE BUILD (eech-dc/terrain-fixture): test-only, not a shipping artifact\"|" "$src" > "$tmp"
if [ "$(diff "$src" "$tmp" | grep -c '^>')" != 2 ]; then echo "the adapted copy differs in more than the two intended lines" >&2; exit 1; fi
EECH_WIN_TARGET=${EECH_WIN_TARGET:-eech-win-target-terrain-fixture} sh "$tmp" "$out"
