#!/bin/sh
# Sanity control of the selftest itself: in a TEMPORARY copy of the helper whose SHA-256 comparison is disabled (the one line that decides whether downloaded bytes are accepted), the selftest MUST fail.
# Exit 0 = the mutation was caught (selftest failed with at least one FAIL line); exit 1 = the selftest did not notice. The repository files are never modified.
# Usage: ci3-mutation-20261107.sh <repo>/eech-campaign/tools
tools=$1
t=$(mktemp -d) || exit 2
trap 'rm -rf -- "$t"' EXIT
cp -- "$tools/fetch-pinned-archive.sh" "$tools/fetch-pinned-archive-selftest.sh" "$t/" || exit 2
before=$(grep -c 'if \[ "\$got" != "\$want" \]; then' "$t/fetch-pinned-archive.sh")
if [ "$before" -ne 1 ]; then
	echo "mutation target not found exactly once: $before" >&2
	exit 2
fi
sed 's/if \[ "\$got" != "\$want" \]; then/if [ "$got" != "$got" ]; then/' "$t/fetch-pinned-archive.sh" >"$t/mutated" && mv "$t/mutated" "$t/fetch-pinned-archive.sh"
sh "$t/fetch-pinned-archive-selftest.sh" >"$t/out" 2>"$t/err"
rc=$?
fails=$(grep -c '^FAIL:' "$t/err")
echo "mutated helper: selftest exit $rc, $fails FAIL lines"
grep '^FAIL:' "$t/err" | head -8
if [ "$rc" -ne 0 ] && [ "$fails" -ge 1 ]; then
	exit 0
fi
exit 1
