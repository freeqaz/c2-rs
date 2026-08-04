#!/usr/bin/env python3
"""mutate_cd.py — MUT-CD: does a reference FROM CODE define a DATA symbol?

This is the claim the whole lane rests on, and w-joint's `joint.py` asserts its
negation:

    cc  f -> RGL(f) … so there is NO code->data edge and the data half cannot
    be reached from code

That assertion comes from w-skip T-e, which is right about the pass it names:
`0x10b27f3c` resolves `[head+0x14]` into `[head+0xc]` and keeps an edge only for
a tag-`0x0E` function target.  But `[head+0xc]` is the list **`Mark`** walks, and
`Mark`'s `+0x4c |= 0x20` is the **CODE** emit bit — data symbols are not emitted
by `Mark` at all.  They are emitted by the COFF writer's own recursion,
`0x10b28a9b`, guarded by `[sym+0x32] & 1` and re-entered from `0x10b28cb9` /
`0x10b29057`, which is a second closure over the *unpruned* relation.

So the discriminating experiment is: **retarget one token of a function's `.gl`
reference list to a DATA symbol that is not defined in the baseline obj**, and
ask the sole judge whether it becomes defined.

    H+  the function IS emitted        PREDICTION: the data symbol APPEARS
    H-  the function is NOT emitted    PREDICTION: it does NOT

Both arms can go red.  If H- comes back green-as-APPEARS the edge is not
conditional on the referrer being emitted; if H+ comes back inert the code->data
edge does not exist and the lane's §0 is withdrawn.

The retarget is byte-length preserving by construction: a `varU` token is 2
bytes iff `b1 & 0x80 == 0`, so only a same-width swap is ever written, and the
length is asserted before the file is touched.

MUT-CD2 (`--datatok`) is the CLEAN arm, added after MUT-CD's first run came
back 0/5 with 56..120 symbols of collateral churn per replay (disclosed under
prereg clause 5).  Replacing a function's FIRST list token destroys that
function's own closure, so whether the payload appears is confounded.  With
`--datatok` the token replaced already points at a DATA symbol, so the pruned
Mark channel (`0x10b27f3c` drops non-function targets) sees no change at all and
the only variable left is the data target itself.

    usage: mutate_cd.py <src.cpp> [n_per_arm] [--datatok]
"""
import os
import shutil
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
sys.path.insert(0, os.path.join(REPO, "work", "w-refs"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il           # noqa: E402
import refs         # noqa: E402
import glowner      # noqa: E402
import mutate       # noqa: E402

mutate.WORK = os.path.join(HERE, "mut_cd")
HELDOUT = os.path.join(REPO, "work", "emitpred", "magnitude", "heldout.txt")


def check_quarantine(src):
    q = set(l.strip() for l in open(HELDOUT) if l.strip())
    if src in q:
        raise SystemExit("QUARANTINED TU refused: " + src)
    print("quarantine check: %d held-out TUs, %s is not among them" % (len(q), src))


def zts(b):
    return b[:4] + b"\0\0\0\0" + b[8:]


def defined_symbols(b):
    """{name} for every COFF symbol with a real section number — the widest
    reading, verbatim from w-skip's `mutate_owner.defined_symbols`."""
    nsec = struct.unpack_from("<H", b, 2)[0]
    psym = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    strtab = b[psym + nsym * 18:]

    def str_at(i):
        e = strtab.find(b"\0", i)
        return strtab[i:e].decode("utf-8", "replace") if e >= 0 else None

    out = set()
    i = 0
    while i < nsym:
        o = psym + i * 18
        naux = b[o + 17]
        sec = struct.unpack_from("<h", b, o + 12)[0]
        if 1 <= sec <= nsec:
            nm = (str_at(struct.unpack_from("<I", b, o + 4)[0])
                  if b[o:o + 4] == b"\0\0\0\0"
                  else b[o:o + 8].rstrip(b"\0").decode("utf-8", "replace"))
            if nm:
                out.add(nm)
        i += 1 + naux
    return out


def enc(tok, w):
    return (bytes([tok >> 8 & 255, tok & 255]) if w == 2 else
            bytes([tok >> 24 & 255, tok >> 16 & 255,
                   tok >> 8 & 255, tok & 255]))


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2].isdigit() else 5
    datatok = "--datatok" in sys.argv
    check_quarantine(src)

    bdir, base, argv, pipeline_obj = mutate.capture(src)
    glp = os.path.join(bdir, base + "gl")
    glb0 = open(glp, "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()
    inb = open(os.path.join(bdir, base + "in"), "rb").read()

    def replay():
        od = os.path.join(bdir, "o_fixed")      # ONE path for every arm
        shutil.rmtree(od, ignore_errors=True)
        return mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))

    b0, err = replay()
    if b0 is None:
        raise SystemExit("baseline replay failed: " + err[-1500:])
    D0 = defined_symbols(b0)
    L0 = mutate.leaders(b0)
    print("bundle %s ; baseline defined %d ; leaders %d ; leader set == "
          "pipeline obj %s" % (base, len(D0), len(L0),
                               mutate.leaders(pipeline_obj) == L0))

    recs, _ = refs.scan(glb0, exb, wide_count=True)
    gidx = il.gl_symbol_index(glb0)
    syms, _ = glowner.read_symbols(glb0)

    # the payload pool: DATA symbols (kind-1 `.gl` records) that are NOT
    # defined in the baseline obj, keyed by the token's varU width.  vftables
    # and RTTI records are preferred, because that is the class the model gets
    # right corpus-wide and the class w-skip's owner split was built on.
    pool = {2: [], 4: []}
    for r in syms.values():
        nm, tok = r.get("name"), r.get("tok")
        if r["kind"] != 1 or nm is None or tok is None:
            continue
        if nm in D0:
            continue
        pool[4 if (tok >> 16) else 2].append((tok, nm))
    pref = ("??_7", "??_R")
    for k in pool:
        pool[k].sort(key=lambda x: (x[1].startswith(pref), x[1]))
    print("payload pool: %d 2-byte, %d 4-byte undefined DATA symbols"
          % (len(pool[2]), len(pool[4])))

    U = set(recs)
    plus, minus = [], []
    for nm, r in recs.items():
        for tok, cnt, tp in r["refs"]:
            if not cnt:
                continue
            t = il.read_token_var(glb0, tp)
            if t is None or t[1] not in (2, 4):
                continue
            tgt = gidx.get(tok)
            if datatok:
                # the token must ALREADY name a data symbol, and one that is
                # defined -- so the swap moves a live data reference and
                # nothing else.
                if tgt is None or tgt in U or tgt not in D0:
                    continue
            (plus if nm in L0 else minus).append((nm, tok, tp, t[1], tgt))
            break
    plus.sort(key=lambda x: x[0])
    minus.sort(key=lambda x: x[0])
    print("referrers with a usable list token: EMITTED %d ; NOT emitted %d"
          % (len(plus), len(minus)))

    tally = {}
    for tag, lst in (("H+", plus), ("H-", minus)):
        hit = n = 0
        used = 0
        for (nm, tok, tp, w, oldn) in lst:
            if n >= want:
                break
            if not pool[w]:
                continue
            tnew, nmnew = pool[w].pop()
            mg = bytearray(glb0)
            mg[tp:tp + w] = enc(tnew, w)
            assert len(mg) == len(glb0)
            open(glp, "wb").write(bytes(mg))
            b1, e1 = replay()
            open(glp, "wb").write(glb0)
            if b1 is None:
                print("  [%s] REPLAY FAIL on %s: %s" % (tag, nm[:40], e1[-160:]))
                continue
            D1 = defined_symbols(b1)
            app = nmnew in D1
            n += 1
            hit += 1 if app else 0
            used += 1
            print("  [%s] referrer %-34s  old %-30s -> new DATA %-34s  "
                  "gained=%-3d lost=%-3d  old LOST: %-5s  new APPEARS: %s"
                  % (tag, nm[:34], (oldn or "?")[:30], nmnew[:34],
                     len(D1 - D0), len(D0 - D1),
                     (oldn is not None and oldn not in D1), app))
        tally[tag] = (hit, n)

    print("\n---- %s" % src)
    print("  H+ referrer IS emitted      expect APPEARS     %d/%d" % tally["H+"])
    print("  H- referrer is NOT emitted  expect suppressed  %d/%d appeared"
          % tally["H-"])


if __name__ == "__main__":
    main()
