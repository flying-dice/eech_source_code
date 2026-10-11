#!/bin/bash
# Installs what eech-core-ts's verify gates need in Claude Code cloud sessions:
# Lua 5.1, a 32-bit capable C compiler (c-reference harness) and npm packages,
# the same system packages .github/workflows/eech-core-ts.yml installs. Node
# itself comes from the cloud image (22 at the time of writing, as in CI).
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# SessionStart stdout is added to Claude's context; keep install noise out of it.
exec 1>&2

installed() {
  dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q 'install ok installed'
}

if ! installed lua5.1 || ! installed gcc || ! installed gcc-multilib; then
  if [ "$(id -u)" -eq 0 ]; then
    SUDO=()
  elif command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    SUDO=(sudo -n)
  else
    echo "session-start: need root or passwordless sudo to install lua5.1 gcc gcc-multilib" >&2
    exit 1
  fi
  "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get update -qq
  "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends lua5.1 gcc gcc-multilib
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
