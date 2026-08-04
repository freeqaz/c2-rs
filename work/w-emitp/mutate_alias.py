#!/usr/bin/env python3
"""mutate_alias.py — the tag-0x10 ALIAS channel against the SOLE JUDGE.

PREREG §5.  The corpus scan says that resolving an `in` `02` node through the
tag-0x10 alias table takes the ORACLE from per-TU exact 151 to 472.  That is a
statement about a decode.  This asks `c2.dll` itself.

The mutation is w-joint's, unchanged in shape: retarget ONE `02` node's token
in the `in` stream and replay.  A `varU` token is 2 bytes iff `b1 & 0x80 == 0`,
so a 2-for-2 or 4-for-4 swap is byte-length preserving by construction and the
rest of the stream does not move.  The `in` file is restored between arms.

    H+   retarget to `??_E<B>` — a name with NO `.ex` body, in NO model's `U`,
         whose tag-0x10 record aliases `??_G<B>` which IS in `U` and is NOT in
         the baseline leader set.
         H-ALIAS predicts   `??_G<B>` APPEARS.
         Every landed model predicts NOTHING (the target is not in `U`).

    H-   retarget to a token naming a symbol that is in neither `U` nor
         `dom(alias)`.  Predicts NOTHING.  This is the arm that makes H+ mean
         something: without it, "the obj changed" is not "the alias was
         followed", and w-db §10a is the record of what a missing parity control
         costs.

    H0   rewrite the same token over itself.  Predicts a BYTE-IDENTICAL obj
         (TimeDateStamp zeroed).  Catches a harness that perturbs by writing.

    X4   in the H+ arm, does the ALIAS's own name appear as a COMDAT?  H-ALIAS
         says no — the target is emitted, the alias is not.

    usage: mutate_alias.py <src.cpp> [n_per_arm]

RUNS REAL c2.  The 21-TU quarantine is checked by name before anything is
written.
"""
import os
import shutil
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT",
                      os.path.abspath(os.path.join(HERE, "..", "..", "..",
                                                   "..", "..")))
