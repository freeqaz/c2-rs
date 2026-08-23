#!/usr/bin/env python3
"""R6 confirmation probe: is the prologue pseudo-op's expansion OBSERVABLE?

Read R6 (`docs/whitebox/ref/P_EXPAND.md`) reads the final-expansion switch and
claims the prologue pseudo-op `0x2f4`/`0x2f0` is ONE tuple that the switch
rewrites in situ into MANY words.  `[R]` means the instructions were read
correctly -- not that this is what c2 does.  This is the probe that can say.

The oracle needs no tap and no recompilation: **c2 records the prologue's word
count in the object it emits.**  Every `.pdata` record's unwind word carries
`prolog_words` in its low 8 bits and the function's total length in words in
bits 8..29 (`WB_EH_FINDINGS.md` §5 row W-EH-1, obj-confirmed; emitter side
`crates/c2-core/src/coff/pdata.rs:71`).  So for every framed function in the
corpus we get, for free, the exact number of words the expansion produced.

WHAT THIS PROBE CAN FALSIFY, stated before it is run (prereg P3.3/P5.1/P5.2):

  1. that the count is NOT a constant          -> a single-valued histogram kills it
  2. that the common case is <= 8 words        -> a fat tail kills it
  3. that it is NOT linear in saved registers  -> if c2 emitted one store per
     saved register the count would climb with the save count; the helper-call
     shape (`bl __savegprlr_N`) keeps it flat.  These two give NUMERICALLY
     DIFFERENT answers, which is what makes the cell capable of failing.
  4. that the prologue's last word is a frame-establish or a helper call
     -> a `prolog_words` boundary landing mid-body kills it.

WHAT IT CANNOT DO -- read this before quoting the number (prereg §7):
  * An obj is POST-EVERYTHING.  These words have been through selection,
    expansion, the peephole and the encoder.  If the peephole deleted a word
    the expansion emitted, this probe is right about the obj and wrong about
    the expansion.  It cannot separate the two passes.
  * LEAF functions emit no `.pdata` record at all, so they are absent from the
    denominator rather than counted as zero.
  * It sees only shapes the corpus contains.

Usage:
    python3 docs/whitebox/scripts/probe_prolog_words.py [--cache DIR] [--limit N]
"""

import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))

from gt_dump import Obj, u32  # noqa: E402


def be32(b, o):
    return int.from_bytes(b[o:o + 4], "big")


# PPC primary opcodes we need to recognise a prologue word by shape alone.
def classify(w):
    op = w >> 26
    if op == 31 and ((w >> 1) & 0x3FF) == 339:
        return "mfspr"                      # mflr r12 is mfspr r12,LR
    if op == 31 and ((w >> 1) & 0x3FF) == 467:
        return "mtspr"
    if op == 18:
        return "b" + ("l" if w & 1 else "")
    if op == 16:
        return "bc"
    if op == 37:
        return "stwu"
    if op == 31 and ((w >> 1) & 0x3FF) == 183:
        return "stwux"
    if op == 36:
        return "stw"
    if op == 54:
        return "stfd"
    if op == 14:
        return "addi"
    if op == 15:
        return "addis"
    if op == 31 and ((w >> 1) & 0x3FF) == 444:
        return "or"                         # mr
    return "op%d" % op


def iter_objs(root, limit):
    n = 0
    for dirpath, _dirnames, filenames in os.walk(root):
        if "out.obj" in filenames:
            yield os.path.join(dirpath, "out.obj")
            n += 1
            if limit and n >= limit:
                return


def main(argv):
    cache = os.path.expanduser("~/.cache/c2rs/capture")
    limit = 4000
    i = 0
    while i < len(argv):
        if argv[i] == "--cache":
            cache = argv[i + 1]; i += 2
        elif argv[i] == "--limit":
            limit = int(argv[i + 1]); i += 2
        else:
            i += 1
    if not os.path.isdir(cache):
        print("SKIP: no capture cache at %s" % cache)
        return 0

    hist = collections.Counter()
    first_word = collections.Counter()
    last_word = collections.Counter()
    shapes = collections.Counter()
    nrec = nobj = nbad = 0
    nshape = [0]
    helper_calls = collections.Counter()

    for path in iter_objs(cache, limit):
        try:
            o = Obj(open(path, "rb").read())
        except Exception:
            nbad += 1
            continue
        nobj += 1
        texts = {s["idx"]: o.raw(s) for s in o.sections
                 if s["name"].startswith(".text")}
        for s in o.sections:
            if not s["name"].startswith(".pdata"):
                continue
            try:
                raw = o.raw(s)
            except Exception:
                continue
            for off in range(0, len(raw) - 7, 8):
                w = be32(raw, off + 4)
                prolog = w & 0xFF
                nwords = (w >> 8) & 0x3FFFFF
                if nwords == 0 or prolog > nwords or prolog == 0:
                    continue
                nrec += 1
                hist[prolog] += 1
                # Shape decoding is only SOUND when the record's function is
                # unambiguous.  A multi-COMDAT obj has many .text sections and
                # this probe does not resolve the BeginAddress relocation, so
                # picking "the first .text long enough" would misattribute the
                # words.  Restrict shapes to objs with exactly ONE .text whose
                # length matches the record; the HISTOGRAM above is unaffected
                # because prolog_words is read straight out of .pdata.
                if len(texts) == 1:
                    tb = next(iter(texts.values()))
                    if len(tb) == 4 * nwords:
                        ws = [be32(tb, 4 * k) for k in range(prolog)]
                        cls = [classify(x) for x in ws]
                        first_word[cls[0]] += 1
                        last_word[cls[-1]] += 1
                        shapes["|".join(cls)] += 1
                        helper_calls[cls.count("bl")] += 1
                        nshape[0] += 1

    print("# R6 confirmation probe -- prologue expansion, corpus side")
    print("# cache=%s  objs read=%d  unreadable=%d  .pdata records=%d"
          % (cache, nobj, nbad, nrec))
    if not nrec:
        print("FAIL: no .pdata records -- the probe graded nothing")
        return 1
    tot = sum(hist.values())
    print("\n## prolog_words histogram (denominator %d framed functions)" % tot)
    for k in sorted(hist):
        print("  %3d words : %6d  (%5.2f%%)" % (k, hist[k], 100.0 * hist[k] / tot))
    le8 = sum(v for k, v in hist.items() if k <= 8)
    print("  <= 8 words: %d / %d = %.2f%%" % (le8, tot, 100.0 * le8 / tot))
    print("  min=%d max=%d distinct=%d" % (min(hist), max(hist), len(hist)))

    print("\n## SHAPE SUB-POPULATION: %d of %d records (%.1f%%) are in a"
          " single-.text obj whose length matches the record, so their words"
          " can be attributed without resolving the BeginAddress relocation."
          % (nshape[0], tot, 100.0 * nshape[0] / tot))
    print("\n## first prologue word, by shape")
    for k, v in first_word.most_common(8):
        print("  %-8s %6d" % (k, v))
    print("\n## last prologue word, by shape")
    for k, v in last_word.most_common(8):
        print("  %-8s %6d" % (k, v))
    print("\n## number of `bl` (register-save helper calls) inside the prologue")
    for k in sorted(helper_calls):
        print("  %d bl : %6d" % (k, helper_calls[k]))
    print("\n## the 12 most common whole-prologue shapes")
    for k, v in shapes.most_common(12):
        print("  %6d  %s" % (v, k))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
