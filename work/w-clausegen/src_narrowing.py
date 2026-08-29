#!/usr/bin/env python3
"""Would narrowing check 6 to `crates/*/src/` change which rows it flags?

The brief warns that narrowing the ABSENCE screen to `crates/*/src/` redefines
what `absent` MEANS. That warning is about check 5. This asks the separate,
measurable question for check 6, rather than asserting an answer.
"""
import csv, os

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
rows = list(csv.DictReader(
    [l for l in open(os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv'), encoding='utf-8')
     if not l.startswith('#')], delimiter='\t'))

flagged_all, flagged_src = [], []
for r in rows:
    if r['cites'].strip() in ('-', ''):
        continue
    paths = [p.strip() for p in r['cites'].split(',') if p.strip()]
    src = [p for p in paths if '/src/' in p]
    print(f"{r['id']:<4} {r['state']:<14} {len(src)} of {len(paths)} hits are under crates/*/src/")
    for p in paths:
        print(f"       {'src ' if '/src/' in p else 'NOT '} {p}")
    if r['state'] in ('absent', 'unexercisable'):
        flagged_all.append(r['id'])
        if src:
            flagged_src.append(r['id'])

print(f"\nabsent/unexercisable rows with ANY citation : {flagged_all}")
print(f"...and with a citation under crates/*/src/  : {flagged_src}")
print(f"\nnarrowing check 6 to crates/*/src/ would change the flagged set by "
      f"{len(flagged_all) - len(flagged_src)} row(s)")
