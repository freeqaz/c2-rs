#!/usr/bin/env python3
"""kagate.py — lane w-emit's known-answer controls KA1 and KA4.

KA1 reproduces `magnitude/gate.py`'s `67` virtual-slot discriminator.  Its
original cell tree (`axes1/detect/`) holds IL and objs and was therefore never
committed, so the four `mech` cells are RECONSTRUCTED from the verbatim
description in `docs/PHASE7_VALIDATION.md` §3b (the four call forms and their
emitted/not-emitted verdicts).  `mf1..mf4` are that reconstruction; the seven
`p_*` cells are the original committed sources, unmodified.  Stated so nobody
reads KA1 as a byte-identical re-run of w-emitpred's gate.

KA4 is new: it checks the `26`-prefixed DIRECT-edge extractor this lane's
headline rides on.  w-emitpred's gate validated the `67` side only.

    usage: kagate.py <outroot>
"""
import os
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "magnitude"))
import il      # noqa: E402
import model   # noqa: E402
import coff    # noqa: E402


VCALL_PREFIX = 0x67
DIRECT_PREFIX = 0x26


def edges(glb, exb, Nf):
    """{F: {owner}} for each of three edge kinds:
         v    — exb[p-2] == 0x67   (virtual dispatch)
         d26  — exb[p-1] == 0x26   (direct call/reference, TIGHT)
         dany — anything else      (the loose complement of v)
    Strict attribution: only `.gl`-named segments own edges.  No folding
    (attrib.py grades the folding rule correct on 1 of 14842)."""
    idx = il.gl_symbol_index(glb)
    v, d26, dany = {}, {}, {}
    n = len(exb)
    get = idx.get
    for (s, e) in il.segments(exb):
        owner = Nf.get(s)
        if owner is None:
            continue
        for p in range(s, min(e, n - 1)):
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
            else:
                tok = (exb[p] << 8) | b1
            f = get(tok)
            if f is None or f == owner:
                continue
            if p >= 2 and exb[p - 2] == VCALL_PREFIX:
                v.setdefault(f, set()).add(owner)
            else:
                dany.setdefault(f, set()).add(owner)
                if p >= 1 and exb[p - 1] == DIRECT_PREFIX:
                    d26.setdefault(f, set()).add(owner)
    return v, d26, dany


def load(out, name):
    d = os.path.join(out, name)
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    Nf = model.named_bodies(glb, exb)
    E = {n for n, _ in coff.text_comdat_entries(open(os.path.join(d, "x.obj"), "rb").read())}
    return Nf, set(Nf.values()), E, edges(glb, exb, Nf)


# ---------------------------------------------------------------- KA1
KA1 = [
    ("mf1", {"?v@C@@UAAHH@Z"}),      # pc->v(x)      virtual dispatch, no ctor kept
    ("mf2", set()),                  # pc->nv(x)     non-virtual member
    ("mf3", set()),                  # pc->C::v(x)   qualified / devirtualized
    ("mf4", set()),                  # &C::v         in a data initializer
    ("p_w", {"?w@C@@UAAHH@Z"}),
    ("p_u", {"?u@C@@UAAHH@Z"}),
    ("p_ref", {"?v@C@@UAAHH@Z"}),
    ("p_del", {"??_GC@@UAAPAXI@Z"}),
    ("p_base", {"?bv@B@@UAAHH@Z"}),
    ("p_mi", {"?v@MI@@UAAHH@Z", "?q@MI@@UAAHH@Z"}),
    ("p_ctor", set()),               # same call WITH a kept ctor
]


def ka1(out):
    fails = 0
    for name, want in KA1:
        Nf, U, E, (v, d26, dany) = load(out, name)
        cls = set()
        for f, callers in v.items():
            if f not in U or f in E:
                continue
            if not (callers & E):
                continue
            if dany.get(f, set()) & E:
                continue
            cls.add(f)
        ok = cls == want
        fails += 0 if ok else 1
        print("KA1 %-8s %s  class=%s%s" % (
            name, "PASS" if ok else "FAIL", sorted(cls),
            "" if ok else "   WANT " + str(sorted(want))))
    print("KA1: %d/%d pass" % (len(KA1) - fails, len(KA1)))
    return fails


# ---------------------------------------------------------------- KA4
# (cell, target, want_26_edge_from_an_emitted_body, want_target_emitted)
KA4 = [
    ("mf1", "?v@C@@UAAHH@Z", False, False),    # virtual dispatch: 67 only, no 26
    ("mf2", "?nv@C@@QAAHH@Z", True, True),     # non-virtual member call: 26 edge
    ("mf3", "?v@C@@UAAHH@Z", True, True),      # qualified call: 26 edge
    ("mf5", "?helper@@YAHH@Z", True, True),    # direct call to a kept static
    ("mf6", "?unused_helper@@YAHH@Z", False, False),  # unreferenced static: no edge
]


def ka4(out):
    fails = 0
    for name, tgt, want_edge, want_emit in KA4:
        Nf, U, E, (v, d26, dany) = load(out, name)
        got_edge = bool(d26.get(tgt, set()) & E)
        got_emit = tgt in E
        got_v = bool(v.get(tgt, set()) & E)
        ok = (got_edge == want_edge) and (got_emit == want_emit)
        fails += 0 if ok else 1
        print("KA4 %-8s %-26s %s  26-edge=%s(want %s) emitted=%s(want %s) 67-edge=%s in_U=%s"
              % (name, tgt, "PASS" if ok else "FAIL", got_edge, want_edge,
                 got_emit, want_emit, got_v, tgt in U))
    print("KA4: %d/%d pass" % (len(KA4) - fails, len(KA4)))
    return fails


def main():
    out = os.path.abspath(sys.argv[1])
    f = ka1(out)
    print()
    f += ka4(out)
    return 1 if f else 0


if __name__ == "__main__":
    sys.exit(main())
