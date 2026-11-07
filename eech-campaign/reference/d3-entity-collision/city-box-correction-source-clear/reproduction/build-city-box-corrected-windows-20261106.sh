#!/bin/sh
set -eu
repo=${1:?exact frozen snapshot WSL path}
source=${2:?exact source commit}
volume=${3:?fresh dedicated cache}
profile=${4:?ordinary, hosted or original}
case "$repo" in
 /mnt/d/EECH-Migration-Evidence/city-box-corrected-*-20261106-source) ;;
 *) echo 'unexpected frozen source directory' >&2; exit 91 ;;
esac
test "$(git -C "$repo" rev-parse HEAD)" = "$source"
test -z "$(git -C "$repo" status --porcelain --untracked-files=all)"
test -z "$(docker ps -q --filter volume="$volume")"
short=$(printf '%.9s' "$source")
cd "$repo/eech-campaign"
case "$profile" in
 ordinary)
  if docker volume inspect "$volume" >/dev/null 2>&1; then echo 'fresh cache exists: inspect actual owners before any retry' >&2; exit 90; fi
  EECH_WIN_TARGET="$volume" sh tools/build-windows.sh "target/windows-city-box-$short"
  ;;
 hosted)
  docker volume inspect "$volume" >/dev/null
  EECH_WIN_TARGET="$volume" sh tools/build-windows-weapon-fixture.sh "target/weapon-fixture/city-box-$short"
  ;;
 original)
  docker volume inspect "$volume" >/dev/null
  EECH_WIN_TARGET="$volume" sh "target/.build-city-box-original-$short.sh" "target/weapon-fixture/city-box-original-$short"
  ;;
 *) exit 2 ;;
esac
