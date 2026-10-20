#!/usr/bin/env python3
# EEDCS-11, test only: appends the hooks of patches.json to a copy of
# crates/eech-engine-sys/build/patches.rs, after the product patches, as Rust
# Patch entries. Used by tools/build-observed-windows.sh on its temporary
# worktree only; never on the product tree.
#
# Usage: insert-hooks.py <patches.rs to change> <patches.json>
import json, sys

path, hooks = sys.argv[1], json.load(open(sys.argv[2]))['patches']


def rust(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n').replace('\t', '\\t') + '"'


text = open(path, newline='').read()
end = text.index('\n];\n', text.index('pub const PATCHES'))
entries = ''.join(
    '\n    // EEDCS-11, TEST ONLY (tools/arrival-observe/patches.json)\n    Patch {\n'
    f"        file: {rust(h['file'])},\n        id: {rust(h['id'])},\n        why: {rust(h['why'])},\n"
    f"        original: {rust(h['original'])},\n        replacement: {rust(h['replacement'])},\n        count: {h['count']},\n    }},"
    for h in hooks)
open(path, 'w', newline='').write(text[:end] + entries + text[end:])
