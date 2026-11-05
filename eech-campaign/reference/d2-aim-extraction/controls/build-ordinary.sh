#!/bin/sh
# usage: build-ordinary.sh <volume> <outdir under eech-campaign/target/>  (WSL path of the repository below). The unchanged tools/build-windows.sh in a fresh volume,
# with git told what the Windows side already has configured (core.autocrlf=input), as build-aim-fixture-lf.sh does.
set -u
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code/eech-campaign" || exit 97
if docker volume ls --format '{{.Name}}' | grep -Fx "$1" >/dev/null; then echo "Refusing to reuse existing volume $1" >&2; exit 2; fi
export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.autocrlf GIT_CONFIG_VALUE_0=input
EECH_WIN_TARGET=$1 sh tools/build-windows.sh "$2"
echo "BUILD_EXIT=$?"
