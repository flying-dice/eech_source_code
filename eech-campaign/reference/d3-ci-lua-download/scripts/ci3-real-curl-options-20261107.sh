#!/bin/sh
# The REAL curl accepts the helper's option set and the helper then fails closed: one source on a closed LOOPBACK port (no external network, no other process contacted).
# Exit 0 = the helper exits 1, curl reports exit 7 (connection refused: the options parsed), nothing at the destination and no temporary file. Usage: ci3-real-curl-options-20261107.sh <repo>/eech-campaign/tools
tools=$1
t=$(mktemp -d) || exit 2
trap 'rm -rf -- "$t"' EXIT
curl --version | head -1
sh "$tools/fetch-pinned-archive.sh" "$t/lua.tar.gz" 2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333 https://127.0.0.1:9/lua-5.1.5.tar.gz 2>"$t/err"
rc=$?
cat "$t/err"
left=$(ls -A "$t" | grep -v '^err$' | tr '\n' ' ')
echo "helper exit $rc; files besides the log: [${left}]"
if [ "$rc" -eq 1 ] && grep -q 'curl exit 7' "$t/err" && [ -z "$left" ]; then
	exit 0
fi
exit 1
