#!/usr/bin/env python3
"""addr_align.py -- FOLDED. This is a shim; the check lives in check_table.py.

`w-inlfit` (2026-08-27, board #3721) wrote this program to answer the half of
the address question `work/w-inlmetric/check_table.py` could not: does every
`CLAUSES.tsv` address START an instruction? It kept it as a SEPARATE program for
one stated reason -- `check_table.py` was another lane's frozen instrument and
`w-inlfit`'s prereg forbade editing it.

`w-clausefix` (2026-08-28) owns both files under `work/w-clausefix/PREREG.md`,
so that reason is gone, and the check was folded in as `check_table.py`'s
**check 2, ALIGN**, alongside a new **check 3, DECODE** for the class alignment
cannot reach (aligned address, wrong instruction -- C10 and C15).

Why folded rather than left beside it: #3679. Two programs is two chances to run
only one, and the repo has already paid for a `scripts/` entry no funnel
invokes. ADDRESS and ALIGN are two halves of a single question -- *is this the
address of the thing the clause names* -- and they now go red from one place,
under one `cargo test` target (`crates/c2-harness/tests/clause_table.rs`).

This path survives so that `docs/rungs/2026-08-27-w-inlfit.md` SS4's citation
keeps resolving. It delegates; it holds no logic of its own. `--plant` is
forwarded, and the argument form is now `--plant ID=ADDR` rather than
`--plant 0xADDR`.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FOLDED = os.path.join(os.path.dirname(HERE), 'w-inlmetric', 'check_table.py')

sys.path.insert(0, os.path.dirname(FOLDED))
import check_table  # noqa: E402

if __name__ == '__main__':
    print("addr_align.py: FOLDED into work/w-inlmetric/check_table.py "
          "(check 2, ALIGN) by w-clausefix, 2026-08-28 -- delegating.\n")
    sys.exit(check_table.main(sys.argv[1:]))
