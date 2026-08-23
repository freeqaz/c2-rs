#!/usr/bin/env python3
"""w-tailread confirmation probe: does c2 ever EMIT a redundant self-move?

`ref/P_OPATTR.md` §5 reads the peephole's arm 6 (`fmr`, `0x10c1838b`) and its
class-1 siblings (arms 14/15/16 = `mr`, `mr.`, `vmr`) and finds a redundant-move
eliminator: when the source and destination operands name the SAME register the
handler tail-calls `0x10c16cde` and the instruction is UNLINKED by the delete
primitive `0x10bd5516`.  Separately, the final-expansion switch's own join at
`0x10c0e4a4` calls that same primitive.

`[R]` means the instructions were read correctly -- never that this is what c2
does (`ref/README.md` §2; the `.bss`-bump failure mode).  This probe is what can
say, and it needs no tap: a deleted instruction is one that never reaches the
object, so the claim is directly falsifiable against emitted `.text`.

  claim   ->  c2 emits NO self-move: no `mr rX,rX`, no `fmr fX,fX`
  falsify ->  a single self-move in any captured .text kills it

WHAT MAKES THIS CELL CAPABLE OF FAILING, and why the denominator is printed:

  "0 self-moves found" is worthless on its own -- a scan that decodes nothing
  reports 0 too.  This probe therefore also counts every NON-self `mr` and
  `fmr` it sees.  If those are 0 the scan is VACUOUS and the probe says so
  instead of reporting a pass.  (Board #3341: "0 SKIP lines" is not evidence.)

WHAT IT CANNOT DO:
  * An obj is POST-EVERYTHING.  It cannot attribute the absence to the
    peephole rather than to selection never creating a self-move in the first
    place.  It confirms the OUTCOME, not the mechanism or which pass owns it.
  * It sees only the shapes this corpus contains.

Usage:
    python3 docs/whitebox/scripts/probe_selfmove.py [--cache DIR] [--limit N]
"""

import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))

from gt_dump import Obj  # noqa: E402


def be32(b, o):
    return int.from_bytes(b[o:o + 4], "big")


def decode(w):
    """-> (kind, is_self) for the move forms this probe is about, else None.

    `mr rA,rS`  is `or rA,rS,rS`   : primary 31, xo 444, RS == RB.
                                     self-move iff RA == RS == RB.
    `fmr fT,fB` is primary 63, xo 72.  self-move iff FRT == FRB.
    """
    op = w >> 26
    rc = w & 1
    if op == 31 and ((w >> 1) & 0x3FF) == 444:
        rs, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
        if rs != rb:
            return None                      # a real `or`, not a `mr`
        return ("mr." if rc else "mr"), (ra == rs)
    if op == 63 and ((w >> 1) & 0x3FF) == 72:
        frt, frb = (w >> 21) & 31, (w >> 11) & 31
        return ("fmr." if rc else "fmr"), (frt == frb)
    return None


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
    limit = 6000
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

    seen = collections.Counter()      # kind -> non-self occurrences
    self_ = collections.Counter()     # kind -> SELF occurrences
    self_reg = collections.Counter()  # which register a self-move names
    self_nb = collections.Counter()   # what sits either side of one
    self_objs = set()
    examples = []
    nobj = nbad = nword = ntext = 0

    for path in iter_objs(cache, limit):
        try:
            o = Obj(open(path, "rb").read())
        except Exception:
            nbad += 1
            continue
        nobj += 1
        for s in o.sections:
            if not s["name"].startswith(".text"):
                continue
            try:
                raw = o.raw(s)
            except Exception:
                continue
            ntext += 1
            for off in range(0, len(raw) - 3, 4):
                w = be32(raw, off)
                nword += 1
                d = decode(w)
                if d is None:
                    continue
                kind, is_self = d
                if is_self:
                    self_[kind] += 1
                    self_objs.add(path)
                    self_reg[(w >> 21) & 31] += 1
                    nb = []
                    for k in (-1, 1):
                        o2 = off + 4 * k
                        if 0 <= o2 < len(raw) - 3:
                            w2 = be32(raw, o2)
                            d2 = decode(w2)
                            nb.append("self" if (d2 and d2[1])
                                      else "op%d" % (w2 >> 26))
                    self_nb[tuple(nb)] += 1
                    if len(examples) < 8:
                        examples.append((path, off, kind, w))
                else:
                    seen[kind] += 1

    print("corpus: %d objs (%d unreadable), %d .text sections, %d words"
          % (nobj, nbad, ntext, nword))
    print()
    print("kind   non-self   SELF-MOVE")
    kinds = sorted(set(seen) | set(self_))
    for k in kinds:
        print("%-6s %8d   %9d" % (k, seen[k], self_[k]))
    total_nonself = sum(seen.values())
    total_self = sum(self_.values())
    print()
    print("non-self move forms seen: %d   self-moves: %d"
          % (total_nonself, total_self))
    print()

    # ORDER MATTERS, and it was wrong here until a fence test caught it.  A
    # corpus whose ONLY move form is a self-move has total_nonself == 0, so a
    # vacuity check placed first reports VACUOUS for the one input that most
    # clearly refutes the claim.  A refutation is a refutation whatever the
    # denominator: test it first.
    if total_self:
        print("REFUTED: c2 emits self-moves.")
        print("  objs containing at least one: %d of %d (%.2f%%)"
              % (len(self_objs), nobj, 100.0 * len(self_objs) / max(nobj, 1)))
        print("  register named:  %s"
              % ", ".join("r%d x%d" % (r, n)
                          for r, n in self_reg.most_common()))
        print("  neighbours:      %s"
              % ", ".join("%s x%d" % ("|".join(k), n)
                          for k, n in self_nb.most_common(4)))
        print("  A single register and a branch-adjacent placement is an")
        print("  IDIOM, not a missed optimisation.  What this refutes is the")
        print("  LICENCE, not the code read: arm 6 plainly implements")
        print("  redundant-move deletion, and c2 still emits these.  Do not")
        print("  quote the [R] read as 'c2 emits no self-move'.")
        print("  first %d:" % len(examples))
        for path, off, kind, w in examples:
            print("  %s +%#x  %s  word=%#010x" % (path, off, kind, w))
        return 1
    if total_nonself == 0:
        print("VACUOUS: the scan decoded no move form at all, so `0 self-moves`")
        print("is a statement about the scan and not about c2.  NOT a pass.")
        return 2
    print("CONFIRMED on this corpus: %d move-form instructions were decoded and"
          % total_nonself)
    print("NONE of them is a self-move.  The liveness half is the %d: the scan"
          % total_nonself)
    print("demonstrably recognises these encodings, so the 0 is a measurement.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
