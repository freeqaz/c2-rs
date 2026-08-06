#!/usr/bin/env python3
"""spotcheck.py — read the REFERENCE bytes of functions the elision converted,
by hand, out of objs this script compiles itself.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

    spotcheck.py <before.jsonl> <after.jsonl> <objdir> [n]

`PREREG.md` P8: a function that moves `fnbyte-differs → fnbyte-exact` must move
because its bytes now equal c2's, and "the instrument says exact" is not
independent evidence of that — the instrument is the thing under test. So this
script does not ask the instrument anything. It

  1. takes the `(TU, symbol)` pairs that LEFT the differs set,
  2. samples `n` of them across as many distinct TUs as possible,
  3. compiles each TU with the real toolchain at the workload's own flags
     (`work/w-frame/refobj.sh` — a fresh compilation, not the capture cache),
  4. prints that symbol's whole `.text` COMDAT, word for word, with its
     relocation count, and
  5. prints the CALLEE's COMDAT beside it, because the claim is not only "the
     caller is a `blr`" but "the caller is a `blr` and the callee it no longer
     branches to is defined right here".

The port's side needs no dump: the rule emits the single word `4e800020` by
construction (`crates/c2-core/src/elide.rs`), which is what makes the reference
word the whole check. A row is PASS only when the reference COMDAT is exactly
one `4e800020` with zero relocations.
"""

import json
import os
import random
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.join(ROOT, "work", "w-inline"))
from scan_obj import read_obj  # noqa: E402

BLR = 0x4E800020


def differs(path):
    out = set()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                out.add((r["src"], k.split("|", 4)[4]))
    return out


def main(argv):
    before, after, objdir = argv[0], argv[1], argv[2]
    n = int(argv[3]) if len(argv) > 3 else 24
    gone = sorted(differs(before) - differs(after))
    # Spread across TUs: one per TU first, then fill. Seeded so the sample is
    # reproducible from the committed command line.
    rng = random.Random(20260806)
    by_tu = {}
    for src, sym in gone:
        by_tu.setdefault(src, []).append(sym)
    tus = sorted(by_tu)
    rng.shuffle(tus)
    sample = [(t, by_tu[t][0]) for t in tus[:n]]
    os.makedirs(objdir, exist_ok=True)

    print("converted (TU, symbol) pairs: %d over %d TUs; sampling %d"
          % (len(gone), len(by_tu), len(sample)))
    npass = nfail = 0
    for i, (src, sym) in enumerate(sample):
        obj = os.path.join(objdir, "s%02d.obj" % i)
        r = subprocess.run([os.path.join(ROOT, "work", "w-frame", "refobj.sh"), src, obj],
                           capture_output=True, text=True)
        if r.returncode != 0 or not os.path.exists(obj):
            print("  COMPILE-FAIL %s" % src)
            nfail += 1
            continue
        fns = read_obj(obj)
        f = fns.get(sym)
        if f is None:
            print("  NO-COMDAT %s in %s" % (sym[:50], src))
            nfail += 1
            continue
        words = " ".join("%08x" % w for w in f.words)
        ok = f.words == [BLR] and not f.rel24
        npass += ok
        nfail += not ok
        print("  [%s] %s" % ("PASS" if ok else "FAIL", src))
        print("        %s" % sym)
        print("        ref .text = %s   rel24 = %s" % (words, f.rel24 or "none"))
    print("\nhand-checked %d: PASS %d, FAIL %d" % (len(sample), npass, nfail))
    return 0 if nfail == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