for _p in (HERE,
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-joint"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import il             # noqa: E402
import refs           # noqa: E402
import glowner        # noqa: E402
import mutate         # noqa: E402
import mutate_gate as mg   # noqa: E402
import objsyms        # noqa: E402
import alias as al    # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")
HELDOUT = os.path.join(MAIN, "work", "emitpred", "magnitude", "heldout.txt")


def zero_stamp(b):
    return b[:4] + b"\0\0\0\0" + b[8:] if len(b) > 8 else b


def defined_symbols(b):
    o = objsyms.ObjSyms(b)
    if not o.ok:
        raise SystemExit("baseline obj unreadable: %s" % o.err)
    return set(objsyms.sets(o)["D_all"])


def check_quarantine(src):
    held = set(l.strip() for l in open(HELDOUT) if l.strip())
    if src in held:
        raise SystemExit("REFUSING: %s is in the 21-TU quarantine" % src)
    return len(held)


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    nheld = check_quarantine(src)

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
    AL, ALT, ast = al.scan(glb0)
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
    print("bundle %s  quarantine %d TUs (this TU is NOT one)" % (base, nheld))
    print("baseline leaders %d ; defined %d ; leader set == pipeline obj %s"
          % (len(L0), len(D0), mutate.leaders(base_obj) == L0))
    print("alias records %d bound %d ; targets in U %d ; dom(alias) in U %d"
          % (ast["tag10"], ast["bound"],
             sum(1 for v in AL.values() if v in U),
             sum(1 for k in AL if k in U)))

    named = set(t for _o, t, _p, _w in nodes)

    # ---- the H+ pool: aliases whose TARGET is a fresh, unemitted U name ----
    pool_plus = {2: [], 4: []}
    for atok, ttok in ALT.items():
        anm = idx.get(atok)
        tnm = idx.get(ttok)
        if anm is None or tnm is None:
            continue
        if tnm not in U or tnm in L0 or tnm in P:
            continue
        if anm in L0 or atok in named:
            continue
        pool_plus[4 if (atok >> 16) else 2].append((atok, anm, tnm))

    # ---- the H- pool: neither a body nor an alias ------------------------
    dom = set(ALT)
    pool_minus = {2: [], 4: []}
    for tok, nm in idx.items():
        if nm in U or tok in dom or tok in named or nm in L0:
            continue
        pool_minus[4 if (tok >> 16) else 2].append((tok, nm))
    for d in (pool_plus, pool_minus):
        for k in d:
            d[k].sort(key=lambda x: x[1])

    print("pool H+ %d/%d (2B/4B) ; pool H- %d/%d"
          % (len(pool_plus[2]), len(pool_plus[4]),
             len(pool_minus[2]), len(pool_minus[4])))

    # ---- pick nodes whose OWNER is a defined data symbol (w-skip's filter) --
    sites = []
    seen_owner = set()
    for (o, t, tp, w) in nodes:
        r = syms.get(o)
        if r is None or not r["name"] or r["name"] not in D0:
            continue
        if o in seen_owner:
            continue
        if not pool_plus[w] or not pool_minus[w]:
            continue
        seen_owner.add(o)
        sites.append((o, t, tp, w, r["name"]))
        if len(sites) >= want:
            break

    res = {"H+": [0, 0], "H-": [0, 0], "H0": [0, 0], "X4": [0, 0]}
    for i, (o, t, tp, w, oname) in enumerate(sites):
        old = inb0[tp:tp + w]
        for arm in ("H0", "H+", "H-"):
            if arm == "H0":
                newtok, newnm, tgt = t, idx.get(t), None
            elif arm == "H+":
                newtok, newnm, tgt = pool_plus[w][i % len(pool_plus[w])]
            else:
                newtok, newnm = pool_minus[w][i % len(pool_minus[w])]
                tgt = None
            # w-joint's encoder, verbatim: big-endian, width preserved
            enc = (bytes([newtok >> 8 & 255, newtok & 255]) if w == 2 else
                   bytes([newtok >> 24 & 255, newtok >> 16 & 255,
                          newtok >> 8 & 255, newtok & 255]))
            if len(enc) != w or il.read_token_var(enc + b"\0\0", 0)[0] != newtok:
                print("  SKIP %s: width %d != %d" % (arm, len(enc), w))
                continue
            mut = inb0[:tp] + enc + inb0[tp + w:]
            assert len(mut) == len(inb0)
            open(inp, "wb").write(mut)
            rb2, err2 = replay()
            open(inp, "wb").write(inb0)
            if rb2 is None:
                print("  %s replay FAILED: %s" % (arm, err2[-300:]))
                continue
            L2 = mutate.leaders(rb2)
            gained, lost = L2 - L0, L0 - L2
            if arm == "H0":
                same = zero_stamp(rb2) == zero_stamp(rb)
                res["H0"][0 if same else 1] += 1
                print("  [%d] H0  owner=%-38s IDENTICAL=%s (gained %d lost %d)"
                      % (i, oname[:38], same, len(gained), len(lost)))
            elif arm == "H+":
                ok = tgt in gained
                res["H+"][0 if ok else 1] += 1
                res["X4"][0 if newnm not in gained else 1] += 1
                print("  [%d] H+  %-34s -> alias %-30s TARGET %-34s "
                      "APPEARS=%s alias_appears=%s (gained %d lost %d)"
                      % (i, oname[:34], newnm[:30], tgt[:34], ok,
                         newnm in gained, len(gained), len(lost)))
            else:
                ok = len(gained) == 0
                res["H-"][0 if ok else 1] += 1
                print("  [%d] H-  %-34s -> %-34s INERT=%s (gained %d lost %d)"
                      % (i, oname[:34], newnm[:34], ok, len(gained), len(lost)))
        assert open(inp, "rb").read() == inb0

    print("\nRESULT %s  H+ %d/%d APPEARS   H- %d/%d INERT   H0 %d/%d IDENTICAL"
          "   X4 %d/%d alias-absent"
          % (src, res["H+"][0], sum(res["H+"]), res["H-"][0], sum(res["H-"]),
             res["H0"][0], sum(res["H0"]), res["X4"][0], sum(res["X4"])))


if __name__ == "__main__":
    main()
