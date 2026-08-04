#!/usr/bin/env python3
"""mutate_ref.py — KA-D: is a `.gl` reference-list ENTRY load-bearing for emit?

PREREG §4 KA-D. w-roots showed that flipping the `0x20` SEED bit changes the
obj. This asks the other half of the model: **does removing one EDGE remove the
COMDAT it was the only path to?**

The mutation is chosen so it is *byte-length preserving*, which matters because
the `.gl` is a byte stream with no length prefixes: the use count is written by
`i16c` (`10c1f9a6`), so a small positive count is **one byte**, and writing
`0x00` over it leaves every following record where it was. Per `10b9bfde`
(`test bx,bx / je`) a zero-use entry is parsed and then **not linked into the
list**, so the edge disappears and nothing else about the record moves.

    baseline replay  -> must reproduce the pipeline obj (leaders identical)
    mutate one edge  -> the target COMDAT must disappear, and nothing else

A survivor is reported with which of the two explanations applies — the edge is
not load-bearing, or another root reaches the target anyway — and scored as a
miss either way.

    usage: mutate_ref.py <src.cpp> [n]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il          # noqa: E402
import model       # noqa: E402
import refs        # noqa: E402
import mutate      # noqa: E402  (w-roots' capture/replay/leaders, reused verbatim)
from glflags import var_u, i16c, i32c   # noqa: E402


def walk_pairs(b, p, wide):
    """The same five reads as `refs.reflist`, carrying BYTE OFFSETS.

    Deliberately a second implementation: `main` asserts it agrees with
    `refs.reflist` token for token and count for count, so the duplication is a
    cross-check rather than a liability.
    """
    n, _ = (i32c(b, p) if wide else i16c(b, p))
    _, p = (i32c(b, p) if wide else i16c(b, p))
    if not wide:
        n &= 0xFFFF
    out = []
    for _ in range(max(0, n)):
        tp = p
        t = il.read_token_var(b, p)
        _, p = var_u(b, p)
        cp = p
        cnt, p = i16c(b, p)
        out.append((t[0], cnt & 0xFFFF, tp, cp))
    return out, p


def record_pairs(glb, exb, wide=True):
    """{name: [(token, count, tokpos, cntpos)]} for gate-clean tag-0x0E records."""
    import struct
    starts = set(il.split_ex(exb))
    runs = model.indexable_runs(glb)
    n = len(glb)
    out = {}
    for (s, e, nm, _sep) in runs:
        p, _sc = refs.head(glb, e + 1)
        if p is None or p + 5 > n or glb[p] != 0x80:
            continue
        if struct.unpack_from("<I", glb, p + 1)[0] not in starts:
            continue
        flags, _fp, _fw, q = refs.tail(glb, p)
        if not (flags & refs.LIST_BIT):
            continue
        try:
            pairs, _ = walk_pairs(glb, q, wide)
        except (IndexError, ValueError):
            continue
        out[nm] = pairs
    return out


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 5

    bdir, base, argv, base_obj = mutate.capture(src)
    print("bundle", base)
    glp = os.path.join(bdir, base + "gl")
    glb = open(glp, "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()

    recs, st = refs.scan(glb, exb, wide_count=True)
    print("decode:", {k: st[k] for k in ("recs", "list_bit", "term_ok", "term_bad",
                                         "pairs", "pairs_zero", "bound")})
    pairs = record_pairs(glb, exb)
    # cross-check the two independent walks
    bad = 0
    for nm, r in recs.items():
        a = [(t, c) for t, c, _ in r["refs"]]
        b = [(t, c) for t, c, _, _ in pairs.get(nm, [])]
        if a != b:
            bad += 1
    print("walk cross-check: %d of %d records disagree  [must be 0]" % (bad, len(recs)))
    if bad:
        raise SystemExit("the two reference-list walks disagree; refusing to mutate")

    rb, err = mutate.replay(bdir, base, argv, os.path.join(bdir, "b", "out.obj"))
    if rb is None:
        raise SystemExit("baseline replay produced no obj: " + err[-2000:])
    L0 = mutate.leaders(rb)
    b0 = bytearray(base_obj); b0[4:8] = b"\0\0\0\0"
    r0 = bytearray(rb); r0[4:8] = b"\0\0\0\0"
    print("BASELINE replay == pipeline obj (TimeDateStamp zeroed):", bytes(b0) == bytes(r0))
    print("baseline leaders:", len(L0))

    U = set(recs)
    idx = il.gl_symbol_index(glb)
    seeds = set(k for k, v in recs.items() if v["seed"])

    # in-edges over the DECODED list, restricted to U, zero-use entries dropped
    indeg = {}
    for nm, plist in pairs.items():
        for tok, cnt, _tp, cp in plist:
            if cnt == 0:
                continue
            f = idx.get(tok)
            if f is None or f == nm or f not in U:
                continue
            indeg.setdefault(f, []).append((nm, tok, cnt, cp))

    # candidates: emitted, NOT seeded, and reached by exactly ONE list edge whose
    # owner is itself emitted, with a one-byte (i16c small positive) use count
    cand = []
    for f, ins in sorted(indeg.items()):
        if f not in L0 or f in seeds or len(ins) != 1:
            continue
        owner, tok, cnt, cp = ins[0]
        if owner not in L0:
            continue
        if not (1 <= glb[cp] <= 0x7F):
            continue
        cand.append((f, owner, cnt, cp))
    print("\nKA-D  zero one reference-list edge (%d single-in-edge candidates)" % len(cand))

    hits = 0
    n = 0
    for f, owner, cnt, cp in cand[:want]:
        mg = bytearray(glb)
        mg[cp] = 0x00
        assert bytes(mg) != glb and len(mg) == len(glb)
        open(glp, "wb").write(bytes(mg))
        try:
            ob, err = mutate.replay(bdir, base, argv, os.path.join(bdir, "m", "out.obj"))
        finally:
            open(glp, "wb").write(glb)
        n += 1
        if ob is None:
            print("  NO-OBJ  %s  (owner %s)" % (f[:60], owner[:40]))
            continue
        L = mutate.leaders(ob)
        lost, gained = L0 - L, L - L0
        good = (lost == {f} and not gained)
        hits += good
        why = ""
        if not good and f in L:
            why = "  SURVIVOR: another root reaches it, or the entry is not the edge"
        print("  lost=%d gained=%d exact=%s  %s  <- %s (x%d)%s"
              % (len(lost), len(gained), good, f[:56], owner[:36], cnt, why))
        if lost and lost != {f}:
            print("        lost:", sorted(lost)[:4])
    print("  KA-D %d/%d" % (hits, n))


if __name__ == "__main__":
    main()
