#!/usr/bin/env python3
"""grid2gen.py — GRID2, a DECLARED POST-HOC HOLDOUT on the one axis GRID freezes.

**Declared post-hoc, and the declaration is the point.** GRID (22 cells, frozen
at `2edec474` before the first `cl.exe`) crosses the bind against the store-run
shape — but its two "generic run + call" cells (`b_run_call`, `b_ctrl_runcall`)
are *void member functions*, and board #1131 restricts the composition to the
CONSTRUCTOR tail, so both refuse on the tail before the bind is ever reached.
The consequence, found by reading the graded table rather than by design: the
ONLY cell in GRID that exercises the call tail with a bind in it is
`b_target_bind` — `xboxheap.cpp` itself.

That is fitting to one TU on axis G, which the lane brief names as how all six
refuted allocation keys got written. GRID2 is the repair: four constructors that
are not `xboxheap`, each varying something the target holds fixed, frozen in
`GRID2.sha256` before any of them is compiled.

It is scored as a HOLDOUT — a prediction is written per cell, in this file,
before the grader runs.
"""
import os
import sys

# cell -> (predicted verdict of the reader, why, source)
CELLS = {
    # PREDICTED: StoreRunBind, the call tail. Nothing here resembles xboxheap
    # except the shape: three members, a different layout, the bind at +4 rather
    # than +8, the bind FIRST rather than in the middle, one formal rather than
    # two, and a nullary callee.
    "c2_ctor_bind_call": (
        "store-run-bind-no-emitter-carrier:eof",
        "a constructor that is not xboxheap: bind at +4, bind FIRST, one formal, "
        "a NULLARY callee",
        """struct Node { Node* n; Node* p; };
struct Q {
    unsigned mTag;   // 0
    Node mRing;      // 4  (n at 4, p at 8)
    unsigned mLen;   // 12
    Q(unsigned t);
    void Reset();
};

Q::Q(unsigned t) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
    Reset();
}
""",
    ),
    # PREDICTED: StoreRunBind, the call tail. A WIDE displacement (+64) and a
    # longer run, so the bound base's own bound (`bind.off + off` must encode)
    # is exercised past the target's +8.
    "c2_ctor_bind_wide": (
        "store-run-bind-no-emitter-carrier:eof",
        "the bind at +64 with a five-store run — the bound base's displacement "
        "SUM, which the target exercises only at 8 and 12",
        """struct Node { Node* n; Node* p; };
struct W {
    unsigned pad[16];  // 0..63
    Node mRing;        // 64 (n at 64, p at 68)
    unsigned mA;       // 72
    unsigned mB;       // 76
    W(unsigned a, unsigned b);
    void Reset();
};

W::W(unsigned a, unsigned b) {
    mA = a;
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mB = b;
    Reset();
}
""",
    ),
    # PREDICTED: StoreRunBind, the PLAIN tail. The same constructor with the call
    # removed — so the pair separates "the bind reads" from "the call tail reads".
    "c2_ctor_bind_nocall": (
        "store-run-bind-no-emitter-carrier:eof",
        "the same constructor with NO trailing call — the plain run tail",
        """struct Node { Node* n; Node* p; };
struct Q {
    unsigned mTag;
    Node mRing;
    unsigned mLen;
    Q(unsigned t);
};

Q::Q(unsigned t) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
}
""",
    ),
    # PREDICTED: **REFUSED** — and NOT under this lane's key. Board #1129's
    # regime gate: the callee takes an argument that is not already in its slot,
    # so the call's setup writes r3 and the run stops transferring. If this cell
    # comes back under `store-run-bind-*` the production has widened past the
    # gate it inherited, which is a reader over-accept.
    "c2_ctor_bind_argsetup": (
        "NOT store-run-bind-no-emitter-carrier:eof",
        "board #1129's break: the callee's argument setup WRITES a register, so "
        "the composition must refuse — inherited, not re-derived",
        """struct Node { Node* n; Node* p; };
struct R {
    unsigned mTag;
    Node mRing;
    unsigned mLen;
    R(unsigned t, unsigned u);
    void Grow(unsigned n);
};

R::R(unsigned t, unsigned u) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
    Grow(u);
}
""",
    ),
}


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    for name, (pred, why, body) in CELLS.items():
        d = os.path.join(here, "grid2", name)
        os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write("// GRID2 (declared POST-HOC HOLDOUT) cell `%s`\n" % name)
            f.write("// PREDICTED, before this cell was compiled: %s\n" % pred)
            f.write("// WHY IT IS HERE: %s\n" % why)
            f.write(body)
    print("wrote %d holdout cells" % len(CELLS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
