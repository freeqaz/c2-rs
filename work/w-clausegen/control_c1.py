#!/usr/bin/env python3
"""CONTROL C-1 -- the retrospective one, registered in `PREREG.md` §4.

This is not a plant. It replays a false negative that **really happened** and
asks whether check 6 would have caught it.

The sequence being replayed, all of it in this repo's git history:

  8b4ca972c^   lane `w-inlbudget` has not landed. C14 and C18 read `absent`,
               tokens `INLINE_MAX_DEPTH` / `FORTY_INSTRS`, and nothing under
               `crates/` cites `0x10b60a1c` or `0x10b625b6`. A `cites` cell
               frozen at this moment reads `-` for both, correctly.
  8b4ca972c    `w-inlbudget` adopts the budget model. `splice.rs` now cites
               BOTH addresses, twice each, under the names
               `INLINE_LEVEL_DEPTH_CAP` and `INLINE_CHARGE_EXEMPT_MAX`.
  72caf2586    the wave-18 merge repair. C14 and C18 STILL read `absent`, and
               `check_table.py` prints GREEN, because the tokens the rows cite
               are still genuinely absent. **The counterparts existed for a
               full wave and no instrument could say so.**

So: freeze `cites` at `8b4ca972c^`, grade at `72caf2586`, and read both
verdicts. Check 5 must be GREEN on C14/C18 (that is the defect). Check 6 must
be RED on C14/C18 (that is the fix). If check 6 is not RED here, deliverable 2
is FAILED and the rung says so.
"""
import os, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BEFORE, AFTER = '8b4ca972c^', '72caf2586'
OUT = os.path.join(REPO, 'work/w-clausegen/CLAUSES_c1_control.tsv')


def cites_at(rev, addr):
    r = subprocess.run(['git', '-C', REPO, 'grep', '-l', '-F', '--', '0x' + addr,
                        rev, '--', 'crates/'], capture_output=True, text=True)
    ps = sorted(p.split(':', 1)[1] for p in r.stdout.split() if ':' in p)
    return ', '.join(ps) or '-'


def main():
    raw = subprocess.run(['git', '-C', REPO, 'show', f'{AFTER}:work/w-inlmetric/CLAUSES.tsv'],
                         capture_output=True, text=True, check=True).stdout
    lines = raw.split('\n')
    hdr = next(i for i, l in enumerate(lines) if l.startswith('id\t'))
    ncol = len(lines[hdr].split('\t'))
    # `wgloss`/`egloss` are page-rendering columns; check 6 needs only `cites`.
    lines[hdr] += '\twgloss\tegloss\tcites'
    for i in range(hdr + 1, len(lines)):
        if not lines[i].strip() or lines[i].startswith('#'):
            continue
        c = lines[i].split('\t')
        assert len(c) == ncol, c[0]
        lines[i] += f'\t-\t-\t{cites_at(BEFORE, c[2])}'
    open(OUT, 'w', encoding='utf-8').write('\n'.join(lines))
    print(f"control table: {os.path.relpath(OUT, REPO)}")
    print(f"  rows from   {AFTER} (the tree whose check 5 was GREEN)")
    print(f"  cites frozen at {BEFORE} (before w-inlbudget adopted)\n")
    return subprocess.run([sys.executable,
                           os.path.join(REPO, 'work/w-inlmetric/check_table.py'),
                           OUT, '--rev', AFTER]).returncode


if __name__ == '__main__':
    sys.exit(main())
