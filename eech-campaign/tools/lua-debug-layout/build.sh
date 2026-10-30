#!/bin/sh
# Builds lua-debug-layout.exe in the eech-build image (tools/Dockerfile), against the image's
# Lua 5.1.5 headers and lua.dll import library. The exe imports lua.dll by name, so it measures
# whichever lua.dll sits beside it.
# Usage (Git Bash or any Docker host): tools/lua-debug-layout/build.sh [<out dir under eech-campaign/, default target/lua-debug-layout>]
set -eu
out=${1:-target/lua-debug-layout}
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")/../.." && pwd)
case $here in /[a-zA-Z]/*) here=$(echo "$here" | sed -E 's|^/([a-zA-Z])/|\1:/|') ;; esac
docker build -q -t eech-build - < "$here/tools/Dockerfile" > /dev/null
docker run --rm --network none -v "$here:/src" -w /src eech-build sh -c "
	set -e
	mkdir -p \"$out\"
	x86_64-w64-mingw32-gcc -O2 -Wall -Wextra -I/opt/lua-mingw/include tools/lua-debug-layout/lua-debug-layout.c \
		-L/opt/lua-mingw -llua -o \"$out/lua-debug-layout.exe\"
	sha256sum \"$out/lua-debug-layout.exe\""
