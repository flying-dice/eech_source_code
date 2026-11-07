#!/bin/sh
# Self-test of tools/fetch-pinned-archive.sh: runs the ACTUAL helper against a fake `curl` placed first on a
# process-local PATH (no network, no installed tool or global state changed).
#
#   tools/fetch-pinned-archive-selftest.sh                           a generated payload
#   tools/fetch-pinned-archive-selftest.sh <archive> <sha256>        the same matrix over the real pinned bytes
#
# Fails (non-zero) on any unmet expectation.
set -u

here=$(cd "$(dirname "$0")" && pwd)
helper=$here/fetch-pinned-archive.sh
work=$(mktemp -d) || exit 2
trap 'rm -rf -- "$work"' EXIT
trap 'exit 1' HUP INT TERM

mkdir -p "$work/bin" "$work/mode"
payload=$work/payload.bin
if [ "$#" -eq 2 ]; then
	cp -- "$1" "$payload" || exit 2
	pin=$2
else
	awk 'BEGIN { for (i = 0; i < 2000; i++) print "pinned archive stand-in line", i }' >"$payload"
	pin=$(sha256sum -- "$payload" | cut -d ' ' -f 1)
fi
if [ "$(sha256sum -- "$payload" | cut -d ' ' -f 1)" != "$pin" ]; then
	echo "selftest setup: the payload does not have the stated SHA-256" >&2
	exit 2
fi

# the fake transport: the behaviour of each URL class is read from mode/<primary|fallback>; every call is logged
cat >"$work/bin/curl" <<'FAKE'
#!/bin/sh
out=; url=; prev=
for a in "$@"; do
	[ "$prev" = -o ] && out=$a
	prev=$a
	url=$a
done
printf '%s\n' "$*" >>"$FAKE_DIR/log"
case $url in *primary*) cls=primary ;; *) cls=fallback ;; esac
mode=$(cat "$FAKE_DIR/mode/$cls")
case $mode in
	ok) cat "$FAKE_DIR/payload.bin" >"$out"; exit 0 ;;
	timeout) exit 28 ;;
	http) exit 22 ;;
	partial) head -c 100 "$FAKE_DIR/payload.bin" >"$out"; exit 18 ;;
	truncated) head -c 100 "$FAKE_DIR/payload.bin" >"$out"; exit 0 ;;
	corrupt) printf 'not the pinned archive' >"$out"; exit 0 ;;
	empty) : >"$out"; exit 0 ;;
	silent) exit 0 ;;
esac
exit 99
FAKE
chmod +x "$work/bin/curl"

P=https://primary.invalid/lua-5.1.5.tar.gz
F=https://fallback.invalid/lua-5.1.5.tar.gz
fails=0
n=0

# set the two modes and reset the call log; sources are always the primary first, then the fallback
modes() {
	printf '%s' "$1" >"$work/mode/primary"
	printf '%s' "$2" >"$work/mode/fallback"
	: >"$work/log"
}
# run the actual helper with the fake curl first on PATH; $rc and $work/err are the results
helper_run() {
	rc=0
	PATH="$work/bin:$PATH" FAKE_DIR=$work sh "$helper" "$@" 2>"$work/err" || rc=$?
}
calls() {
	wc -l <"$work/log" | tr -d ' '
}
dest_hash() {
	sha256sum -- "$1" | cut -d ' ' -f 1
}
# each case runs in its own empty directory, so a stray temporary file is visible
fresh() {
	d=$work/case.$n
	mkdir -p "$d"
}
expect() {
	n=$((n + 1))
	if [ "$2" = ok ]; then
		return 0
	fi
	fails=$((fails + 1))
	echo "FAIL: $1" >&2
	sed 's/^/    | /' "$work/err" >&2
}
check() { # check <description> <test args...>
	desc=$1
	shift
	if "$@"; then
		expect "$desc" ok
	else
		expect "$desc" no
	fi
}
equal() { [ "$1" = "$2" ]; }
only_dest() { # the directory holds exactly the destination (no temporary file left behind)
	equal "$(ls -A "$d" | tr '\n' ' ')" "$1 "
}
empty_dir() { equal "$(ls -A "$d")" ""; }

