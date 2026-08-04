#!/usr/bin/env python3
"""mutate_joint.py — the joint fixpoint against the SOLE JUDGE, three arms.

w-skip ran the owner split on two TUs and got 10/10 against 0/10.  This lane
owes two things the sole judge can settle and a corpus scan cannot:

    M12  REPLICATION on THREE TUs w-skip did not use.  Same shape:
         retarget one `02` node's token to a function c2 was not going to emit,
         split only by whether the OWNER is a defined symbol in the baseline
         obj.  H+ must APPEAR, H- must not.

    M13  THE dd-EDGE ARM — the one that separates a FILTER from a FIXPOINT,
         and the one I registered as most likely to refute my own model.
         Pick an owner `d'` that is NOT defined in the obj but IS named by an
         `02` node of an owner that IS defined.

             a pure "owner in D" FILTER  predicts NO appearance
             the JOINT FIXPOINT          predicts APPEARANCE

         So the two readings make opposite predictions on the same mutation,
         and whichever way it lands, something is refuted.

The H- arm is tightened against w-skip's: an owner is only H- if it is neither
defined NOR dd-reachable from a defined owner.  w-skip's H- did not make that
split, so if the dd-edge is real some of its H- draws were actually DD draws —
which is exactly the confusion this arm exists to remove.

The mutation is byte-length preserving by construction: a `varU` token is 2
bytes iff `b1 & 0x80 == 0`, so a 2-for-2 or 4-for-4 swap moves nothing else in
the stream.  The `in` file is restored between every arm.

    usage: mutate_joint.py <src.cpp> [n_per_arm]

RUNS REAL c2.  Never point it at a quarantined TU — the held-out list is
checked by name before anything is written.
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
sys.path.insert(0, os.path.join(REPO, "work", "w-skip"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il             # noqa: E402
import refs           # noqa: E402
import glowner        # noqa: E402
import mutate         # noqa: E402
import mutate_gate as mg   # noqa: E402
import objsyms        # noqa: E402
import joint          # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")
HELDOUT = os.path.join(REPO, "work", "emitpred", "magnitude", "heldout.txt")


def defined_symbols(b):
    """The widest reading — every symbol with a real section number — which is
    the definition `w-skip/mutate_owner.py` used for its 10/10 vs 0/10."""
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
    print("baseline leaders %d ; defined symbols %d ; leader set == pipeline "
          "obj %s" % (len(L0), len(D0), mutate.leaders(base_obj) == L0))

    # ---- the dd-reachable set, from the same decode the scan uses ----
    own = {}
    for (o, t, _tp, _w) in nodes:
        r = syms.get(o)
        if r is None or not r["name"]:
            continue
        nm = idx.get(t)
        if nm is not None:
            own.setdefault(r["name"], set()).add(nm)
    Rd = set(d for d in own if d in D0)
    live, _code = joint.data_fixpoint(own, Rd, U)
    dd_only = (live - D0)      # reached ONLY through a dd-edge
    print("owners with a named node %d ; owners in D %d ; dd-reachable but NOT "
          "in D %d" % (len(own), len(Rd), len(dd_only)))

    named = set(t for _o, t, _p, _w in nodes)
    pool = {2: [], 4: []}
    for tok, nm in idx.items():
        if nm in U and nm not in L0 and nm not in P and tok not in named:
            pool[4 if (tok >> 16) else 2].append((tok, nm))
    for k in pool:
        pool[k].sort(key=lambda x: x[1])

    arms = {"H+": [], "H-": [], "DD": []}
    seen = set()
    for (o, t, tp, w) in nodes:
        r = syms.get(o)
        if r is None or not r["name"] or o in seen or not pool[w]:
            continue
        seen.add(o)
        nm = r["name"]
        if nm in D0:
            arms["H+"].append((o, t, tp, w, r))
        elif nm in dd_only:
            arms["DD"].append((o, t, tp, w, r))
        else:
            arms["H-"].append((o, t, tp, w, r))
    print("candidates  H+ %d   H- %d   DD %d"
          % (len(arms["H+"]), len(arms["H-"]), len(arms["DD"])))

    tally = {}
    for tag in ("H+", "H-", "DD"):
        hit = n = 0
        for (o, t, tp, w, r) in arms[tag][:want]:
            if not pool[w]:
                break
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
            print("  [%s] owner %-46s f20=%#07x  F_new %-34s  gained=%d "
                  "lost=%d  APPEARS: %s"
                  % (tag, r["name"][:46], r["f20"], nmnew[:34],
                     len(L1 - L0), len(L0 - L1), app))
        tally[tag] = (hit, n)

    print("\n---- %s" % src)
    print("  M12  H+  owner IS defined            expect APPEARS      %d/%d"
          % tally["H+"])
    print("  M12  H-  owner NOT defined, NOT dd    expect suppressed   %d/%d "
          "appeared" % tally["H-"])
    print("  M13  DD  owner NOT defined but dd-reachable from a defined owner")
    print("           FIXPOINT predicts APPEARS ; pure FILTER predicts not "
          "  %d/%d appeared" % tally["DD"])


if __name__ == "__main__":
    main()
