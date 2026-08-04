#!/usr/bin/env python3
"""probe_f20.py — DOES a write at my computed `+0x20` offset reach c2 at all?

Every arm of `mutate_skip.py` came back inert on `EventTrigger.cpp`.  Inert is
exactly what a mislocated write looks like, and exactly what a semantically
inert field looks like, so the two must be separated before anything is read
into the result.  This is that separation, and it is a control that can go red
in either direction:

  P1  a NO-OP write (the same bytes back)      -> the obj must be BYTE-IDENTICAL
                                                  (proves the replay is stable)
  P2  set bit 0x200 at the same offset         -> `0x10b9ba5f` then reads an
                                                  EXTRA varU, so the record
                                                  DESYNCS.  The obj must change,
                                                  or c2 must fail.
  P3  the w-roots POSITIVE CONTROL, in this same script: clear the seed bit
      `0x20` at `+0x4c` on an emitted leaf     -> its COMDAT must disappear.
      This grades the harness independently of anything this lane decoded.

If P3 moves and P2 does not, the `+0x20` offset is wrong.
If P2 moves and the skip arms do not, `+0x20` is located and the gates are inert.

    usage: probe_f20.py <src.cpp>
"""
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
sys.path.insert(0, os.path.join(REPO, "work", "w-refs"))
sys.path.insert(0, os.path.join(REPO, "work", "w-mark"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il          # noqa: E402
import refs        # noqa: E402
import glowner     # noqa: E402
import marks as mk  # noqa: E402
import mutate      # noqa: E402
from glflags import enc_var_u   # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")


def main():
    src = sys.argv[1]
    bdir, base, argv, base_obj = mutate.capture(src)
    glp = os.path.join(bdir, base + "gl")
    glb0 = open(glp, "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()
    inb = open(os.path.join(bdir, base + "in"), "rb").read()
    recs, _ = refs.scan(glb0, exb, wide_count=True)
    U = set(recs)
    idx = il.gl_symbol_index(glb0)
    syms, _ = glowner.read_symbols(glb0)
    _ok, inrecs = mk.parse_records(inb)

    def norm(b):
        """TimeDateStamp (file offset 4..8) zeroed — CLAUDE.md's own criterion."""
        if b is None:
            return None
        x = bytearray(b)
        x[4:8] = b"\0\0\0\0"
        return bytes(x)

    def replay(tag):
        """ONE fixed output path for every replay.  A per-arm path makes the obj
        differ for a reason that has nothing to do with the mutation (the path
        lands in the obj), which is what made P1 red on the first run."""
        od = os.path.join(bdir, "p_fixed")
        shutil.rmtree(od, ignore_errors=True)
        o, e = mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))
        return norm(o), e

    rb, err = replay("base")
    if rb is None:
        raise SystemExit("baseline replay failed: " + err[-1500:])
    L0 = mutate.leaders(rb)
    print("baseline leaders %d ; leader set == pipeline obj %s"
          % (len(L0), mutate.leaders(base_obj) == L0))

    # pick the same owner mutate_skip would: a kind-1 owner naming emitted fns
    owned = {}
    for _t, _f, o, toks in inrecs:
        for t in toks:
            nm = idx.get(t)
            if nm in U:
                owned.setdefault(o, set()).add(nm)
    cands = [(o, n) for o, n in owned.items()
             if o in syms and syms[o]["kind"] == 1 and (n & L0)]
    cands.sort(key=lambda kv: -len(kv[1] & L0))
    if not cands:
        raise SystemExit("no candidate owner")
    tok, names = cands[0]
    r = syms[tok]
    print("owner %s  f20=%#06x at gl[%d..%d] width %d  names %d emitted"
          % (r["name"], r["f20"], r["f20_pos"], r["f20_pos"] + r["f20_width"],
             r["f20_width"], len(names & L0)))
    print("  bytes there now: %s   enc(f20)=%s"
          % (glb0[r["f20_pos"]:r["f20_pos"] + r["f20_width"]].hex(" "),
             enc_var_u(r["f20"]).hex(" ")))

    def write_f20(v):
        e = enc_var_u(v)
        assert len(e) == r["f20_width"], (v, len(e), r["f20_width"])
        mb = bytearray(glb0)
        mb[r["f20_pos"]:r["f20_pos"] + r["f20_width"]] = e
        open(glp, "wb").write(bytes(mb))

    def restore():
        open(glp, "wb").write(glb0)

    # ---- P1: no-op write -------------------------------------------
    write_f20(r["f20"])
    o1, e1 = replay("p1")
    restore()
    print("P1 no-op write        -> obj identical to baseline: %s"
          % (o1 is not None and o1 == rb))

    # ---- P2: set 0x200, which must desync the record ----------------
    if not (r["f20"] & 0x200):
        write_f20(r["f20"] | 0x200)
        o2, e2 = replay("p2")
        restore()
        if o2 is None:
            print("P2 set 0x200          -> c2 FAILED (desync): %s" % e2[-200:])
        else:
            L2 = mutate.leaders(o2)
            print("P2 set 0x200          -> obj changed: %s  leaders %d->%d "
                  "(lost %d gained %d)"
                  % (o2 != rb, len(L0), len(L2), len(L0 - L2), len(L2 - L0)))
    else:
        print("P2 skipped: owner already has 0x200")

    # ---- P2b: a wild value at the same offset -----------------------
    wild = (r["f20"] ^ 0x7000) & 0x7FFF
    if len(enc_var_u(wild)) == r["f20_width"]:
        write_f20(wild)
        o2b, e2b = replay("p2b")
        restore()
        if o2b is None:
            print("P2b wild %#06x       -> c2 FAILED: %s" % (wild, e2b[-200:]))
        else:
            L2b = mutate.leaders(o2b)
            print("P2b wild %#06x       -> obj changed: %s  leaders %d->%d"
                  % (wild, o2b != rb, len(L0), len(L2b)))

    # ---- P3: the w-roots positive control, seed bit at +0x4c --------
    leaf = None
    for nm in sorted(L0):
        v = recs.get(nm)
        if v is not None and v["seed"]:
            leaf = (nm, v)
            break
    if leaf is None:
        print("P3 skipped: no seeded emitted record")
        return
    nm, v = leaf
    from glflags import enc_var_u as ev
    e = ev(v["flags"] & ~0x20)
    if len(e) != v["fwidth"]:
        print("P3 skipped: width change")
        return
    mb = bytearray(glb0)
    mb[v["fpos"]:v["fpos"] + v["fwidth"]] = e
    open(glp, "wb").write(bytes(mb))
    o3, e3 = replay("p3")
    restore()
    if o3 is None:
        print("P3 clear seed on %s -> c2 FAILED: %s" % (nm[:40], e3[-200:]))
    else:
        L3 = mutate.leaders(o3)
        print("P3 clear seed 0x20 at +0x4c on %s -> lost %d (target lost: %s)"
              % (nm[:50], len(L0 - L3), nm not in L3))


if __name__ == "__main__":
    main()
