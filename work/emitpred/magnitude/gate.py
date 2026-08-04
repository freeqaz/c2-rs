#!/usr/bin/env python3
"""gate.py — known-answer gate for the `67` virtual-slot discriminator.

Runs detect.edges_by_kind + the class rule over cells whose emitted set is
known from a real obj compiled here, and asserts the expected class.

    axes1 mech f1  virtual dispatch, no ctor kept  -> class {?v@C@@UAAHH@Z}
    axes1 mech f2  non-virtual member call         -> class {}
    axes1 mech f3  qualified (devirtualized) call  -> class {}
    axes1 mech f4  &C::v in a data initializer     -> class {} (thunk, no vcall)
    a6c5 tu2       axes1's graded VIOLATION cell   -> class {?v@C@@UAAHH@Z}
    p_w/p_u/p_ref/p_del/p_base/p_mi   slot sweep   -> class = the dispatched fn
    p_ctor         same call WITH a kept ctor      -> class {} (all virtuals emit)
"""
import glob
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import model    # noqa: E402
import coff     # noqa: E402
import detect   # noqa: E402

CASES = [
    ("mechil/f1", "../axes1/detect/mech/f1.obj", {"?v@C@@UAAHH@Z"}),
    ("mechil/f2", "../axes1/detect/mech/f2.obj", set()),
    ("mechil/f3", "../axes1/detect/mech/f3.obj", set()),
    ("mechil/f4", "../axes1/detect/mech/f4.obj", set()),
    ("../axes1/detect/il_a6c5_tu2", "../axes1/detect/il_a6c5_tu2/tu2.obj", {"?v@C@@UAAHH@Z"}),
    ("probes/il_p_w", "probes/obj_p_w/x.obj", {"?w@C@@UAAHH@Z"}),
    ("probes/il_p_u", "probes/obj_p_u/x.obj", {"?u@C@@UAAHH@Z"}),
    ("probes/il_p_ref", "probes/obj_p_ref/x.obj", {"?v@C@@UAAHH@Z"}),
    ("probes/il_p_del", "probes/obj_p_del/x.obj", {"??_GC@@UAAPAXI@Z"}),
    ("probes/il_p_base", "probes/obj_p_base/x.obj", {"?bv@B@@UAAHH@Z"}),
    ("probes/il_p_mi", "probes/obj_p_mi/x.obj", {"?v@MI@@UAAHH@Z", "?q@MI@@UAAHH@Z"}),
    ("probes/il_p_ctor", "probes/obj_p_ctor/x.obj", set()),
]


def run(ild, objp):
    glb = open(glob.glob(os.path.join(HERE, ild, "_CL_*gl"))[0], "rb").read()
    exb = open(glob.glob(os.path.join(HERE, ild, "_CL_*ex"))[0], "rb").read()
    Nf = model.named_bodies(glb, exb)
    U = set(Nf.values())
    E = {n for n, _ in coff.text_comdat_entries(
        open(os.path.join(HERE, objp), "rb").read())}
    V, D = detect.edges_by_kind(glb, exb, Nf)
    cls = set()
    for f, callers in V.items():
        if f not in U or f in E:
            continue
        if not (callers & E):
            continue
        if D.get(f, set()) & E:
            continue
        cls.add(f)
    return cls, E


def main():
    fails = 0
    for ild, objp, want in CASES:
        got, E = run(ild, objp)
        ok = got == want
        fails += 0 if ok else 1
        print("%-38s %s  class=%s%s" % (
            ild, "PASS" if ok else "FAIL", sorted(got),
            "" if ok else "  WANT " + str(sorted(want))))
    print("gate: %d/%d pass" % (len(CASES) - fails, len(CASES)))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
