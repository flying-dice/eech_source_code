#!/bin/sh
# The campaign regression test: runs the retail Lebanon and Georgia campaigns
# for a few simulated hours each, in parallel, without recording, and
# compares their campaign metrics (sorties, weapons, losses, regen, captures,
# keysites, supply, forces; lua/metrics.lua) with the committed baselines in
# regression/. About 20 minutes of wall time. Linux (or the eech-build
# container: tools/regress-docker.sh does it all from Windows).
#
# Usage: tools/regress.sh <georgia root> <lebanon root> [--update] [--exact]
#   roots from tools/retail-map3-installs.sh and tools/retail-cvh.sh
#   --update  writes the runs' metrics as the new baselines
#   --exact   fails unless the runs reproduce the baselines exactly
#
# Environment: EECH_WORLD (default target/release/eech-world), REGRESS_OUT
# (default target/regress), REGRESS_HOURS (default 3; the baselines are for 3).
set -eu
georgia=$1 lebanon=$2
shift 2
update=0 exact=""
for a in "$@"; do
	case $a in
		--update) update=1 ;;
		--exact) exact=--exact ;;
		*) echo "unknown option $a" >&2; exit 2 ;;
	esac
done
here=$(cd "$(dirname "$0")/.." && pwd)
world=${EECH_WORLD:-$here/target/release/eech-world}
out=${REGRESS_OUT:-$here/target/regress}
hours=${REGRESS_HOURS:-3}
mkdir -p "$out"

run () { # scenario root
	"$world" "$here/crates/eech-world/lua/campaign.lua" root="$2" scenario="$1" hours="$hours" \
		record=0 checkpoint_every=3600 metrics="$out/$1.json" > "$out/$1.log" 2>&1
}

start=$(date +%s)
echo "running georgia_retail and lebanon_retail for $hours simulated hours each ..."
run georgia_retail "$georgia" & g=$!
run lebanon_retail "$lebanon" & l=$!
status=0
wait $g || { echo "georgia_retail failed: see $out/georgia_retail.log" >&2; status=1; }
wait $l || { echo "lebanon_retail failed: see $out/lebanon_retail.log" >&2; status=1; }
echo "runs took $(( $(date +%s) - start )) s"
[ $status -eq 0 ] || exit $status

for s in georgia_retail lebanon_retail; do
	base=$here/regression/$s.json
	echo
	if [ $update -eq 1 ] || [ ! -f "$base" ]; then
		cp "$out/$s.json" "$base"
		python3 "$here/tools/regress-compare.py" "$base"
		echo "baseline written: regression/$s.json"
	elif python3 "$here/tools/regress-compare.py" "$base" "$out/$s.json" $exact; then
		echo "$s: PASS"
	else
		echo "$s: FAIL"
		status=1
	fi
done
exit $status
