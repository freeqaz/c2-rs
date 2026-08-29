#!/usr/bin/env python3
"""read_scan.py -- WHICH CLAUSES HAS THIS PROJECT ACTUALLY READ? Lane `w-inlclause`.

`work/w-inlmetric/CLAUSES.tsv`'s `state` column says whether the **port** has a
counterpart. It says nothing about whether **this project** has read the clause
well enough to build one, and on 2026-08-29 all fifteen `absent` rows were
carrying one word for two different situations:

  * *"nobody has read this"*                    -- a read is the next step
  * *"it is read, and the port cannot use it"*  -- a read buys nothing

`docs/ADOPTION_BRIEF_2026-08-29.md` §L2 asks for the difference. This is the
evidence half: for each clause, the addresses that PIN it (not the row's
citation address, which for five rows is a function entry and for three more is
a block head -- `work/w-clausefix/REPAIRS.md` §"14 rows, all verified"), grepped
over a FROZEN corpus of prose reads.

WHAT COUNTS AS A READ, from `work/w-inlclause/PREREG.md` §2, fixed before any
row was classified:

  an address-cited passage that goes BEYOND RESTATING the clause -- it names at
  least one address other than the row's own `addr`, or enumerates the
  readers/writers of the datum the clause tests.

WHAT IS DELIBERATELY NOT CORPUS: `*.asm` dumps and `*.out` transcripts. A raw
disassembly listing is the INPUT to a read, not a read: `FUN_10b600e6.asm`
contains every address in the site collector and tells a later reader nothing
about which of them matter. Counting it would make every clause in a dumped
function "read" for free, which is the exact shape of the false green this
lane exists to prevent. Only `.md` prose is scanned.

Usage:  read_scan.py [--plant CID=ADDR]     # plant an address to watch RED
"""
import os, re, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# The frozen corpus, `PREREG.md` §3. Directories are scanned for `*.md` only.
CORPUS = [
    'docs/whitebox/ref/P_INLINE.md',
    'docs/whitebox/WB_INLINE_FINDINGS.md',
    'docs/whitebox/WB_INLSWITCH_FINDINGS.md',
    'docs/whitebox/WB_LOWERBAND_FINDINGS.md',
    'docs/whitebox/WB_CANDID_FINDINGS.md',
    'docs/INLINE_PREDICATE.md',
    'work/w-inlmetric', 'work/w-inlfit', 'work/w-inlbudget',
    'work/w-inlswitch', 'work/w-clausefix', 'work/w-lowerband',
]

# This lane's own read, which is corpus only for the `--tip` reading.
OWN = 'work/w-inlclause/IMAGE_READ.md'

# The CLAUSE-PINNING addresses, per row. Every one is verified against the
# independent objdump listing in `work/w-inlclause/IMAGE_READ.md`; none is
# taken from `CLAUSES.tsv`, because for eight of these fifteen rows the cited
# address is an entry or a block head that pins no clause.
PINS = {
    'C1':  ['10b6267b', '10b626c1'],
    'C2':  ['10b626f7', '10b62703'],
    'C4':  ['10b6276e'],
    'C5':  ['10b6020b'],
    'C6':  ['10b603ef', '10b60405', '10b60347'],
    'C7':  ['10b5e4d1', '10b5e4d7', '10b5e4e8'],
    'C9':  ['10b5fc7e', '10b5fc84'],
    'C10': ['10b60a28', '10b60a2d', '10b60a3c'],
    'C11': ['10b5c06e', '10b5c080', '10b5c08f', '10b5c093'],
    'C12': ['10b5c078', '10b5c087'],
    'C14': ['10b60a1c', '10b60a1f'],
    'C15': ['10b60a2f', '10b60a37'],
    'C16': ['10b60a63', '10b60a6d'],
    'C17': ['10b60a73', '10b60a78'],
    'C18': ['10b625b6', '10b625b9'],
}


def md_files(with_own):
    out = []
    for c in CORPUS + ([OWN] if with_own else []):
        p = os.path.join(REPO, c)
        if os.path.isfile(p):
            out.append(c)
        elif os.path.isdir(p):
            for f in sorted(os.listdir(p)):
                if f.endswith('.md'):
                    out.append(os.path.join(c, f))
    return [f for f in out if os.path.exists(os.path.join(REPO, f))]


def hits(files, addr):
    """Files in which `addr` appears, as a list."""
    r = subprocess.run(['grep', '-l', '-F', '-i', '--', addr, *files],
                       cwd=REPO, capture_output=True, text=True)
    return [x for x in r.stdout.strip().split('\n') if x]


def main(argv):
    plants = {}
    i = 0
    while i < len(argv):
        if argv[i] == '--plant':
            cid, _, a = argv[i + 1].partition('=')
            plants.setdefault(cid, []).append(a)
            i += 2
            continue
        i += 1
    pins = {k: list(v) for k, v in PINS.items()}
    for cid, extra in plants.items():
        pins[cid + '(PLANTED)'] = extra
        pins.pop(cid, None)

    base = md_files(False)
    tip = md_files(True)
    print(f"corpus   : {len(base)} prose files (.md only; .asm dumps and .out "
          f"transcripts are NOT corpus -- see the module doc)")
    print(f"  +own   : {len(tip)} with this lane's own read included")
    if plants:
        print(f"planted  : {plants}")
    print()
    print(f"{'row':<16}{'pins':>5}  {'as-dispatched':<13}{'at-tip':<8} where")
    n_base = n_tip = 0
    for cid in sorted(pins, key=lambda s: int(re.sub(r'\D', '', s))):
        addrs = pins[cid]
        hb, ht = set(), set()
        for a in addrs:
            hb.update(hits(base, a))
            ht.update(hits(tip, a))
        n_base += 1 if hb else 0
        n_tip += 1 if ht else 0
        where = ', '.join(sorted(hb)) if hb else '(none)'
        print(f"{cid:<16}{len(addrs):>5}  {'READ' if hb else 'unread':<13}"
              f"{'READ' if ht else 'unread':<8} {where}")
    n = len(pins)
    print()
    print(f"PIN-SCAN: {n_base} of {n} rows have at least one clause-pinning address "
          f"cited in the frozen corpus as dispatched")
    print(f"PIN-SCAN: {n_tip} of {n} rows once this lane's own read is included")
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
