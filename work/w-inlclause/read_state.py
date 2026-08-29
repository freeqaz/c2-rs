#!/usr/bin/env python3
"""read_state.py -- grade the `read` / `readcite` / `blocker` columns. Lane `w-inlclause`.

`work/w-inlmetric/check_table.py` grades the `state` column five ways. It cannot
grade these three, because they are about the PROJECT's knowledge rather than
the port's code, and a `readcite` that names a file which does not say what the
row claims is exactly the failure `#3470` and `#3641` are about in their own
domains: a citation nobody re-derived.

FOUR checks, and each one is a rule `work/w-inlclause/PREREG.md` fixed BEFORE
the classification, not after:

 1. VALUE      `read` is R1 | R2 | R3; `blocker` is drawn from the vocabulary
               the prereg fixed. A free-text `blocker` is an allowlist entry
               pretending to be a diagnosis.

 2. CITE       an R1/R2 row's `readcite` is `path#anchor`, the path exists, and
               the anchor STRING IS PRESENT IN IT. This is the whole check:
               `read` is a claim about a document, so it is checkable against
               the document, and an unchecked one is worth nothing.

 3. GRAMMAR    the cross-rules between `read`, `blocker` and `state`:
                 R3            => readcite `-` and blocker `unread`
                 R1 + absent   => blocker != none  (a derivable clause with no
                                  blocker is one somebody should have adopted,
                                  and leaving it is the thing this lane was
                                  told not to do)
                 R-derived     => R1  (the port cannot carry a counterpart
                                  derived from a read that does not exist)
                 unexercisable => blocker `n-a`

 4. EVIDENCE   every R3 row has a section in UNREAD_EVIDENCE.md naming it.
               `R3` is a UNIVERSAL NEGATIVE over a corpus -- the cheapest cell
               in the table to write and the most expensive to falsify -- so
               the prereg made it cost the same as R1. Today the population is
               EMPTY (no row is R3), so this check grades zero rows and says
               so, which is `#3470`: a clean report over nothing is not clean.

Exit 0 = GREEN. Non-zero = RED. Read the verdict line, never the exit code.

Usage: read_state.py [CLAUSES.tsv] [--plant ID=COL=VALUE ...]

  --plant  overwrite one cell before grading, so the RED path is WATCHED rather
           than assumed (`#3336`). Repeatable.
             --plant C7=read=R3            (grammar: R3 with a live readcite)
             --plant C7=readcite=x.md#0x1  (cite: the path does not exist)
             --plant C14=read=R2           (grammar: R-derived must be R1)
"""
import csv, os, sys
from collections import Counter

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TABLE = 'work/w-inlmetric/CLAUSES.tsv'
EVIDENCE = 'work/w-inlclause/UNREAD_EVIDENCE.md'

READS = {'R1', 'R2', 'R3'}
# Fixed by PREREG SS2. A blocker outside this set is a defect, not a new case:
# widening it is a source edit somebody reviews.
BLOCKERS = {
    'none',             # R1, adoptable
    'emit-change',      # R1, derivable and out of a byte-neutral lane's scope
    'no-instr-count',   # R2, the port has no pre-codegen instruction count
    'no-instr-stream',  # R2, the port has no c2 instruction/opcode stream
    'unit-gap',         # R2, c2's quantity and the port's are different units
    'writer-unread',    # R2, the datum is read; nothing writes to it is known
    'unread',           # R3
    'n-a',              # unexercisable
}


