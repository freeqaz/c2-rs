#!/usr/bin/env python3
"""scan2w.py — w-emitp2's `scan2.py`, run UNMODIFIED, with the crate-acceptance
transcription swapped for the widened one.

The whole file is four lines of import order: patch `strictin._crate_verdict`
(via `crateacc`) BEFORE `scan2` binds it, then hand over.  `scan2.py` itself is
not edited and not copied — a lane that copied it would no longer be running
w-emitp2's known-answer control.

    usage: scan2w.py <cacheidx.tsv> <dtruth-dir> <w-emit-truth> <out.jsonl> [jobs]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT", os.path.abspath(os.path.join(HERE, "..", "..")))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(MAIN, "work", "w-emitp2"))
import crateacc  # noqa: E402,F401  — the patch; must precede `scan2`
import scan2     # noqa: E402

assert scan2.strictin._crate_verdict is crateacc.crate_verdict_widened, \
    "the widened acceptance did not reach scan2 — the patch order is wrong"

if __name__ == "__main__":
    scan2.main()
