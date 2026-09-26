#!/bin/sh
# Builds a test-only Windows engine with one source patch left out (a mutant),
# or the same build with nothing left out (the control), without touching this
# working tree or the normal build:
#
#   - the build comes from a temporary git worktree of this checkout's HEAD
#     (which must be clean), under target/mutants/, removed afterwards;
#   - in that worktree only, the patch's entry is removed from
#     crates/eech-engine-sys/build/patches.rs, and the change is checked to be
#     that one entry and nothing else;
#   - the worktree's own tools/build-windows.sh builds it, in a cargo target
#     volume of its own (eech-win-target-mutants-<id>), so the normal volume
#     never compiles mutant code;
#   - the output goes only under target/mutants/, stamped MUTANT or CONTROL,
#     with the source diff (mutation.diff) and the patch markers found in the
#     staged sources the build compiled (patch-markers.txt).
#
# Usage (Git Bash, Docker): tools/build-mutant-windows.sh <patch id | control> target/mutants/<dir>
#   e.g. tools/build-mutant-windows.sh S3-pick-up-the-tasks-cargo target/mutants/s3/mutant-bin
set -eu
id=$1 out=$2
case $out in target/mutants/*) ;; *) echo "the output must be under target/mutants/" >&2; exit 2 ;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
patches=crates/eech-engine-sys/build/patches.rs
commit=$(cd "$here" && git rev-parse HEAD)
dirty=$(cd "$here" && git status --porcelain -- . ../aphavoc ../modules | grep -v '^?? ' | wc -l | tr -d ' ')
if [ "$dirty" != 0 ]; then echo "$dirty uncommitted changes: a mutant is built from a clean commit" >&2; exit 2; fi
if [ "$id" != control ] && ! grep -q "id: \"$id\"" "$here/$patches"; then echo "no patch with id $id in $patches" >&2; exit 2; fi
volume=eech-win-target-mutants-$(echo "$id" | tr 'A-Z' 'a-z' | tr -c 'a-z0-9\n' '-')

wt=$here/target/mutants/worktree-$id-$$
wtw=$(cygpath -m "$wt" 2>/dev/null || echo "$wt")
cleanup() { cd "$here" && git worktree remove --force "$wtw" >/dev/null 2>&1; rm -rf "$wt"; git worktree prune; }
trap cleanup EXIT
trap 'exit 1' INT TERM
mkdir -p "$here/target/mutants"
(cd "$here" && git worktree add --detach --no-checkout "$wtw" "$commit" >/dev/null 2>&1)
(cd "$wt" && git sparse-checkout set --cone eech-campaign aphavoc modules >/dev/null && git checkout -q --detach "$commit")
if [ ! -f "$wt/eech-campaign/$patches" ]; then echo "the worktree checkout is incomplete" >&2; exit 1; fi

if [ "$id" != control ]; then
	python - "$wt/eech-campaign/$patches" "$id" <<'EOF'
import sys
path, pid = sys.argv[1], sys.argv[2]
text = open(path, newline='').read()
i = text.index(f'id: "{pid}"')
start = text.rindex('    Patch {', 0, i)
end = text.index('\n', text.index('    },', i)) + 1
open(path, 'w', newline='').write(text[:start] + text[end:])
EOF
	changed=$(cd "$wt" && git diff --name-only)
	if [ "$changed" != "eech-campaign/$patches" ]; then echo "the mutation changed more than $patches: $changed" >&2; exit 1; fi
	if (cd "$wt" && git diff -U0 | grep '^+[^+]' >/dev/null); then echo "the mutation added lines" >&2; exit 1; fi
fi

(cd "$wt/eech-campaign" && EECH_WIN_TARGET=$volume sh tools/build-windows.sh target/mutant-out) >/dev/null
rm -rf "$here/$out"
mkdir -p "$here/$out"
cp -r "$wt/eech-campaign/target/mutant-out/." "$here/$out/"
(cd "$wt" && git diff) > "$here/$out/mutation.diff"
# the patch markers ("EECH headless (<id>)") in the sources this build staged and compiled
docker run --rm -v "$volume:/t:ro" eech-build sh -c 'tree=$(ls -dt /t/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/tree | head -1); grep -rhoE "EECH headless \([A-Za-z0-9]+\)" "$tree" | sort -u' > "$here/$out/patch-markers.txt"
{
	if [ "$id" = control ]; then echo "CONTROL: commit $commit, no patch removed"; else echo "MUTANT (test only, not a product build): commit $commit with patch $id removed"; fi
	cat "$here/$out/BUILD-INFO.txt"
} > "$here/$out/BUILD-INFO.tmp" && mv "$here/$out/BUILD-INFO.tmp" "$here/$out/BUILD-INFO.txt"
[ "$id" = control ] || echo "test-only mutant: $id removed from $patches; see BUILD-INFO.txt" > "$here/$out/MUTANT.txt"
head -3 "$here/$out/BUILD-INFO.txt"
echo "patch markers: $(tr '\n' ' ' < "$here/$out/patch-markers.txt")"