def main(argv):
    plants, args, i = [], [], 0
    while i < len(argv):
        if argv[i] == '--plant':
            plants.append(argv[i + 1].split('=', 2))
            i += 2
            continue
        args.append(argv[i])
        i += 1

    path = args[0] if args else os.path.join(REPO, TABLE)
    rows = list(csv.DictReader(
        [l for l in open(path) if not l.startswith('#')], delimiter='\t'))
    by_id = {r['id']: r for r in rows}
    for rid, col, val in plants:
        if rid in by_id:
            by_id[rid][col] = val
            by_id[rid]['id'] = rid + '(PLANTED)'

    fails, cited, r3 = [], 0, []
    for r in rows:
        rid, rd = r['id'], (r.get('read') or '').strip()
        cite = (r.get('readcite') or '').strip()
        blk = (r.get('blocker') or '').strip()

        # 1. VALUE
        if rd not in READS:
            fails.append(f"{rid}: VALUE read={rd!r} is not one of {sorted(READS)}")
            continue
        if blk not in BLOCKERS:
            fails.append(f"{rid}: VALUE blocker={blk!r} is outside the fixed vocabulary")

        # 2. CITE
        if rd in ('R1', 'R2'):
            if '#' not in cite:
                fails.append(f"{rid}: CITE {rd} must cite path#anchor, got {cite!r}")
            else:
                p, _, anchor = cite.partition('#')
                full = os.path.join(REPO, p)
                if not os.path.exists(full):
                    fails.append(f"{rid}: CITE path {p!r} does not exist")
                elif anchor not in open(full, encoding='utf-8', errors='replace').read():
                    fails.append(
                        f"{rid}: CITE anchor {anchor!r} IS NOT IN {p} — the row claims a "
                        f"read that the cited document does not contain")
                else:
                    cited += 1

        # 3. GRAMMAR
        if rd == 'R3':
            r3.append(rid)
            if cite != '-':
                fails.append(f"{rid}: GRAMMAR R3 must cite '-', got {cite!r}")
            if blk != 'unread':
                fails.append(f"{rid}: GRAMMAR R3 must have blocker 'unread', got {blk!r}")
        if rd == 'R1' and r['state'] == 'absent' and blk == 'none':
            fails.append(
                f"{rid}: GRAMMAR R1 + absent + blocker 'none' — derivable, unblocked and "
                f"NOT ADOPTED. Either adopt it or name what stops you.")
        if r['state'] == 'R-derived' and rd != 'R1':
            fails.append(
                f"{rid}: GRAMMAR state R-derived with read={rd} — the port cannot carry a "
                f"counterpart derived from a read that does not exist")
        if r['state'] == 'unexercisable' and blk != 'n-a':
            fails.append(f"{rid}: GRAMMAR unexercisable must have blocker 'n-a', got {blk!r}")

    # 4. EVIDENCE
    ev = os.path.join(REPO, EVIDENCE)
    ev_text = open(ev, encoding='utf-8').read() if os.path.exists(ev) else None
    for rid in r3:
        if ev_text is None:
            fails.append(f"{rid}: EVIDENCE {EVIDENCE} does not exist and this row is R3")
        elif f"## {rid} " not in ev_text and f"## {rid}\n" not in ev_text:
            fails.append(f"{rid}: EVIDENCE {EVIDENCE} has no `## {rid}` section")

    n = len(rows)
    c = Counter((r.get('read') or '?').strip() for r in rows)
    b = Counter((r.get('blocker') or '?').strip() for r in rows)
    print(f"table    : {os.path.relpath(path, REPO)}")
    print(f"rows     : {n}")
    print("  read     :", dict(c))
    print("  blocker  :", dict(b))
    print(f"  CITE     : {cited} of {n} rows had a readcite resolved to a present anchor")
    if r3:
        print(f"  EVIDENCE : {len(r3)} R3 row(s) graded against {EVIDENCE}: {r3}")
    else:
        print(f"  EVIDENCE : SKIP — 0 of {n} rows are R3, so this check graded NOTHING. "
              f"That is the population being empty, not the check passing (#3470).")
    if plants:
        print(f"planted  : {plants}")
    for f in fails:
        print("  FAIL " + f)
    print(f"\nREAD-STATE: {'RED' if fails else 'GREEN'}  "
          f"({len(fails)} failure(s) over {n} rows)")
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
