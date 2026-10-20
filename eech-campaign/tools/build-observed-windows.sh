#!/bin/sh
# EEDCS-11, TEST ONLY: builds the Windows engine with the arrival-observation
# hooks of tools/arrival-observe/ added (OBSERVED), or the same build without
# them (CONTROL), without touching this working tree or the normal build:
#
#   - the build comes from a temporary git worktree of this checkout's HEAD
#     (which must be clean), under target/observed/, removed afterwards (never
#     `git worktree prune`: docs/m4-s3-mutation.md);
#   - in that worktree only, the hooks in tools/arrival-observe/patches.json are
#     appended to crates/eech-engine-sys/build/patches.rs, after the product
#     patches, and tools/arrival-observe/eech_arrival_observe.c is copied into
#     crates/eech-engine-sys/csrc/; the change is checked to be exactly that;
#   - the worktree's own tools/build-windows.sh builds it, in a cargo target
#     volume of its own (eech-win-target-observed-<mode>), so the normal volume
#     never compiles observation code;
#   - the output goes only under target/observed/, stamped OBSERVED or CONTROL,
#     with the source diff (observation.diff) and the hook markers found in the
#     staged sources the build compiled (hook-markers.txt).
#
# The observer writes nothing unless EECH_ARRIVAL_TRACE names a directory
# (tools/arrival-observe/eech_arrival_observe.c). An OBSERVED build is evidence
# only when it reproduces the accepted baseline exactly (docs/m5-arrival-characterisation.md).
#
# Usage (Git Bash, Docker): tools/build-observed-windows.sh <observed | controlled | control> target/observed/<dir>
#   controlled: OBSERVED plus the helicopter discriminator's test-only control (patches-control.json,
#   eech_arrival_control.c), dormant unless EECH_ARRIVAL_CONTROL is set (docs/m5-helicopter-discriminator.md)
set -eu
mode=$1 out=$2
case $mode in observed | controlled | control) ;; *) echo "mode: observed, controlled or control" >&2; exit 2 ;; esac
case $out in target/observed/*) ;; *) echo "the output must be under target/observed/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
patches=crates/eech-engine-sys/build/patches.rs
csrc=crates/eech-engine-sys/csrc
commit=$(cd "$here" && git rev-parse HEAD)
dirty=$(cd "$here" && git status --porcelain -- . ../aphavoc ../modules | grep -v '^?? ' | wc -l | tr -d ' ')
if [ "$dirty" != 0 ]; then echo "$dirty uncommitted changes: an observed build is made from a clean commit" >&2; exit 2; fi
python "$here/tools/arrival-observe/verify-patches.py" $( [ "$mode" = controlled ] && echo --controlled ) > /dev/null || { echo "tools/arrival-observe/verify-patches.py failed" >&2; exit 1; }
volume=eech-win-target-observed-$mode

wt=$here/target/observed/worktree-$mode-$$
wtw=$(cygpath -m "$wt" 2>/dev/null || echo "$wt")
cleanup() { cd "$here" && git worktree remove --force "$wtw" >/dev/null 2>&1; rm -rf "$wt"; }
trap cleanup EXIT
trap 'exit 1' INT TERM
mkdir -p "$here/target/observed"
(cd "$here" && git worktree add --detach --no-checkout "$wtw" "$commit" >/dev/null 2>&1)
(cd "$wt" && git sparse-checkout set --cone eech-campaign aphavoc modules >/dev/null && git checkout -q --detach "$commit")
if [ ! -f "$wt/eech-campaign/$patches" ]; then echo "the worktree checkout is incomplete" >&2; exit 1; fi

if [ "$mode" != control ]; then
	python "$here/tools/arrival-observe/insert-hooks.py" "$wt/eech-campaign/$patches" "$here/tools/arrival-observe/patches.json"
	cp "$here/tools/arrival-observe/eech_arrival_observe.c" "$wt/eech-campaign/$csrc/"
	expected=" M eech-campaign/$patches ?? eech-campaign/$csrc/eech_arrival_observe.c "
	if [ "$mode" = controlled ]; then
		# the helicopter discriminator: the control hook after the observation hooks, and its dormant module
		python "$here/tools/arrival-observe/insert-hooks.py" "$wt/eech-campaign/$patches" "$here/tools/arrival-observe/patches-control.json"
		cp "$here/tools/arrival-observe/eech_arrival_control.c" "$wt/eech-campaign/$csrc/"
		expected=" M eech-campaign/$patches ?? eech-campaign/$csrc/eech_arrival_control.c ?? eech-campaign/$csrc/eech_arrival_observe.c "
	fi
	changed=$(cd "$wt" && git status --porcelain | sort | tr '\n' ' ')
	if [ "$changed" != "$expected" ]; then echo "the observation changed more than $patches and the test-only modules: $changed" >&2; exit 1; fi
	if (cd "$wt" && git diff -U0 | grep '^-[^-]' >/dev/null); then echo "the observation removed lines from $patches" >&2; exit 1; fi
fi

# build-windows.sh prints compiler errors on stdout: keep them, drop the rest
(cd "$wt/eech-campaign" && EECH_WIN_TARGET=$volume sh tools/build-windows.sh target/observed-out) > "$here/target/observed/build-$mode.log" 2>&1 \
	|| { tail -30 "$here/target/observed/build-$mode.log" >&2; exit 1; }
rm -rf "$here/$out"
mkdir -p "$here/$out"
cp -r "$wt/eech-campaign/target/observed-out/." "$here/$out/"
(cd "$wt" && git add -N . && git diff) > "$here/$out/observation.diff"
# the hook markers ("EECH observe (A<n>)") in the sources this build staged and compiled
docker run --rm -v "$volume:/t:ro" eech-build sh -c 'tree=$(ls -dt /t/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/tree | head -1); grep -rhoE "EECH (observe|control) \(A[0-9]+\)" "$tree" | sort -u' > "$here/$out/hook-markers.txt" || true
{
	case $mode in
		control) echo "CONTROL: commit $commit, no observation hooks" ;;
		observed) echo "OBSERVED (test only, not a product build): commit $commit with the hooks of tools/arrival-observe/" ;;
		controlled) echo "CONTROLLED (test only, not a product build): commit $commit with the hooks of tools/arrival-observe/ and its control hook A13 (dormant unless EECH_ARRIVAL_CONTROL is set)" ;;
	esac
	cat "$here/$out/BUILD-INFO.txt"
} > "$here/$out/BUILD-INFO.tmp" && mv "$here/$out/BUILD-INFO.tmp" "$here/$out/BUILD-INFO.txt"
[ "$mode" = control ] || echo "test-only observed build: tools/arrival-observe/ hooks added; see BUILD-INFO.txt" > "$here/$out/OBSERVED.txt"
head -3 "$here/$out/BUILD-INFO.txt"
echo "hook markers: $(tr '\n' ' ' < "$here/$out/hook-markers.txt")"
