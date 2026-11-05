#!/bin/sh
set -eu
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/shipping-source-d2-sensing-a2a041fec/eech-campaign"
export EECH_WIN_TARGET=eech-win-target-d2-sensing-final-a2a041fec
if docker volume ls --format '{{.Name}}' | grep -Fx "$EECH_WIN_TARGET" >/dev/null; then
  echo 'Refusing to reuse an existing final-build target volume' >&2
  exit 2
fi
sh tools/build-windows.sh target/windows-d2-sensing-final
