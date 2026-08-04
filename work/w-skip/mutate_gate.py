#!/usr/bin/env python3
"""mutate_gate.py — the DECISIVE test of the owner skips: w-mark's retarget,
plus one gate bit, against the SOLE JUDGE.

w-mark established the causal fact this lane builds on: retargeting one `02`
node in the `in` stream to a function c2 was not going to emit makes exactly
that function's COMDAT appear — 15/15.  That is a mutation whose effect is
*known to be carried by the initializer walk*.

So the sharpest available test of a gate in `0x10b98e26` is to run **the same
retarget twice**, differing only in one bit of the owner's `.gl` flag word
`+0x20`:

    G0  retarget only                       -> F_new APPEARS      (w-mark, replicated)
    G1  retarget + owner f20 |= 0x20        -> SKIP 1 fires at 0x10b98e9f
                                               PREDICTION: F_new does NOT appear
    G2  retarget + owner f20 |= 0x60        -> the compare at 0x10b98ea2 does not
                                               match.  DISCRIMINATING CONTROL:
                                               PREDICTION: F_new APPEARS
    G3  retarget + owner f20 &= ~0x480      -> W2 refuses at 0x10b98b14/0x10b98c89
                                               PREDICTION: F_new does NOT appear
    G4  retarget + owner f20 |= 0x4000      -> SKIP 3 fires at 0x10b98ed9 (kind-1)
                                               PREDICTION: F_new does not appear
                                               *if* the S5 pass is what carried it

G0 is the positive control and it can go red on its own; G2 is the
discriminating control that says a write to that byte is not simply destructive.
**If G0 is green and G1/G3 are green too, the gates are causal.  If G0 is green
and G1/G2/G3/G4 all agree with G0, the gates are INERT for the emit set** — and
that is a finding about c2, not a broken instrument, because `probe_f20.py`
already showed a write at this offset changes the obj (P2) while `P3` shows the
harness moves COMDATs when a real emit bit is flipped.

Byte-length preserving throughout: the retarget swaps a 2-byte `varU` for a
2-byte `varU`; the flag write asserts `enc_var_u` keeps its width and never
touches bit `0x200`, which gates an extra field.

    usage: mutate_gate.py <src.cpp> [n]
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
import il           # noqa: E402
import refs         # noqa: E402
import instream     # noqa: E402
import glowner      # noqa: E402
import marks as mk  # noqa: E402
import mutate       # noqa: E402
from glflags import enc_var_u   # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")
ARMS = [("G0", None), ("G1", ("or", 0x20)), ("G2", ("or", 0x60)),
        ("G3", ("andnot", 0x480)), ("G4", ("or", 0x4000)),
        ("G5", ("or", 0x400)), ("G6", ("andnot", 0x4000)),
        # A VALUE SWEEP, so "the gate bits are inert" is not confused with
        # "this field is inert": if no value of +0x20 moves the emit set while
        # probe_f20's P2 shows a write here moves the obj, the field is read
        # for something other than the emit decision.
        ("S-min", ("set", 0x0001)), ("S-max", ("set", 0x7DFF)),
        ("S-walk", ("set", 0x0481))]


def closure(seed, edges, U, skip=()):
    seen = set(x for x in seed if x in U)
    stack = list(seen)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def nodes_with_owner(inb):
    """[(owner_token, node_token, token_offset, width)] over every `02` node."""
    out = []
    p, n = 0, len(inb)
    while p < n:
        if p == n - 1 and inb[p] == 0x07:
            break
        tag = inb[p]
        if tag not in instream.REC_TAGS:
            raise ValueError("tag")
        q = p + 1 + (1 if tag == 0x07 else 0)
        owner, q = instream.var_u_be(inb, q)
        _, q = instream.i32c(inb, q)
        while q < n and inb[q] not in instream.REC_TAGS:
            if inb[q] == instream.SYM_NODE:
                tp = q + 1
                t = il.read_token_var(inb, tp)
                out.append((owner, t[0], tp, t[1]))
            q = instream.node(inb, q, [])
        p = q
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
    P = closure(seed, refs.edges(glb0, recs, U), U, xskip)
    idx = il.gl_symbol_index(glb0)
    syms, _ = glowner.read_symbols(glb0)
    nodes = nodes_with_owner(inb0)

    def replay():
        od = os.path.join(bdir, "g_fixed")
        shutil.rmtree(od, ignore_errors=True)
        o, e = mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))
        return o, e

    rb, err = replay()
    if rb is None:
        raise SystemExit("baseline replay failed: " + err[-1500:])
    L0 = mutate.leaders(rb)
    print("bundle %s   baseline leaders %d   leader set == pipeline obj %s"
          % (base, len(L0), mutate.leaders(base_obj) == L0))

    # F_new pool: in U, NOT emitted, NOT Seed-reachable, named by no node
    named = set(t for _o, t, _p, _w in nodes)
    pool = {2: [], 4: []}
    for tok, nm in idx.items():
        if nm in U and nm not in L0 and nm not in P and tok not in named:
            pool[4 if (tok >> 16) else 2].append((tok, nm))
    for k in pool:
        pool[k].sort(key=lambda x: x[1])

    # candidate nodes: w-mark's OWN selection rule, so G0 replicates a mutation
    # already known to be carried by this channel -- F_old must be emitted,
    # named by exactly ONE `02` node, and NOT reachable from Seed -- and then
    # restricted to owners whose f20 this lane can write.
    per_token = {}
    for (o, t, tp, w) in nodes:
        per_token.setdefault(t, []).append((o, tp, w))
    cand = []
    for t, lst in per_token.items():
        if len(lst) != 1:
            continue
        nm = idx.get(t)
        if nm not in U or nm not in L0 or nm in P:
            continue
        o, tp, w = lst[0]
        r = syms.get(o)
        if r is None or r["kind"] != 1:
            continue
        if (r["f20"] & 0x60) != 0:
            continue          # SKIP 1 must not already fire
        if len(enc_var_u(r["f20"] | 0x4000)) != r["f20_width"]:
            continue
        if not pool[w]:
            continue
        cand.append((o, t, tp, w, r))
    cand.sort(key=lambda c: idx[c[1]])
    seen_owner = set()
    picked = []
    for c in cand:
        if c[0] in seen_owner:
            continue
        seen_owner.add(c[0])
        picked.append(c)
        if len(picked) >= want:
            break
    print("w-mark-style retarget candidates on writable kind-1 owners: %d "
          "(distinct owners %d), F_new pool 2-byte %d"
          % (len(cand), len(seen_owner), len(pool[2])))
    import collections as _c
    print("  owner f20 histogram over those candidates: %s"
          % _c.Counter("%#06x" % c[4]["f20"] for c in cand).most_common(8))

    tally = {a: [0, 0] for a, _ in ARMS}
    for (o, t, tp, w, r) in picked:
        tnew, nmnew = pool[w].pop()
        enc = (bytes([tnew >> 8 & 255, tnew & 255]) if w == 2 else
               bytes([tnew >> 24 & 255, tnew >> 16 & 255,
                      tnew >> 8 & 255, tnew & 255]))
        mi = bytearray(inb0)
        mi[tp:tp + w] = enc
        assert len(mi) == len(inb0)
        print("\nowner %-42s f20=%#06x\n   F_old %-46s -> F_new %s"
              % (r["name"][:42], r["f20"], idx.get(t, "?")[:46], nmnew[:46]))
        for arm, op in ARMS:
            if op is None:
                f20 = r["f20"]
            elif op[0] == "or":
                f20 = r["f20"] | op[1]
            elif op[0] == "set":
                f20 = (op[1] & ~0x200) | (r["f20"] & 0x200)
            else:
                f20 = r["f20"] & ~op[1]
            e = enc_var_u(f20)
            if len(e) != r["f20_width"]:
                print("  [%s] skipped: varU width would change" % arm)
                continue
            assert (f20 ^ r["f20"]) & 0x200 == 0
            mg = bytearray(glb0)
            mg[r["f20_pos"]:r["f20_pos"] + r["f20_width"]] = e
            open(glp, "wb").write(bytes(mg))
            open(inp, "wb").write(bytes(mi))
            mo, merr = replay()
            open(glp, "wb").write(glb0)
            open(inp, "wb").write(inb0)
            if mo is None:
                print("  [%s] REPLAY FAIL: %s" % (arm, merr[-160:]))
                continue
            L1 = mutate.leaders(mo)
            appeared = nmnew in L1
            oldgone = idx.get(t) not in L1
            tally[arm][1] += 1
            tally[arm][0] += 1 if appeared else 0
            print("  [%s] f20 %#06x  gained=%-3d lost=%-3d   F_new APPEARS: %-5s"
                  "  F_old gone: %s"
                  % (arm, f20, len(L1 - L0), len(L0 - L1), appeared, oldgone))

    print("\n---- %s   (n = per-arm replays)" % src)
    exp = {"G0": "APPEARS", "G1": "suppressed", "G2": "APPEARS",
           "G3": "suppressed", "G4": "suppressed", "G5": "APPEARS",
           "G6": "APPEARS", "S-min": "?", "S-max": "?", "S-walk": "?"}
    for a, _ in ARMS:
        hit, n = tally[a]
        print("  %-3s expect %-11s  F_new appeared in %d/%d" % (a, exp[a], hit, n))


if __name__ == "__main__":
    main()
