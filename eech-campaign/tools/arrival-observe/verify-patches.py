#!/usr/bin/env python3
# EEDCS-11, test only: checks, without building, that every observation hook in
# patches.json applies to the original EECH text after the product patches of
# crates/eech-engine-sys/build/patches.rs, exactly `count` times, as the build's
# own assertion (patches::apply) will require; and that each replacement keeps
# its original text (or each statement it lists in "keeps"), so a hook only adds.
#
# Usage: python tools/arrival-observe/verify-patches.py [--controlled]   (from eech-campaign/)
#   --controlled: also the control hook of patches-control.json, after the observation hooks
import json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
CAMPAIGN = os.path.dirname(os.path.dirname(HERE))
REPO = os.path.dirname(CAMPAIGN)


def rust_string(s):
    # the subset of Rust string escapes patches.rs uses
    return re.sub(r'\\(n|t|r|"|\\)', lambda m: {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\'}[m.group(1)], s)


def product_patches():
    text = open(os.path.join(CAMPAIGN, 'crates/eech-engine-sys/build/patches.rs'), encoding='utf-8').read()
    out = []
    for block in re.finditer(r'Patch \{(.*?)\n    \},', text, re.S):
        body = block.group(1)
        field = lambda k: rust_string(re.search(k + r': "((?:[^"\\]|\\.)*)"', body, re.S).group(1))
        out.append({'file': field('file'), 'id': field('id'), 'original': field('original'), 'replacement': field('replacement'),
                    'count': int(re.search(r'count: (\d+)', body).group(1))})
    return out


def main():
    hooks = json.load(open(os.path.join(HERE, 'patches.json')))['patches']
    if '--controlled' in sys.argv:
        # the helicopter discriminator's control hook, applied after the observation hooks
        hooks += json.load(open(os.path.join(HERE, 'patches-control.json')))['patches']
    product = product_patches()
    bad = 0
    for file in sorted({h['file'] for h in hooks}):
        text = open(os.path.join(REPO, file), 'rb').read().decode('latin-1')
        for p in [p for p in product if p['file'] == file] + [h for h in hooks if h['file'] == file]:
            found = text.count(p['original'])
            kind = 'hook' if p in hooks else 'product'
            if found != p['count']:
                print(f"FAIL {kind} {p['id']}: {found} occurrence(s) in {file}, expected {p['count']}")
                bad += 1
                continue
            # a hook only adds: it keeps its original text, or (A1, which must pass the
            # verdict through) each statement it lists in "keeps", exactly once
            for keep in (p.get('keeps') or [p['original']]) if kind == 'hook' else []:
                if p['replacement'].count(keep) != 1:
                    print(f"FAIL hook {p['id']}: the replacement does not keep {keep!r} exactly once")
                    bad += 1
            text = text.replace(p['original'], p['replacement'])
            print(f"ok   {kind} {p['id']} ({found}x in {file})")
    print('PASS' if not bad else f'FAIL ({bad})')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
