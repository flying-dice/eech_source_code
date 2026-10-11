#!/bin/bash
# Installs what eech-core-ts's verify gates need in Claude Code cloud sessions:
# Lua 5.1, a 32-bit capable C compiler (c-reference harness) and npm packages.
# Mirrors .github/workflows/eech-core-ts.yml.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

SUDO=""
if [ "$(id -u)" -ne 0 ]; then SUDO="sudo"; fi

if ! command -v lua5.1 >/dev/null 2>&1 || ! dpkg -s gcc-multilib >/dev/null 2>&1; then
  $SUDO apt-get update -qq
  DEBIAN_FRONTEND=noninteractive $SUDO apt-get install -y -qq --no-install-recommends lua5.1 gcc gcc-multilib
fi

cd "$CLAUDE_PROJECT_DIR/eech-core-ts"
# npm ci keeps package-lock.json byte-identical (npm install reformats it).
# Skip it when node_modules already matches the lockfile from a cached container.
stamp=node_modules/.lock-sha
want=$(sha256sum package-lock.json | cut -d' ' -f1)
if [ ! -f "$stamp" ] || [ "$(cat "$stamp")" != "$want" ]; then
  npm ci --no-audit --no-fund
  echo "$want" > "$stamp"
fi
