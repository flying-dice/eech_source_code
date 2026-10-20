#!/usr/bin/env python3
"""Compares two regression-pack logs/ directories file by file and line by line.

Both trees are walked, so a file or directory present on one side only is reported. A symbolic
link or a Windows junction is reported and not followed; an unreadable directory is an error. Before comparing, CRLF becomes LF and each --mask regular
expression is replaced by MASKED, so that the two runs' own worktree and candidate names do not
count as differences; the masks used are printed. No other normalisation is done: a lone CR, a
form feed or a missing final newline is a difference. A file that is not valid UTF-8 is compared
byte for byte, unmasked, and only reported as differing. Text diffs are printed in full.

Exit 0 when every file is the same after masking, 1 when any differs, is one-sided or is a link,
2 on bad arguments, an unreadable file, or two trees with no file to compare.

Usage: python -I compare-logs.py <a logs dir> <b logs dir> [--mask REGEX ...]
"""
import argparse
import difflib
import os
import re
import sys


def raise_error(e):
    raise e


def is_link(path):
    """A symbolic link, or a Windows junction (Python 3.12 and later can tell)."""
    return os.path.islink(path) or getattr(os.path, 'isjunction', lambda p: False)(path)


def entries(root):
    """Every file, directory and link under root, relative and '/'-separated, mapped to its kind."""
    found = {}
    for d, dirs, names in os.walk(root, onerror=raise_error):
        for n in dirs + names:
            path = os.path.join(d, n)
            kind = 'link' if is_link(path) else 'dir' if n in dirs else 'file'
            found[os.path.relpath(path, root).replace(os.sep, '/')] = kind
    return found


def read_masked(path, masks):
    """(lines after CRLF→LF and masking, raw bytes), or (None, raw bytes) when not UTF-8."""
    raw = open(path, 'rb').read()
    try:
        text = raw.decode('utf-8')
    except UnicodeDecodeError:
        return None, raw
    text = text.replace('\r\n', '\n')
    for m in masks:
        text = m.sub('MASKED', text)
    return text.split('\n'), raw


def compare_file(name, a_path, b_path, masks):
    """Prints the file's verdict and diff; returns True when the two are the same after masking."""
    (ta, ra), (tb, rb) = read_masked(a_path, masks), read_masked(b_path, masks)
    if ta is None or tb is None:
        same = ra == rb
        print(f'== {name}: {"same bytes" if same else "DIFFERS"} (not UTF-8, compared unmasked)')
        return same
    diff = list(difflib.unified_diff(ta, tb, 'a/' + name, 'b/' + name, lineterm='', n=0))
    print(f'== {name}: {"same after masking" if not diff else "DIFFERS"}')
    for line in diff:
        print('   ' + repr(line)[1:-1])
    return not diff


def compare(a_dir, b_dir, masks):
    """Prints one verdict per entry; returns (files compared, entries that differ)."""
    ea, eb = entries(a_dir), entries(b_dir)
    compared = bad = 0
    for name in sorted(ea.keys() | eb.keys()):
        kinds = ea.get(name), eb.get(name)
        if None in kinds:
            print(f'== {name}: {kinds[0] or kinds[1]} ONLY IN {"a" if kinds[0] else "b"}')
            bad += 1
        elif 'link' in kinds:
            print(f'== {name}: LINK, not followed or compared')
            bad += 1
        elif kinds[0] != kinds[1]:
            print(f'== {name}: {kinds[0]} in a, {kinds[1]} in b')
            bad += 1
        elif kinds[0] == 'file':
            compared += 1
            bad += not compare_file(name, os.path.join(a_dir, name), os.path.join(b_dir, name), masks)
    return compared, bad


def pattern(text):
    try:
        return re.compile(text)
    except re.error as e:
        raise argparse.ArgumentTypeError(f'bad regular expression {text!r}: {e}')


def main(argv):
    sys.stdout.reconfigure(encoding='utf-8', errors='backslashreplace')
    parser = argparse.ArgumentParser(description='Compares two regression-pack logs/ directories.')
    parser.add_argument('a_dir')
    parser.add_argument('b_dir')
    parser.add_argument('--mask', action='append', default=[], type=pattern, help='regular expression to mask')
    args = parser.parse_args(argv[1:])
    if not (os.path.isdir(args.a_dir) and os.path.isdir(args.b_dir)):
        print('both arguments must be directories', file=sys.stderr)
        return 2
    print('masks:', [m.pattern for m in args.mask] or 'none')
    try:
        compared, bad = compare(args.a_dir, args.b_dir, args.mask)
    except OSError as e:
        print(f'UNREADABLE: {e}')
        return 2
    print(f'{compared} file(s) compared; {bad} differ, are one-sided or are links')
    if not compared and not bad:
        print('NOTHING TO COMPARE')
        return 2
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
