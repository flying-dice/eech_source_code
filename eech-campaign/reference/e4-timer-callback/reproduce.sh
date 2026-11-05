#!/bin/sh
# Reproduce the E4 evidence (README.md): newer-GCC red/green builds, the -O2
# inventory and the x86_64 suite on newer GCC, and the GCC 12 timer.o
# disassembly comparison. Needs git and Docker; any host with sh.
#
#   reproduce.sh [<red revision> [<green revision>]] [-- <out dir>]
#   defaults: red 94432391 (no E4), green 303103d2 (E4), out ./out
#
# The images are pinned by digest (the tags they were pulled as are noted).
set -eu
red=${1:-94432391}
green=${2:-303103d2}
here=$(cd "$(dirname "$0")" && pwd)
out=${OUT:-$here/out}
repo=$(git -C "$here" rev-parse --show-toplevel)

TRIXIE=rust@sha256:e259494b14672950a909b4c54527e19d29cae6ebf980672114c8bb85e5594e4a   # rust:trixie, GCC 14.2.0, Rust 1.99.1
GCC15=gcc@sha256:1fdcd041afad61e21d4ca0f8046e36f8da61baaa815f5ba290e39a15c487926a     # gcc:15, GCC 15.3.0 (+ rustup 1.99.1)
BOOKWORM=rust@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e # rust:bookworm, GCC 12.2.0 (tools/Dockerfile's base)

src=$(mktemp -d)
for r in red green; do
	rev=$(eval echo \$$r)
	mkdir -p "$src/$r"
	# the whole of eech-campaign: the suite reads scenarios/, corpus/, reference/ ...
	git -C "$repo" archive "$rev" eech-campaign aphavoc modules | tar -x -C "$src/$r"
	git -C "$repo" rev-parse "$rev" > "$src/$r/REVISION"
done
mkdir -p "$out"

# Git Bash on Windows: give Docker Windows paths, and stop MSYS rewriting the mounts
host() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
export MSYS_NO_PATHCONV=1

run() {
	image=$1; shift
	docker run --rm -v "$(host "$src/red"):/red:ro" -v "$(host "$src/green"):/green:ro" \
		-v "$(host "$here/inner.sh"):/inner.sh:ro" -v "$(host "$out"):/out" "$@" "$image" sh /inner.sh $INNER
}
INNER="build gcc14" run $TRIXIE
INNER="build gcc15" run $GCC15 -e RUSTUP_INSTALL=1.99.1
INNER="inventory gcc14" run $TRIXIE
INNER="inventory gcc15" run $GCC15 -e RUSTUP_INSTALL=1.99.1
INNER="suite gcc14" run $TRIXIE
INNER="disasm" run $BOOKWORM
rm -rf "$src"
