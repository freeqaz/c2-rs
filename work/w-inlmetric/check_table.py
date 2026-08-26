#!/usr/bin/env python3
"""check_table.py -- grade the conformance table MECHANICALLY.

PREREG SS6. Three checks, each of which has caught a real defect in this repo:

 1. ADDRESS  every `addr` must lie inside the function `owner` names, per
    FUNCS.tsv's entry+size. `P_INLINE.md` SS2.1's CORRECTION block is exactly
    this check done by hand, once, after four addresses had been published in
    the wrong function. Here it runs on every row, every time.

 2. WITNESS  a row whose state is `R-derived` or `fitted` must cite
    `path:token` and that token must be PRESENT at that path.

 3. ABSENCE  a row whose state is `absent` or `unexercisable` must cite
    `none:<token>` and that token must be ABSENT from `crates/`. An `absent`
    verdict that is merely unchecked is the failure mode this exists for.

Exit 0 = GREEN. Non-zero = RED. Read the verdict line, never the exit code.

Usage: check_table.py [CLAUSES.tsv]
"""
import bisect, csv, os, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def funcs():
    p = os.path.join(REPO, 'docs/whitebox/ref/FUNCS.tsv')
    out = []
    for x in csv.DictReader([l for l in open(p) if not l.startswith('#')], delimiter='\t'):
        try:
            out.append((int(x['addr'], 16), int(x['size'])))
        except (ValueError, TypeError):
            pass
    out.sort()
    return out


def owner_of(fns, a):
    starts = [f[0] for f in fns]
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s, n = fns[i]
    return s if a < s + n else None


def token_in_file(path, tok):
    p = os.path.join(REPO, path)
    if not os.path.exists(p):
        return False
    return tok in open(p, encoding='utf-8', errors='replace').read()


def token_in_crates(tok):
    r = subprocess.run(['git', '-C', REPO, 'grep', '-l', '-F', '--', tok, '--', 'crates/'],
                       capture_output=True, text=True)
    return bool(r.stdout.strip())


def main(argv):
    path = argv[0] if argv else os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')
    fns = funcs()
    rows = list(csv.DictReader([l for l in open(path) if not l.startswith('#')], delimiter='\t'))
    fails = []
    for r in rows:
        rid = r['id']
        # 1. ADDRESS
        try:
            a = int(r['addr'], 16)
            claimed = int(r['owner'], 16)
        except ValueError:
            fails.append(f"{rid}: addr/owner not hex")
            continue
        real = owner_of(fns, a)
        if real is None:
            fails.append(f"{rid}: ADDRESS 0x{a:08x} is inside NO FUNCS.tsv function (orphan)")
        elif real != claimed:
            fails.append(f"{rid}: ADDRESS 0x{a:08x} is in FUN_{real:08x}, "
                         f"table claims FUN_{claimed:08x}")
        # 2/3. WITNESS
        st, w = r['state'], r['witness']
        if st in ('R-derived', 'fitted'):
            if w.startswith('none:') or ':' not in w:
                fails.append(f"{rid}: state {st} must cite path:token, got {w!r}")
            else:
                p, tok = w.rsplit(':', 1)
                if not token_in_file(p, tok):
                    fails.append(f"{rid}: WITNESS {tok!r} NOT FOUND in {p}")
        elif st in ('absent', 'unexercisable'):
            if not w.startswith('none:'):
                fails.append(f"{rid}: state {st} must cite none:<token>, got {w!r}")
            else:
                tok = w[len('none:'):]
                if token_in_crates(tok):
                    fails.append(f"{rid}: state {st} but token {tok!r} IS PRESENT in crates/")
        else:
            fails.append(f"{rid}: unknown state {st!r}")

    from collections import Counter
    c = Counter(r['state'] for r in rows)
    e = Counter(r['exercised'] for r in rows)
    print(f"rows: {len(rows)}")
    print("  state    :", dict(c))
    print("  exercised:", dict(e))
    for f in fails:
        print("  FAIL " + f)
    print(f"\nCONFORMANCE-CHECK: {'RED' if fails else 'GREEN'}  ({len(fails)} failure(s) over {len(rows)} rows)")
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
