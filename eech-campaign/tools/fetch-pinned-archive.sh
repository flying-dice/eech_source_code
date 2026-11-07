#!/bin/sh
# Downloads ONE archive whose SHA-256 is pinned, from the first source that delivers exactly those bytes.
#
#   tools/fetch-pinned-archive.sh <dest> <sha256> <url> [<url> ...]
#
# The URLs are tried in order, each with a bounded connect and overall time. Only bytes whose SHA-256 equals
# <sha256> are ever moved to <dest>; a failed, corrupt, truncated or empty transfer is discarded, so a later
# step can extract <dest> without checking it again. A <dest> that already exists is reused only when it has the
# pinned hash; otherwise it is removed (it is never silently used). Network availability is not source identity:
# an alternative URL is acceptable exactly because the pinned digest is the identity. HTTPS only, no certificate
# bypass, no credentials.
# Exit 0: <dest> holds the pinned bytes. Exit 1: no source delivered them and <dest> does not exist. Exit 2: usage.
set -u

CONNECT_TIMEOUT=15
MAX_TIME=120

if [ "$#" -lt 3 ]; then
	echo "usage: $0 <dest> <sha256> <https-url> [<https-url> ...]" >&2
	exit 2
fi
dest=$1
want=$2
shift 2

case $want in
	*[!0-9a-f]*) want= ;;
esac
if [ -z "$dest" ] || [ "${#want}" -ne 64 ]; then
	echo "fetch-pinned-archive: <dest> must be set and <sha256> must be 64 lowercase hex digits" >&2
	exit 2
fi
if [ -d "$dest" ]; then
	echo "fetch-pinned-archive: $dest is a directory" >&2
	exit 2
fi
for url in "$@"; do
	case $url in
		https://?*) ;;
		*)
			echo "fetch-pinned-archive: only https URLs are accepted: $url" >&2
			exit 2
			;;
	esac
done

# the hash of a file, empty when it cannot be computed (which then never equals the pin)
hash_of() {
	sha256sum -- "$1" 2>/dev/null | cut -d ' ' -f 1
}

if [ -e "$dest" ]; then
	if [ "$(hash_of "$dest")" = "$want" ]; then
		echo "fetch-pinned-archive: reusing $dest (SHA-256 $want)" >&2
		exit 0
	fi
	echo "fetch-pinned-archive: existing $dest is not the pinned archive; removing it" >&2
	rm -f -- "$dest" || exit 2
fi

tmp=$(mktemp "$dest.XXXXXX") || exit 2
trap 'rm -f -- "$tmp"' EXIT
trap 'exit 1' HUP INT TERM

for url in "$@"; do
	: >"$tmp"
	curl -fsSL --proto '=https' --proto-redir '=https' --retry 0 \
		--connect-timeout "$CONNECT_TIMEOUT" --max-time "$MAX_TIME" -o "$tmp" "$url"
	rc=$?
	if [ "$rc" -ne 0 ]; then
		echo "fetch-pinned-archive: $url failed: curl exit $rc" >&2
		continue
	fi
	got=$(hash_of "$tmp")
	if [ "$got" != "$want" ]; then
		echo "fetch-pinned-archive: $url delivered the wrong bytes (SHA-256 ${got:-unreadable}, pinned $want); discarded" >&2
		continue
	fi
	chmod 644 "$tmp" && mv -f -- "$tmp" "$dest" || exit 2
	echo "fetch-pinned-archive: $dest from $url (SHA-256 $want)" >&2
	exit 0
done
echo "fetch-pinned-archive: no source delivered the pinned archive" >&2
exit 1
