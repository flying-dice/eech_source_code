#!/bin/sh
# The FINAL focused checks of the pinned Lua archive downloader, one child at a time, between its PRE and POST input maps (no source write in between). Nothing here builds, runs native code, uses Docker or the
# network (c7 contacts only a closed loopback port). Each child's stdout/stderr/exit is kept in the output directory; the progress file lists the real exits.
PY="C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
REPO=R:/d3-evidence/collision-characterization-20261029
ARCH=R:/d3-evidence/lua-5.1.5-MacPorts-verified-20261107.tar.gz
PIN=2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333
O=R:/d3-evidence/ci3-final-set-20261107
mkdir "$O" || exit 2
P=$O/progress
: >"$P"
run() {
	label=$1
	shift
	"$@" >"$O/$label.stdout" 2>"$O/$label.stderr"
	echo "$label exit=$?" >>"$P"
}
cd "$REPO/eech-campaign" || exit 2
# the pinned archive file is the one Root verified over HTTPS (receipt be4adb3d...); its hash is checked here again, not trusted from the receipt
run c0-archive-identity sh -c "test \"\$(sha256sum '$ARCH' | cut -d ' ' -f 1)\" = $PIN && echo archive-sha-ok && wc -c <'$ARCH'"
run c1-syntax-bash-posix sh -c 'bash --posix -n tools/fetch-pinned-archive.sh && bash --posix -n tools/fetch-pinned-archive-selftest.sh && echo syntax-ok'
run c2-selftest-generated-payload sh tools/fetch-pinned-archive-selftest.sh
run c3-selftest-actual-pinned-archive sh tools/fetch-pinned-archive-selftest.sh "$ARCH" "$PIN"
run c4-selftest-actual-pinned-archive-bash-posix bash --posix tools/fetch-pinned-archive-selftest.sh "$ARCH" "$PIN"
run c5-mutation-sha-comparison-disabled-is-caught sh R:/d3-evidence/ci3-mutation-20261107.sh tools
run c6-workflow-static-vs-1b89 "$PY" -I -B -X utf8 R:/d3-evidence/ci3-workflow-static-20261107.py "$REPO"
run c7-real-curl-options-closed-loopback sh R:/d3-evidence/ci3-real-curl-options-20261107.sh tools
cd "$REPO" || exit 2
run c8-git-diff-check git diff --check HEAD
echo DONE >>"$P"
