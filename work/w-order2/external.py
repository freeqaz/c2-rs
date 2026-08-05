#!/usr/bin/env python3
"""external.py — lane w-order2. ORDER against the cells published by earlier
lanes, and against the one cell that disagrees with it.

  * `o7` — the cell `w-dclass`/B declared UNFITTED and `w-sched` worked in
    `docs/STORE_SCHEDULE.md` §1.2.
  * `xboxheap` — the FRONTIER's only branch-free TU, `STORE_SCHEDULE.md` §1.1.
    **TWO base symbols**, so it is OUT OF DOMAIN for this rule; it is scored
    here anyway because a refusal that is never checked is not a refusal.
  * the rank-order vs first-consumer-order discriminator, counted over this
    lane's whole grid.
"""
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


A = _load("wa_model", os.path.join(REPO, "work", "w-alloc", "model.py"))
O = _load("wo_order2", os.path.join(W, "order2.py"))


def show(name, specs, want_order, want_layout=None):
    got, relax = O.store_order(specs)
    ok = got == want_order
    print("  %-12s %-34s order %s  want %s  %s"
          % (name, ",".join(specs), "".join("%X" % k for k in got),
             "".join("%X" % k for k in want_order), "OK" if ok else "MISS"))
    if relax:
        print("       (relaxation fired %d times)" % relax)
    return ok


def main():
    print("== published cells ==")
    n = ok = 0

    # o7:  a = x;  b = 1;  c = 2;  d = 3;  e = y     (STORE_SCHEDULE.md §1.2)
    # store order a e b c d, producers at emitted positions 0, 2, 4.
    n += 1
    ok += show("o7", ["F0", "V0", "V1", "V2", "F1"], [0, 4, 1, 2, 3])
    seq = O.predict(["F0", "V0", "V1", "V2", "F1"], 2, "L")
    print("       full: %s" % " ".join(seq))
    want = "Pr11 S0@r4 Pr10 S4@r5 Pr9 S1@r11 S2@r10 S3@r9"
    print("       want: %s   %s" % (want, "OK" if " ".join(seq) == want
                                    else "MISS"))
    ok += (" ".join(seq) == want)
    n += 1

    # xboxheap (STORE_SCHEDULE.md §1.1). TWO base symbols: h for statements
    # 0..3, l for 4..5. OUT OF DOMAIN -- recorded, not scored as a hit.
    print()
    print("== xboxheap -- OUT OF DOMAIN (two base symbols) ==")
    xb = ["F0", "T", "V0", "T", "V1", "V1"]
    got, _ = O.store_order(xb)
    print("  single-symbol ORDER would give store order %s; c2 emits 012345."
          % "".join("%X" % k for k in got))
    print("  ORDER's rank order is V1,V0 (count 2 before count 1); c2 emits")
    print("  the producers P0 (li r10) then P1 (addi r11) -- FIRST-CONSUMER")
    print("  order. Board #561. The domain excludes it and the port refuses.")

    # Does this lane's own grid ever separate rank order from first-consumer
    # order?  If it does not, xboxheap is the ONLY evidence either way.
    print()
    print("== rank order vs first-consumer order, over this lane's grid ==")
    same = diff = diff_int = 0
    for f in ("fit.tsv", "holdout.tsv"):
        for cid, tier, nf, specs, kind, emitted, unc in A.load(
                os.path.join(W, f)):
            if unc or kind in ("M", "W"):
                continue
            pos = A.uses(specs)
            if not pos:
                continue
            rank = sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))
            order, _ = O.store_order(specs)
            slot = {k: q for q, k in enumerate(order)}
            byfirst = sorted(pos, key=lambda v: min(slot[k] for k in pos[v]))
            if rank == byfirst:
                same += 1
            else:
                diff += 1
                if O.head_slots(specs) > 0:
                    diff_int += 1
    print("  cells where the two orders AGREE      : %d" % same)
    print("  cells where they DISAGREE             : %d" % diff)
    print("    ... of those, with interleaved head : %d   <- xboxheap's regime"
          % diff_int)
    print()
    print("published cells: %d of %d" % (ok, n))


if __name__ == "__main__":
    main()
