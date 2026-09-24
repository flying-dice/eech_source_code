#!/bin/sh
# The regression test from a Windows (Git Bash) or any Docker host: builds the
# toolchain image and the engine in Docker, assembles the retail roots once
# (kept in the eech-regress volume), and runs tools/regress.sh.
#
# Usage: tools/regress-docker.sh <cvh install> <avh install> [--update] [--exact] [--rebuild-roots]
#   <cvh install>  Comanche vs Hokum, e.g. "D:/Program Files (x86)/GOG Galaxy/Games/Comanche vs Hokum"
#   <avh install>  Apache vs Havoc, e.g. "D:/SteamLibrary/steamapps/common/Enemy Engaged Apache vs Havoc"
set -eu
cvh=$1 avh=$2
shift 2
opts="" rebuild=0
for a in "$@"; do
	case $a in
		--rebuild-roots) rebuild=1 ;;
		*) opts="$opts $a" ;;
	esac
done
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")/.." && pwd)
repo=$(cd "$here/.." && pwd)
# Docker on Windows wants E:/..., not /e/...
case $repo in /[a-zA-Z]/*) repo=$(echo "$repo" | sed -E 's|^/([a-zA-Z])/|\1:/|') ;; esac

docker build -q -t eech-build - < "$here/tools/Dockerfile" > /dev/null
run () {
	docker run --rm -v "$repo:/src" -v "$cvh:/cvh:ro" -v "$avh:/avh:ro" \
		-v eech-regress:/work -v eech-regress-target:/target -v eech-cargo:/usr/local/cargo/registry \
		-e CARGO_TARGET_DIR=/target -e EECH_WORLD=/target/release/eech-world -e REGRESS_OUT=/work/out \
		-w /src/eech-campaign eech-build sh -c "$1"
}
echo "building ..."
# a failed build must stop the test, not leave it running an older binary
run 'cargo build --release -p eech-dc -p eech-world > /target/build.log 2>&1 || { grep -E "error" /target/build.log | tail -20; exit 1; }; tail -1 /target/build.log'
run "if [ $rebuild -eq 1 ] || [ ! -d /work/georgia ]; then rm -rf /work/georgia && sh tools/retail-map3-installs.sh /cvh /avh /work/georgia; fi
	if [ $rebuild -eq 1 ] || [ ! -d /work/lebanon ]; then rm -rf /work/lebanon && sh tools/retail-cvh.sh /cvh map5 /work/lebanon; fi"
run "sh tools/regress.sh /work/georgia /work/lebanon $opts; s=\$?; mkdir -p /src/eech-campaign/target/regress && cp /work/out/*.json /work/out/*.log /src/eech-campaign/target/regress/ 2>/dev/null || true; exit \$s"
