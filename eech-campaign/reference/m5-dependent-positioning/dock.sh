#!/bin/bash
# usage: dock.sh <logname> '<cmd run in /src/eech-campaign>'
# Isolated target volume eech-dep-pos-target; full output and the true exit are kept in logs/<logname>.log.
repo=$(cd /e/eech_source_code/.claude/worktrees/m5-dependent-positioning && pwd -W)
log=/e/eech-runs/m5-d1/dependent-positioning-20261023/logs/$1.log
MSYS_NO_PATHCONV=1 docker run --rm -v "$repo:/src" -v eech-cargo:/usr/local/cargo/registry -v eech-dep-pos-target:/target -e CARGO_TARGET_DIR=/target -w /src/eech-campaign eech-build bash -c "set -o pipefail; $2; rc=\$?; echo EXIT=\$rc; exit \$rc" > "$log" 2>&1
rc=$?
echo "docker exit=$rc" >> "$log"
echo "log: $log (exit=$rc)"; if [ $rc -ne 0 ]; then tail -60 "$log"; else tail -5 "$log"; fi
exit $rc
