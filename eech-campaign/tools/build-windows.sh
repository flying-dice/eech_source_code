#!/bin/sh
# Builds the Windows binaries (x86_64-pc-windows-gnu, MinGW-w64) in the
# eech-build Docker image (tools/Dockerfile) and collects them, with the lua.dll
# they load, in target/windows/:
#
#   eech-world.exe   the host: embeds nothing, loads lua.dll and requires eech_dc
#   eech_dc.dll      the whole headless engine as a Lua 5.1 module
#   lua.dll          Lua 5.1.5, built as DCS World names its Lua DLL
#   lua/             campaign.lua, metrics.lua and lifecycle.lua
#   BUILD-INFO.txt   the source commit, the toolchain and the SHA-256 of each file above
#
# Usage (Git Bash or any Docker host): tools/build-windows.sh [<out dir under eech-campaign/, default target/windows>]
#   (running binaries are locked by Windows: build into another directory while campaigns run)
# then, on Windows: target\windows\eech-world.exe target\windows\lua\campaign.lua root=... scenario=... hours=...
set -eu
out=${1:-target/windows}
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")/.." && pwd)
repo=$(cd "$here/.." && pwd)
commit=$(cd "$here" && git rev-parse HEAD)
# uncommitted changes to anything the build reads make the commit an incomplete identity
dirty=$(cd "$here" && git status --porcelain -- . ../aphavoc ../modules | wc -l | tr -d ' ')
case $repo in /[a-zA-Z]/*) repo=$(echo "$repo" | sed -E 's|^/([a-zA-Z])/|\1:/|') ;; esac

docker build -q -t eech-build - < "$here/tools/Dockerfile" > /dev/null
image=$(docker image inspect --format '{{.Id}}' eech-build)
# EECH_WIN_TARGET: the cargo target volume (a fresh name builds from scratch)
docker run --rm -v "$repo:/src" -v "${EECH_WIN_TARGET:-eech-win-target}:/target" -v eech-cargo:/usr/local/cargo/registry \
	-e CARGO_TARGET_DIR=/target -e LUA_LIB=/opt/lua-mingw -e LUA_LIB_NAME=lua \
	-e OUT="/src/eech-campaign/$out" -e COMMIT="$commit" -e DIRTY="$dirty" -e IMAGE="$image" \
	-w /src/eech-campaign eech-build sh -c '
	set -e
	cargo build --release --target x86_64-pc-windows-gnu -p eech-dc -p eech-world > /target/build.log 2>&1 || { grep -E "error" /target/build.log | tail -20; exit 1; }
	tail -1 /target/build.log
	out=$OUT
	mkdir -p $out/lua
	cp /target/x86_64-pc-windows-gnu/release/eech_dc.dll /target/x86_64-pc-windows-gnu/release/eech-world.exe /opt/lua-mingw/lua.dll $out/
	cp crates/eech-world/lua/*.lua $out/lua/
	{
		echo "commit: $COMMIT"
		echo "uncommitted changes in the build inputs: $DIRTY files"
		echo "image: $IMAGE"
		echo "rustc: $(rustc -V)"
		echo "cargo: $(cargo -V)"
		echo "mingw: gcc $(x86_64-w64-mingw32-gcc -dumpfullversion), gcc-mingw-w64-x86-64-posix $(dpkg-query -W -f "\${Version}" gcc-mingw-w64-x86-64-posix)"
		echo "lua.dll: Lua 5.1.5 (lua-5.1.5.tar.gz, SHA-256 checked in tools/Dockerfile)"
		echo "target: x86_64-pc-windows-gnu, release"
		echo "sha256:"
		cd $out && sha256sum eech_dc.dll eech-world.exe lua.dll lua/*.lua | sed "s/^/  /"
	} > $out/BUILD-INFO.txt
	cat $out/BUILD-INFO.txt'
