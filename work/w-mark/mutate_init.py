#!/usr/bin/env python3
"""mutate_init.py — KA-D: is a DATA-INITIALIZER `0x02` node load-bearing for
emit, in BOTH directions, against the SOLE JUDGE?

PREREG §3 M11/M12.  w-roots showed the `0x20` seed bit is necessary and
sufficient.  w-refs showed a `.gl` reference-list entry is load-bearing on 9 of
15 edges.  This asks the question those two leave open: **does the `in` stream's
address-take node decide which function is emitted?**

The mutation RETARGETS one node rather than deleting it, which tests both
directions in one replay and is byte-length preserving by construction — a
`varU` token is 2 bytes iff `b1 & 0x80 == 0`, so swapping a 2-byte token for
another 2-byte token moves nothing else in the stream.

    pick  F_old : emitted, named by exactly one `02` node, NOT in closure(Seed)
    pick  F_new : in U, NOT emitted, NOT in closure(Seed), named by no `02` node,
                  and its token has the same varU width
    replay:  F_old must LOSE its COMDAT   (necessity, M12)
             F_new must GAIN its COMDAT   (sufficiency, M11)

A survivor is scored as a miss and reported with which explanation applies.

    usage: mutate_init.py <src.cpp> [n]
"""
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
sys.path.insert(0, os.path.join(REPO, "work", "w-refs"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il          # noqa: E402
import refs        # noqa: E402
import instream    # noqa: E402
import mutate      # noqa: E402  (w-roots' capture / replay / leaders, verbatim)

mutate.WORK = os.path.join(HERE, "mut")


def closure(seed, edges, U, skip=()):
    """w-refs' closure operator, transcribed — same three lines, no import
    ambiguity with `work/w-roots/scan.py`."""
    seen = set(x for x in seed if x in U)
    stack = list(seen)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def find_nodes(inb):
    """Every `02` node as (token, byte offset of the token, varU width).

    A second, offset-carrying walk of `instream.parse`'s grammar; `main` asserts
    the two agree token for token, so the duplication is a cross-check.
    """
    out = []
    p, n = 0, len(inb)
    while p < n:
        if p == n - 1 and inb[p] == 0x07:
            break
        tag = inb[p]
        if tag not in instream.REC_TAGS:
            raise ValueError("tag")
        q = p + 1 + (1 if tag == 0x07 else 0)
        _, q = instream.var_u_be(inb, q)
        _, q = instream.i32c(inb, q)
        while q < n and inb[q] not in instream.REC_TAGS:
            if inb[q] == instream.SYM_NODE:
                tp = q + 1
                t = il.read_token_var(inb, tp)
                out.append((t[0], tp, t[1]))
            q = instream.node(inb, q, [])
        p = q
    return out


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 5

    bdir, base, argv, base_obj = mutate.capture(src)
    print("bundle", base)
    inp = os.path.join(bdir, base + "in")
    glb = open(os.path.join(bdir, base + "gl"), "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()
    inb0 = open(inp, "rb").read()

    clean, irecs = instream.parse(inb0)
    nodes = find_nodes(inb0)
    flat = [t for _o, ts in irecs for t in ts]
    print("in: %d bytes, terminus clean=%s, %d `02` nodes; walks agree: %s"
          % (len(inb0), clean, len(nodes), flat == [t for t, _, _ in nodes]))
    if not clean or flat != [t for t, _, _ in nodes]:
        raise SystemExit("refusing to mutate: the two `in` walks disagree")

    recs, _st = refs.scan(glb, exb, wide_count=True)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    skip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)
    P = closure(seed, egl, U, skip)
    idx = il.gl_symbol_index(glb)

    rb, err = mutate.replay(bdir, base, argv, os.path.join(bdir, "b", "out.obj"))
    if rb is None:
        raise SystemExit("baseline replay produced no obj: " + err[-2000:])
    L0 = mutate.leaders(rb)
    b0 = bytearray(base_obj)
    b0[4:8] = b"\0\0\0\0"
    r0 = bytearray(rb)
    r0[4:8] = b"\0\0\0\0"
    print("BASELINE replay == pipeline obj (TimeDateStamp zeroed):",
          bytes(b0) == bytes(r0), " leaders:", len(L0))

    named = {}
    for t, tp, w in nodes:
        named.setdefault(t, []).append((tp, w))
    # F_old candidates: emitted, in U, unreachable from Seed, exactly one node
    olds = [(t, v[0][0], v[0][1]) for t, v in named.items()
            if len(v) == 1 and idx.get(t) in U and idx.get(t) in L0
            and idx.get(t) not in P]
    # F_new candidates: in U, NOT emitted, unreachable from Seed, no node at all
    # width is decided by the ENCODING actually present in the stream, so recover
    # it from a node that carries the same token width instead of guessing.
    by_w = {2: [], 4: []}
    for tok, nm in idx.items():
        if nm in U and nm not in L0 and nm not in P and tok not in named:
            by_w[4 if (tok >> 16) else 2].append((tok, nm))
    print("candidates: F_old %d   F_new(2-byte) %d  F_new(4-byte) %d"
          % (len(olds), len(by_w[2]), len(by_w[4])))

    olds.sort(key=lambda x: idx[x[0]])
    ok_new = ok_old = tried = 0
    for (told, tp, w) in olds[:want]:
        pool = by_w[w]
        if not pool:
            print("  no same-width F_new pool for width", w)
            continue
        tnew, nmnew = pool.pop()
        nmold = idx[told]
        enc = (bytes([tnew >> 8 & 255, tnew & 255]) if w == 2 else
               bytes([tnew >> 24 & 255, tnew >> 16 & 255,
                      tnew >> 8 & 255, tnew & 255]))
        mb = bytearray(inb0)
        mb[tp:tp + w] = enc
        assert len(mb) == len(inb0)
        open(inp, "wb").write(bytes(mb))
        od = os.path.join(bdir, "m%d" % tried)
        shutil.rmtree(od, ignore_errors=True)
        mo, merr = mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))
        open(inp, "wb").write(inb0)
        tried += 1
        if mo is None:
            print("  REPLAY FAIL %s: %s" % (nmold, merr[-200:]))
            continue
        L1 = mutate.leaders(mo)
        lost = L0 - L1
        gained = L1 - L0
        a = nmold in lost
        b = nmnew in gained
        ok_old += 1 if a else 0
        ok_new += 1 if b else 0
        print("  %-58s -> %-58s  lost=%d gained=%d  necessity=%s sufficiency=%s"
              % (nmold[:58], nmnew[:58], len(lost), len(gained), a, b))
        if not a or not b:
            print("      lost:   %s" % sorted(lost)[:4])
            print("      gained: %s" % sorted(gained)[:4])
    print("M12 necessity  %d/%d" % (ok_old, tried))
    print("M11 sufficiency %d/%d" % (ok_new, tried))


if __name__ == "__main__":
    main()
