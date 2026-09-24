#!/bin/sh
# Builds the Windows binaries (x86_64-pc-windows-gnu, MinGW-w64) in the
# eech-build Docker image (tools/Dockerfile) and collects them, with the lua.dll
# they load, in target/windows/:
#
#   eech-world.exe   the host: embeds nothing, loads lua.dll and requires eech_dc
#   eech_dc.dll      the whole headless engine as a Lua 5.1 module
#   lua.dll          Lua 5.1.5, built as DCS World names its Lua DLL
#   lua/             campaign.lua and metrics.lua
#
# Usage (Git Bash or any Docker host): tools/build-windows.sh [<out dir under eech-campaign/, default target/windows>]
#   (running binaries are locked by Windows: build into another directory while campaigns run)
# then, on Windows: target\windows\eech-world.exe target\windows\lua\campaign.lua root=... scenario=... hours=...
set -eu
out=${1:-target/windows}
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")/.." && pwd)
repo=$(cd "$here/.." && pwd)
case $repo in /[a-zA-Z]/*) repo=$(echo "$repo" | sed -E 's|^/([a-zA-Z])/|\1:/|') ;; esac

docker build -q -t eech-build - < "$here/tools/Dockerfile" > /dev/null
docker run --rm -v "$repo:/src" -v eech-win-target:/target -v eech-cargo:/usr/local/cargo/registry \
	-e CARGO_TARGET_DIR=/target -e LUA_LIB=/opt/lua-mingw -e LUA_LIB_NAME=lua \
	-e OUT="/src/eech-campaign/$out" -w /src/eech-campaign eech-build sh -c '
	set -e
	cargo build --release --target x86_64-pc-windows-gnu -p eech-dc -p eech-world > /target/build.log 2>&1 || { grep -E "error" /target/build.log | tail -20; exit 1; }
	tail -1 /target/build.log
	out=$OUT
	mkdir -p $out/lua
	cp /target/x86_64-pc-windows-gnu/release/eech_dc.dll /target/x86_64-pc-windows-gnu/release/eech-world.exe /opt/lua-mingw/lua.dll $out/
	cp crates/eech-world/lua/*.lua $out/lua/
	ls -la $out'
