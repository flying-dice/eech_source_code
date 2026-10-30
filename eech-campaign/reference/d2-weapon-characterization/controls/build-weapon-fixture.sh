#!/bin/sh
# usage: build-weapon-fixture.sh <volume> <outdir under eech-campaign/target/weapon-fixture>  (WSL physical path of the repository below)
set -u
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code/eech-campaign" || exit 97
if docker volume ls --format '{{.Name}}' | grep -Fx "$1" >/dev/null; then echo "Refusing to reuse existing volume $1" >&2; exit 2; fi
EECH_WIN_TARGET=$1 sh tools/build-windows-weapon-fixture.sh "$2"
echo "BUILD_EXIT=$?"
