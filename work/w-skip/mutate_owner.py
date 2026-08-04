#!/usr/bin/env python3
"""mutate_owner.py — is the filter the OWNER'S OWN FATE rather than any flag bit?

`mutate_gate.py` established, against the sole judge, that no bit of the owner's
`+0x20` flag word — `0x20`, `0x40`, `0x400`, `0x4000`, and a whole value sweep —
changes whether an initializer node's retarget pulls a function in, while a wild
value at the same offset SIGSEGVs c2 (so the offset is right and c2 acts on it).

The one predicate left standing is w-mark's own §9 item 6, the one it named and
did not test: **whether the `in` owner is ITSELF emitted.**  `??_7HttpReq@@6B@`
is the TU's own vftable and it is in the obj; `??_7exception@std@@6B@` is a
header's vftable and it is not.

So this is the discriminating pair, and both arms can go red:

    H+  retarget a node on an owner that IS a defined symbol in the obj
        PREDICTION: F_new APPEARS
    H-  retarget a node on an owner that is NOT defined in the obj
        PREDICTION: F_new does NOT appear

Under w-mark's unfiltered reading H- must be green-as-APPEARS, so the two
readings make opposite predictions on the same mutation shape.

`defined_symbols` is deliberately the widest reading — any symbol with a real
section number — because a narrower one would let a decoder's blind spot look
like a filter.

    usage: mutate_owner.py <src.cpp> [n_per_arm]
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
sys.path.insert(0, os.path.join(REPO, "work", "w-mark"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il           # noqa: E402
import refs         # noqa: E402
import glowner      # noqa: E402
import mutate       # noqa: E402
import mutate_gate as mg   # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")


def defined_symbols(b):
    """{name} for every COFF symbol with a real section number (>= 1).

    Verbatim framing from `coff.text_comdat_entries`, widened from `.text`
    COMDAT leaders to every defined symbol, because the question is whether the
    DATA owner is in the obj at all."""
    nsec = struct.unpack_from("<H", b, 2)[0]
    psym = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    sym_end = psym + nsym * 18
    strtab = b[sym_end:]

    def str_at(i):
        e = strtab.find(b"\0", i)
        return strtab[i:e].decode("utf-8", "replace") if e >= 0 else None

    out = set()
    i = 0
    while i < nsym:
        o = psym + i * 18
        naux = b[o + 17]
        secnum = struct.unpack_from("<h", b, o + 12)[0]
        if 1 <= secnum <= nsec:
            if b[o:o + 4] == b"\0\0\0\0":
                nm = str_at(struct.unpack_from("<I", b, o + 4)[0])
            else:
                nm = b[o:o + 8].rstrip(b"\0").decode("utf-8", "replace")
            if nm:
                out.add(nm)
        i = i + 1 + naux
    return out


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 3

    bdir, base, argv, base_obj = mutate.capture(src)
    glp = os.path.join(bdir, base + "gl")
    inp = os.path.join(bdir, base + "in")
    glb0 = open(glp, "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()
    inb0 = open(inp, "rb").read()

    recs, _ = refs.scan(glb0, exb, wide_count=True)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    P = mg.closure(seed, refs.edges(glb0, recs, U), U, xskip)
    idx = il.gl_symbol_index(glb0)
    syms, _ = glowner.read_symbols(glb0)
    nodes = mg.nodes_with_owner(inb0)

    def replay():
        od = os.path.join(bdir, "h_fixed")
        shutil.rmtree(od, ignore_errors=True)
        return mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))

    rb, err = replay()
    if rb is None:
        raise SystemExit("baseline replay failed: " + err[-1500:])
    L0 = mutate.leaders(rb)
    D0 = defined_symbols(rb)
    print("bundle %s  baseline leaders %d  defined symbols %d  "
          "leader set == pipeline obj %s"
          % (base, len(L0), len(D0), mutate.leaders(base_obj) == L0))

    named = set(t for _o, t, _p, _w in nodes)
    pool = {2: [], 4: []}
    for tok, nm in idx.items():
        if nm in U and nm not in L0 and nm not in P and tok not in named:
            pool[4 if (tok >> 16) else 2].append((tok, nm))
    for k in pool:
        pool[k].sort(key=lambda x: x[1])

    plus, minus = [], []
    seen = set()
    for (o, t, tp, w) in nodes:
        r = syms.get(o)
        if r is None or r["kind"] != 1 or o in seen or not pool[w]:
            continue
        seen.add(o)
        (plus if r["name"] in D0 else minus).append((o, t, tp, w, r))
    print("owners with a node: %d ; owner IS defined in the obj: %d ; "
          "owner is NOT: %d" % (len(seen), len(plus), len(minus)))

    tally = {}
    for tag, lst in (("H+", plus), ("H-", minus)):
        hit = n = 0
        for (o, t, tp, w, r) in lst[:want]:
            tnew, nmnew = pool[w].pop()
            enc = (bytes([tnew >> 8 & 255, tnew & 255]) if w == 2 else
                   bytes([tnew >> 24 & 255, tnew >> 16 & 255,
                          tnew >> 8 & 255, tnew & 255]))
            mi = bytearray(inb0)
            mi[tp:tp + w] = enc
            assert len(mi) == len(inb0)
            open(inp, "wb").write(bytes(mi))
            mo, merr = replay()
            open(inp, "wb").write(inb0)
            if mo is None:
                print("  [%s] REPLAY FAIL on %s: %s"
                      % (tag, r["name"][:40], merr[-160:]))
                continue
            L1 = mutate.leaders(mo)
            app = nmnew in L1
            n += 1
            hit += 1 if app else 0
            print("  [%s] owner %-44s f20=%#06x  F_old %-34s -> F_new %-34s  "
                  "gained=%d lost=%d  APPEARS: %s"
                  % (tag, r["name"][:44], r["f20"], (idx.get(t) or "?")[:34],
                     nmnew[:34], len(L1 - L0), len(L0 - L1), app))
        tally[tag] = (hit, n)

    print("\n---- %s" % src)
    print("  H+ owner IS emitted      expect APPEARS      %d/%d" % tally["H+"])
    print("  H- owner is NOT emitted  expect suppressed   %d/%d appeared"
          % tally["H-"])


if __name__ == "__main__":
    main()
