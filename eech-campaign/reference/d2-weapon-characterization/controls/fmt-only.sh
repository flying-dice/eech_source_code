#!/bin/sh
# usage (inside the EECH-D2-Build distro): fmt-only.sh <logfile> [fix]   -- cargo fmt --all --check (or --all with "fix") in a disposable container
log=$1; mode=${2:-check}
cd "/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code" || exit 2
docker run --rm -v "$PWD:/src" -e MODE="$mode" -w /src/eech-campaign eech-build sh -c '
rustup component add rustfmt >/tmp/rustup.out 2>&1; echo RUSTUP_ADD_EXIT=$?
if [ "$MODE" = fix ]; then cargo fmt --all; echo FMT_FIX_EXIT=$?; else cargo fmt --all --check >/tmp/fmt.out 2>&1; echo FMT_EXIT=$?; head -60 /tmp/fmt.out; fi
' > "$log" 2>&1
echo DOCKER_EXIT=$? >> "$log"
