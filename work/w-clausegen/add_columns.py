#!/usr/bin/env python3
"""One-shot: append `wgloss`, `egloss`, `cites` to CLAUSES.tsv.

ADDITIVE ONLY. Every existing cell is copied through byte-for-byte; the script
asserts that before writing. Run once by `w-clausegen`; kept as the record of
how the three columns were populated.

  wgloss  the parenthetical `P_INLINE.md` §6.1 prints after `witness`
  egloss  the parenthetical / dash-clause it prints after `exercised`
          -- both transcribed from the page as it stood at c5bfe89d9, so
          generating the page from the table drops NO published information
  cites   the frozen set of files under `crates/` citing `0x<addr>`, MEASURED,
          not transcribed (check 6, `check_table.py`)
"""
import csv, os, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TSV = os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')

# Transcribed from P_INLINE.md §6.1 at c5bfe89d9, by hand, once.
WGLOSS = {'C13': '(`0x40`)', 'C24': '(`W-GLATTRS-1`)'}
EGLOSS = {
    'C2': '(F7)', 'C3': '(F7)', 'C6': '(F8, 6 cells)',
    'C9': '— `/O1` pins the bit', 'C10': '(F4, 2 cells)',
    'C14': '— no cell nests 16 deep',
    'C15': '— `#pragma inline_depth` in 0/100 TUs',
    'C17': '(F7)', 'C20': '(#1020, 150 witnesses)',
    'C24': '(99 escaped records)',
}


def cites(addr):
    r = subprocess.run(['git', '-C', REPO, 'grep', '-l', '--untracked',
                        '--exclude-standard', '-F', '--', '0x' + addr,
                        '--', 'crates/'], capture_output=True, text=True)
    return ', '.join(sorted(p for p in r.stdout.split() if p)) or '-'


def main():
    raw = open(TSV, encoding='utf-8').read()
    lines = raw.split('\n')
    hdr_i = next(i for i, l in enumerate(lines) if l.startswith('id\t'))
    header = lines[hdr_i].split('\t')
    if header[-3:] == ['wgloss', 'egloss', 'cites']:
        print('already applied')
        return 0
    lines[hdr_i] = '\t'.join(header + ['wgloss', 'egloss', 'cites'])
    n = 0
    for i in range(hdr_i + 1, len(lines)):
        if not lines[i].strip() or lines[i].startswith('#'):
            continue
        cells = lines[i].split('\t')
        assert len(cells) == len(header), f'row {cells[0]} has {len(cells)} cells'
        rid = cells[0]
        lines[i] = '\t'.join(cells + [WGLOSS.get(rid, '-'), EGLOSS.get(rid, '-'),
                                      cites(cells[2])])
        n += 1
    out = '\n'.join(lines)
    open(TSV, 'w', encoding='utf-8').write(out)

    # The additive guarantee, checked rather than asserted in prose.
    old = list(csv.DictReader([l for l in raw.split('\n') if not l.startswith('#')],
                              delimiter='\t'))
    new = list(csv.DictReader([l for l in out.split('\n') if not l.startswith('#')],
                              delimiter='\t'))
    assert len(old) == len(new) == n, (len(old), len(new), n)
    for a, b in zip(old, new):
        for k in a:
            assert a[k] == b[k], f'{a["id"]}.{k} CHANGED: {a[k]!r} -> {b[k]!r}'
    print(f'appended 3 columns to {n} rows; all {len(header)} existing cells identical')
    return 0


if __name__ == '__main__':
    sys.exit(main())
