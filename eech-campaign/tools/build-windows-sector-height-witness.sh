#!/bin/sh
# TEST-ONLY (EEDCS-49, D3): builds the Windows binaries with the session-save FIXTURE METHOD of the native save/restore witness (`--features eech-dc/sector-height-witness`;
# docs/d3-sector-height-production-contract.md section 12, ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1), through the unchanged tools/build-windows.sh and its Dockerfile image, in a cargo target volume of its own.
# The shipping build, its flags and its volume are untouched; the witness DLL is never a shipping artifact. The only difference from build-windows.sh is the one cargo argument added to its build line, which this
# wrapper checks is the single change before it runs a temporary copy kept in the ignored target/ directory (one level below eech-campaign, so its paths resolve, and the build's uncommitted-files count stays
# honest). The guarded-clearance witness (M5) needs no such build: it runs on the ordinary hosted profile, and its original-solver characterisation on tools/build-windows-collision-fixture.sh.
#
# Usage (Git Bash, Docker Desktop with Linux containers): tools/build-windows-sector-height-witness.sh target/sector-height-witness/<dir>
#   The output must be under target/sector-height-witness/. BUILD-INFO.txt in it names the commit, the image and the SHA-256 of each file and says FIXTURE BUILD. Run from a clean checkout of the commit to be
#   witnessed. The method changes no campaign behaviour: it makes the campaign's own save run once; the run is lua/sector-height-witness.lua on this DLL.
set -eu
out=${1:?output directory under target/sector-height-witness/}
case $out in target/sector-height-witness/*) ;; *) echo "the output must be under target/sector-height-witness/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
src=$here/tools/build-windows.sh
mkdir -p "$here/target"
tmp=$here/target/.build-windows-sector-height-witness.tmp.sh
old='-p eech-dc -p eech-world > /target/build.log'
new='-p eech-dc -p eech-world --features eech-dc/sector-height-witness > /target/build.log'
n=$(grep -cF -e "$old" "$src" || true)
if [ "$n" != 1 ]; then echo "tools/build-windows.sh: expected exactly one cargo build line to adapt, found $n" >&2; exit 1; fi
trap 'rm -f "$tmp"' EXIT
sed "s|$old|$new|; s|echo \"target: x86_64-pc-windows-gnu, release\"|echo \"target: x86_64-pc-windows-gnu, release, FIXTURE BUILD (eech-dc/sector-height-witness): test-only, not a shipping artifact\"|" "$src" > "$tmp"
if [ "$(diff "$src" "$tmp" | grep -c '^>')" != 2 ]; then echo "the adapted copy differs in more than the two intended lines" >&2; exit 1; fi
EECH_WIN_TARGET=${EECH_WIN_TARGET:-eech-win-target-sector-height-witness} sh "$tmp" "$out"
