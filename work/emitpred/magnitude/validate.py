#!/usr/bin/env python3
"""validate.py — bound the detector's own error rate, at workload scale.

Three checks, all independent of the thing being measured:

  V-B  SPECIFICITY of the `67` discriminator.  A vtable slot can only ever hold
       a *virtual* function (or a `??_G`/`??_E` deleting destructor or a `??_9`
       vcall thunk).  So every `67`-edge target that demangles to a NON-virtual
       function is a detector artifact — a token match that landed two bytes
       after a `0x67` by accident.  The rate of those over all `67` targets in
       the workload is a direct upper bound on the artifact rate.

  V-C  TWO-SIDEDNESS.  axes1's mechanism says: a `67`-reached virtual is emitted
       iff the vtable rule fires, i.e. iff a constructor / destructor of its
       class is kept in the TU.  Partition every `67`-target that has a body in
       the TU by (emitted?) x (a ctor/dtor of its class emitted?).  The
       mechanism predicts the off-diagonal cells are empty.  Anything in them
       is either a detector error or a second, unmodelled effect.

  V-D  the hand-check sample is drawn here (`--sample N`), stratified by TU.

    usage: validate.py <ilroot> <truthroot> <tulist> [--sample N]
"""
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import model    # noqa: E402
import detect   # noqa: E402


def undname_batch(names):
    names = list(names)
    out = {}
    B = 20000
    for i in range(0, len(names), B):
        chunk = names[i:i + B]
        p = subprocess.run(["llvm-undname"], input="\n".join(chunk) + "\n",
                           stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                           encoding="latin1", timeout=600)
        lines = p.stdout.split("\n")
        j = 0
        for n in chunk:
            while j < len(lines) and lines[j] != n:
                j += 1
            if j >= len(lines):
                out[n] = None
                continue
            out[n] = lines[j + 1] if j + 1 < len(lines) else None
            j += 2
    return out


_VIRT = re.compile(r"^(public|protected|private):\s*virtual\s")


def kind_of(dec, dem):
    """virtual | nonvirtual | data | thunk | unknown"""
    if dec.startswith("??_G") or dec.startswith("??_E"):
        return "virtual"
    if dec.startswith("??_9") or dec.startswith("??_O") or "[thunk]" in (dem or ""):
        return "thunk"
    if dem is None or dem == dec:
        return "unknown"
    if _VIRT.match(dem):
        return "virtual"
    if "__cdecl" in dem or "__stdcall" in dem or "__thiscall" in dem or "__fastcall" in dem:
        return "nonvirtual"
    return "data"


def cls_of(dec):
    """The immediately-enclosing qualification of a decorated name, as the raw
    decorated scope string (`?v@C@@…` -> `C@@`).  Purely lexical; good enough to
    group ctor/dtor with their class, which is all it is used for."""
    if dec.startswith("??_G") or dec.startswith("??_E") or dec.startswith("??_7"):
        body = dec[4:]
    elif dec.startswith("??0") or dec.startswith("??1"):
        body = dec[3:]
    elif dec.startswith("?"):
        i = dec.find("@")
        if i < 0:
            return None
        body = dec[i + 1:]
    else:
        return None
    i = body.find("@@")
    return body[:i] if i >= 0 else None


def main():
    ilroot, truthroot, tulist = sys.argv[1:4]
    nsample = 0
    if "--sample" in sys.argv:
        nsample = int(sys.argv[sys.argv.index("--sample") + 1])
    srcs = [l.strip() for l in open(tulist) if l.strip()]

    vtargets = {}      # name -> count of (TU) it is a 67-target in
    cells = []         # (src, F, emitted?, class-has-emitted-ctor-or-dtor?)
    flagged = []
    for k, src in enumerate(srcs):
        d = os.path.join(ilroot, detect.slug(src))
        tf = os.path.join(truthroot, detect.slug(src) + ".txt")
        if not os.path.exists(tf) or not os.path.exists(os.path.join(d, "gl")):
            continue
        glb = open(os.path.join(d, "gl"), "rb").read()
        exb = open(os.path.join(d, "ex"), "rb").read()
        Nf = model.named_bodies(glb, exb)
        U = set(Nf.values())
        V, D = detect.edges_by_kind(glb, exb, Nf)
        E = set(x for x in open(tf).read().split() if x)
        ctor_classes = set()
        for n in E:
            if n.startswith("??0") or n.startswith("??1"):
                c = cls_of(n)
                if c:
                    ctor_classes.add(c)
        for f, callers in V.items():
            vtargets[f] = vtargets.get(f, 0) + 1
            if f not in U or not (callers & E):
                continue
            if D.get(f, set()) & E:
                continue
            cells.append((src, f, f in E, cls_of(f) in ctor_classes))
            if f not in E:
                flagged.append((src, f, sorted(callers & E)[:3]))
        if (k + 1) % 200 == 0:
            print("... %d/%d" % (k + 1, len(srcs)), flush=True)

    dm = undname_batch(sorted(vtargets))
    kinds = {n: kind_of(n, dm.get(n)) for n in vtargets}

    print("\n== V-B  specificity of the `67` discriminator")
    agg = {}
    inst = {}
    for n, c in vtargets.items():
        agg[kinds[n]] = agg.get(kinds[n], 0) + 1
        inst[kinds[n]] = inst.get(kinds[n], 0) + c
    print("   distinct 67-targets by kind:", agg)
    print("   TU-instances by kind:       ", inst)
    nv = [n for n in vtargets if kinds[n] == "nonvirtual"]
    print("   NON-VIRTUAL 67-targets (pure artifacts): %d distinct / %d instances"
          % (len(nv), sum(vtargets[n] for n in nv)))
    for n in sorted(nv, key=lambda x: -vtargets[x])[:15]:
        print("      %4d  %s" % (vtargets[n], n))

    print("\n== V-C  two-sidedness: (emitted?) x (ctor/dtor of its class emitted?)")
    tab = {}
    for _, f, em, ct in cells:
        tab[(em, ct)] = tab.get((em, ct), 0) + 1
    for em in (True, False):
        for ct in (True, False):
            print("   emitted=%-5s ctor_kept=%-5s  %6d" % (em, ct, tab.get((em, ct), 0)))
    off = [(s, f) for s, f, em, ct in cells if em != ct]
    print("   off-diagonal (mechanism violations): %d" % len(off))
    for s, f in off[:20]:
        print("      %s  %s  [%s]" % (s, f, kinds.get(f)))

    print("\n== flagged-set composition by kind")
    fk = {}
    for _, f, _c in flagged:
        fk[kinds.get(f, "?")] = fk.get(kinds.get(f, "?"), 0) + 1
    print("  ", fk)

    if nsample:
        import random
        random.seed(20260804)
        sm = random.sample(flagged, min(nsample, len(flagged)))
        json.dump([{"src": s, "name": f, "demangled": dm.get(f),
                    "kind": kinds.get(f), "emitted_callers": c}
                   for s, f, c in sm],
                  open(os.path.join(HERE, "sample.json"), "w"), indent=1)
        print("\nwrote sample.json (%d cells)" % len(sm))


if __name__ == "__main__":
    main()
