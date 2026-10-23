#!/bin/sh
# Out-of-repo adaptation of eech-campaign/tools/build-windows.sh for EEDCS-39:
# uses the pinned image by digest (no docker build / tag), logs to the evidence dir.
set -eu
export MSYS_NO_PATHCONV=1
W=E:/eech_source_code/.claude/worktrees/m5-aircraft-control
WS=E:/eech_source_code/.claude/worktrees/m5-aircraft-control
N=E:/eech-runs/m5-d1/aircraft-control-20261023/native
NS=E:/eech-runs/m5-d1/aircraft-control-20261023/native
IMAGE=sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be
commit=$(git -C $WS rev-parse HEAD)
tree=$(git -C $WS rev-parse HEAD^{tree})
dirty=$(cd $WS/eech-campaign && git status --porcelain -- . ../aphavoc ../modules | wc -l | tr -d ' ')
{ echo "commit: $commit"; echo "tree: $tree"; echo "dirty build inputs: $dirty"; echo "image: $IMAGE"; date -u +"started: %Y-%m-%dT%H:%M:%SZ"; } > $NS/build.head
set +e
docker run --rm -v "$W:/src" -v eech-force-creation-target:/target -v eech-cargo:/usr/local/cargo/registry -v "$N:/evidence" \
	-e CARGO_TARGET_DIR=/target -e LUA_LIB=/opt/lua-mingw -e LUA_LIB_NAME=lua \
	-e OUT=/src/eech-campaign/target/aircraft-control -e COMMIT="$commit" -e DIRTY="$dirty" -e IMAGE="$IMAGE" \
	-w /src/eech-campaign "$IMAGE" sh -c '
	set -e
	rc=0
	cargo build --release --target x86_64-pc-windows-gnu -p eech-dc -p eech-world > /evidence/build.log 2>&1 || rc=$?
	echo "$rc" > /evidence/build.exit
	[ "$rc" = 0 ] || exit $rc
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
	cp $out/BUILD-INFO.txt /evidence/BUILD-INFO.txt
	cat $out/BUILD-INFO.txt'
echo "docker exit=$?" > $NS/docker.exit
date -u +"finished: %Y-%m-%dT%H:%M:%SZ" >> $NS/build.head
