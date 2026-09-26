#!/usr/bin/env python3
# Identifies a scenario's inputs: the SHA-256 of every file in an installation
# root (tools/retail-cvh.sh, tools/retail-map3-installs.sh), and one combined
# hash over them. The files the engine itself writes are listed apart, with
# their hashes, and left out of the combined hash:
#   cohokum/ballistics-data.txt, cohokum/guided-missiles-data.txt,
#   cohokum/game.cfg                   EECH's own exports at every boot
#   common/data/brief_en.dat           prepare_installation's briefing texts
#   cohokum/EECH.INI                   EECH's settings, written at the first
#                                      boot when absent (then read at every
#                                      boot); its floats are printed by the C
#                                      runtime, so a Linux and a Windows first
#                                      boot write slightly different files
#
# Usage:
#   tools/input-manifest.py <root> [-o <manifest.json>]
#   tools/input-manifest.py <root> --compare <manifest.json>    exit 1 if the inputs differ
import argparse, hashlib, json, os, sys

ENGINE_WRITTEN = {
    'cohokum/ballistics-data.txt',
    'cohokum/guided-missiles-data.txt',
    'cohokum/game.cfg',
    'common/data/brief_en.dat',
    'cohokum/EECH.INI',
}


def manifest(root):
    files, written = [], []
    for base, dirs, names in os.walk(root):
        dirs.sort()
        for name in sorted(names):
            path = os.path.join(base, name)
            rel = os.path.relpath(path, root).replace(os.sep, '/')
            h = hashlib.sha256()
            with open(path, 'rb') as f:
                for block in iter(lambda: f.read(1 << 20), b''):
                    h.update(block)
            entry = {'path': rel, 'size': os.path.getsize(path), 'sha256': h.hexdigest()}
            (written if rel in ENGINE_WRITTEN else files).append(entry)
    files.sort(key=lambda e: e['path'])
    combined = hashlib.sha256(''.join(f"{e['sha256']}  {e['path']}\n" for e in files).encode()).hexdigest()
    return {'files': len(files), 'bytes': sum(e['size'] for e in files), 'combined_sha256': combined,
            'inputs': files, 'engine_written': sorted(written, key=lambda e: e['path'])}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('root')
    p.add_argument('-o', '--output')
    p.add_argument('--compare')
    a = p.parse_args()
    m = manifest(a.root)
    print(f"{a.root}: {m['files']} input files, {m['bytes']:,} bytes, combined SHA-256 {m['combined_sha256']}")
    if a.output:
        with open(a.output, 'w', newline='\n') as f:
            json.dump(m, f, indent=1)
            f.write('\n')
    if a.compare:
        other = json.load(open(a.compare))
        mine = {e['path']: e['sha256'] for e in m['inputs']}
        theirs = {e['path']: e['sha256'] for e in other['inputs']}
        only_here = sorted(set(mine) - set(theirs))
        only_there = sorted(set(theirs) - set(mine))
        changed = sorted(k for k in set(mine) & set(theirs) if mine[k] != theirs[k])
        for label, items in (('only in this root', only_here), ('only in the manifest', only_there), ('different content', changed)):
            for k in items:
                print(f'  {label}: {k}')
        if only_here or only_there or changed:
            print('inputs DIFFER')
            return 1
        print(f"inputs IDENTICAL to {a.compare}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
