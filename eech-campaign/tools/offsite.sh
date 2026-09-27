#!/bin/sh
# Offsite storage for large files shared between sessions: one Google Drive
# folder (the "slice"), reached with rclone. It holds install data (retail
# EECH data, generated install roots) and run outputs (Tacview recordings,
# logs).
#
# Configuration, from the environment (set once in the cloud environment's
# settings; rclone reads RCLONE_CONFIG_<REMOTE>_<OPTION> itself):
#   RCLONE_CONFIG_EECHDRIVE_TYPE=drive
#   RCLONE_CONFIG_EECHDRIVE_SCOPE=drive
#   RCLONE_CONFIG_EECHDRIVE_TOKEN={"access_token":...,"refresh_token":...}   (from `rclone authorize drive`)
#   RCLONE_CONFIG_EECHDRIVE_ROOT_FOLDER_ID=<id of the Drive folder that is the slice>
#
# Layout in the slice:
#   install/<name>.tar.gz     an install root or data set, packed
#   outputs/<name>/...        run outputs, as files
#
# Usage:
#   tools/offsite.sh check
#   tools/offsite.sh list [install|outputs]
#   tools/offsite.sh push-install <name> <directory>     pack <directory> as install/<name>.tar.gz
#   tools/offsite.sh pull-install <name> <directory>     unpack install/<name>.tar.gz into <directory>
#   tools/offsite.sh push-output <name> <file>...        copy files to outputs/<name>/
#   tools/offsite.sh pull-output <name> <directory>      copy outputs/<name>/ into <directory>
set -eu

REMOTE=eechdrive:
RCLONE=${RCLONE:-rclone}

need_rclone() {
	if ! command -v "$RCLONE" >/dev/null 2>&1; then
		dir=${HOME}/.local/bin
		mkdir -p "$dir"
		tmp=$(mktemp -d)
		curl -sSL -o "$tmp/rclone.zip" https://downloads.rclone.org/rclone-current-linux-amd64.zip
		(cd "$tmp" && unzip -q rclone.zip && cp rclone-*-linux-amd64/rclone "$dir/")
		RCLONE=$dir/rclone
	fi
	if [ -z "${RCLONE_CONFIG_EECHDRIVE_TOKEN:-}" ] || [ -z "${RCLONE_CONFIG_EECHDRIVE_ROOT_FOLDER_ID:-}" ]; then
		echo "offsite: RCLONE_CONFIG_EECHDRIVE_TOKEN and RCLONE_CONFIG_EECHDRIVE_ROOT_FOLDER_ID must be set (see the header of $0)" >&2
		exit 2
	fi
	export RCLONE_CONFIG_EECHDRIVE_TYPE=${RCLONE_CONFIG_EECHDRIVE_TYPE:-drive}
	export RCLONE_CONFIG_EECHDRIVE_SCOPE=${RCLONE_CONFIG_EECHDRIVE_SCOPE:-drive}
}

cmd=${1:-}
[ $# -gt 0 ] && shift
case "$cmd" in
check)
	need_rclone
	"$RCLONE" about "$REMOTE" && "$RCLONE" lsd "$REMOTE"
	;;
list)
	need_rclone
	"$RCLONE" lsl "$REMOTE${1:-}"
	;;
push-install)
	need_rclone
	name=$1 dir=$2
	tar -C "$dir" -czf - . | "$RCLONE" rcat --progress "${REMOTE}install/$name.tar.gz"
	;;
pull-install)
	need_rclone
	name=$1 dir=$2
	mkdir -p "$dir"
	"$RCLONE" cat "${REMOTE}install/$name.tar.gz" | tar -C "$dir" -xzf -
	;;
push-output)
	need_rclone
	name=$1
	shift
	for f in "$@"; do
		"$RCLONE" copy --progress "$f" "${REMOTE}outputs/$name/"
	done
	;;
pull-output)
	need_rclone
	name=$1 dir=$2
	"$RCLONE" copy --progress "${REMOTE}outputs/$name/" "$dir"
	;;
*)
	sed -n '2,32p' "$0" | sed 's/^# \{0,1\}//'
	exit 1
	;;
esac
