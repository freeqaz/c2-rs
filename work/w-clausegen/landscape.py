#!/usr/bin/env python3
"""Scratch: what does crates/ cite, per clause row? Design input, not a check."""
import csv, subprocess, os

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def grep(args):
    r = subprocess.run(['git', '-C', REPO, 'grep', '-l', '--untracked',
                        '--exclude-standard', '-F'] + args, capture_output=True, text=True)
    return [p for p in r.stdout.split() if p]


rows = list(csv.DictReader(
    [l for l in open(os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv'))
     if not l.startswith('#')], delimiter='\t'))
for r in rows:
    a = r['addr']
    ah = grep(['--', '0x' + a, '--', 'crates/'])
    tok = r['witness'][5:] if r['witness'].startswith('none:') else ''
    th = grep(['-w', '--', tok, '--', 'crates/']) if tok else []
    print(f"{r['id']:<4} {a} {r['state']:<14}")
    print(f"      addr : {' '.join(p[7:] for p in ah) or '—'}")
    if tok:
        print(f"      tok  : {tok} -> {' '.join(p[7:] for p in th) or '—'}")
