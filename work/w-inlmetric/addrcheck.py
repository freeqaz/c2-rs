#!/usr/bin/env python3
"""addrcheck.py -- is address A inside the function the page says it is?

`P_INLINE.md` §2.1 carries a CORRECTION block: four addresses quoted as being
inside `FUN_10b5fb5f` are past its end and land in `FUN_10b5fcd8`. That check
was done by hand, once. This does it mechanically for every address a clause
row cites, against `docs/whitebox/ref/FUNCS.tsv`'s entry+size columns --
`README.md` §6.2's rule as a program.

Usage: addrcheck.py <addr> [<addr> ...]     (hex, with or without 0x)
       addrcheck.py --pairs <addr>:<claimed-owner> ...
"""
import csv, sys, bisect

def load():
    lines = [l for l in open('docs/whitebox/ref/FUNCS.tsv') if not l.startswith('#')]
    fns = []
    for x in csv.DictReader(lines, delimiter='\t'):
        try:
            fns.append((int(x['addr'], 16), int(x['size']), x['tu']))
        except (ValueError, TypeError):
            pass
    fns.sort()
    return fns

def owner(fns, a):
    starts = [f[0] for f in fns]
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s, n, tu = fns[i]
    return (s, n, tu) if a < s + n else None

def main(argv):
    fns = load()
    pairs = "--pairs" in argv
    args = [x for x in argv if x != "--pairs"]
    bad = 0
    for tok in args:
        claim = None
        if pairs and ':' in tok:
            tok, claim = tok.split(':')
        a = int(tok, 16)
        o = owner(fns, a)
        if o is None:
            print(f"0x{a:08x}  ORPHAN -- inside no FUNCS.tsv function")
            bad += 1
            continue
        s, n, tu = o
        tag = ""
        if claim is not None:
            c = int(claim, 16)
            if c != s:
                tag = f"  <-- WRONG: page claims 0x{c:08x}"
                bad += 1
            else:
                tag = "  ok"
        print(f"0x{a:08x}  in FUN_{s:08x} (size {n}, ends 0x{s+n:08x}, tu {tu}){tag}")
    print(f"\n{'RED' if bad else 'GREEN'}: {bad} address claim(s) failed")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