# coherent positive: the primary delivers; one call, bounded and HTTPS-only, the pinned bytes, nothing left over
fresh
modes ok timeout
helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
check "positive: exit 0" equal "$rc" 0
check "positive: pinned bytes at dest" equal "$(dest_hash "$d/lua.tar.gz")" "$pin"
check "positive: exactly one transfer, the primary" equal "$(calls)" 1
check "positive: the transfer is bounded" grep -q -e '--connect-timeout 15 --max-time 120' "$work/log"
check "positive: the transfer is https-only, redirects included" grep -q -e '--proto =https --proto-redir =https' "$work/log"
check "positive: no certificate bypass or credential option" sh -c '! grep -q -e " -k" -e "--insecure" -e "--user" -e " -u " "$1"' _ "$work/log"
check "positive: no temporary file left" only_dest lua.tar.gz

# a primary that cannot deliver the bytes (connection timeout, HTTP error, cut transfer, wrong, truncated or empty bytes):
# the fallback delivers the SAME pinned bytes, the primary's output is never used
for bad in timeout http partial truncated corrupt empty; do
	fresh
	modes "$bad" ok
	helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
	check "primary $bad: fallback succeeds" equal "$rc" 0
	check "primary $bad: pinned bytes at dest" equal "$(dest_hash "$d/lua.tar.gz")" "$pin"
	check "primary $bad: the primary was tried first, then the fallback" equal "$(sed -e 's/.* //' "$work/log" | tr '\n' ' ')" "$P $F "
	check "primary $bad: no temporary file left" only_dest lua.tar.gz
done

# every source fails, or delivers bytes that are not the pinned archive: non-zero and no approved destination
for pair in "timeout timeout" "corrupt corrupt" "truncated partial" "corrupt timeout" "corrupt silent" "empty http" "silent silent"; do
	set -- $pair
	fresh
	modes "$1" "$2"
	helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
	check "all sources fail ($pair): exit 1" equal "$rc" 1
	check "all sources fail ($pair): both sources were tried" equal "$(calls)" 2
	check "all sources fail ($pair): nothing at dest, no temporary file" empty_dir
done

# an existing valid destination is reused without any transfer
fresh
cp -- "$payload" "$d/lua.tar.gz"
modes timeout timeout
helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
check "existing valid dest: exit 0" equal "$rc" 0
check "existing valid dest: no transfer" equal "$(calls)" 0
check "existing valid dest: bytes unchanged" equal "$(dest_hash "$d/lua.tar.gz")" "$pin"

# an existing INVALID destination is never used: replaced by the verified bytes, or removed when nothing delivers them
fresh
printf 'a stale or partial download' >"$d/lua.tar.gz"
modes timeout ok
helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
check "existing invalid dest, fallback ok: exit 0" equal "$rc" 0
check "existing invalid dest, fallback ok: replaced by the pinned bytes" equal "$(dest_hash "$d/lua.tar.gz")" "$pin"
fresh
printf 'a stale or partial download' >"$d/lua.tar.gz"
modes timeout timeout
helper_run "$d/lua.tar.gz" "$pin" "$P" "$F"
check "existing invalid dest, all fail: exit 1" equal "$rc" 1
check "existing invalid dest, all fail: the invalid bytes are not left as the archive" empty_dir

# a destination path with a space and a relative path
fresh
mkdir "$d/with space"
modes ok ok
(cd "$d/with space" && PATH="$work/bin:$PATH" FAKE_DIR=$work sh "$helper" "lua archive.tar.gz" "$pin" "$P" 2>"$work/err")
rc=$?
check "path with a space: exit 0" equal "$rc" 0
check "path with a space: pinned bytes" equal "$(dest_hash "$d/with space/lua archive.tar.gz")" "$pin"

# usage errors: no transfer is made and nothing is created
for args in "" "dest" "dest $pin" "dest 2640 $P" "dest $(echo "$pin" | tr 'a-f' 'A-F') $P" "dest $pin http://primary.invalid/lua.tar.gz"; do
	fresh
	modes ok ok
	rc=0
	(cd "$d" && PATH="$work/bin:$PATH" FAKE_DIR=$work sh "$helper" $args 2>"$work/err") || rc=$?
	check "usage error ($args): exit 2" equal "$rc" 2
	check "usage error ($args): no transfer" equal "$(calls)" 0
	check "usage error ($args): nothing created" empty_dir
done
fresh
mkdir "$d/adir"
modes ok ok
helper_run "$d/adir" "$pin" "$P"
check "a directory as dest: exit 2" equal "$rc" 2
check "a directory as dest: no transfer" equal "$(calls)" 0

if [ "$fails" -ne 0 ]; then
	echo "fetch-pinned-archive selftest: $fails of $n checks FAILED" >&2
	exit 1
fi
echo "fetch-pinned-archive selftest: all $n checks passed (pin $pin)"
