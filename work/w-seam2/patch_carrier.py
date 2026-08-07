#!/usr/bin/env python3
"""Re-shape board #844's carrier from `Vec<IlOp>` to `Option<StoreRunPrefix>`.

Kept as a re-runnable file rather than a shell transcript, for `w-seam`'s own
reason (board #874): a mutation harness's restore trap reverted that lane's
uncommitted work, and an edit that lives in a script survives it.
"""

import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

DOC_OLD = """    /// **Board #844 — THE COMPOSITION CARRIER.** A **store run** emitted before
    /// the first call, as the store-group op stream
    /// [`crate::func::body::BodyShape::StoreRun`] carries: one
    /// `[Load(base), Load(formal) | Lit(k), StoreInd { off, width }]` group per
    /// statement, in source order. **Empty for every sequence every earlier rung
    /// shipped**, so no existing body changes shape by this field existing."""

DOC_NEW = """    /// **Board #844 — THE COMPOSITION CARRIER.** The store run this sequence
    /// emits *before* its call, and the one fact about the call the run's
    /// schedule depends on. `None` for every sequence every earlier rung
    /// shipped, so no existing body changes shape by this field existing."""

PREFIX = '''/// **Board #844** — the store run a [`CallSeq`] emits *before* its call, and the
/// one fact about the call that the run's schedule turns out to depend on.
///
/// # Why `live_args` is here, and why it is not derivable
///
/// The composition is admitted only when the call's argument setup is **empty**
/// (board #1129: every slot `i` already holds `params[i]`), so [`SeqCall`] has
/// nothing in `arg_ops` and the emitter cannot see how many arguments the call
/// takes. That looked like a fact nobody needed — and then it turned out to
/// decide the run's **order**:
///
/// ```text
///   void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }         the LEAF
///       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
///
///   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }     FRAMED
///       li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
///
///   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Reset(); }      FRAMED,
///       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; mr 31,3 ; stw 11,0(3) ; bl  nullary
/// ```
///
/// **The two unproduced stores swap — and only when the call passes `a`.** `a`
/// is live until the `bl`; `b` dies at its own store. With a nullary callee
/// nothing is kept alive and the run is the leaf's, word for word. So *"the leaf
/// schedule transfers unchanged into a framed body"* — board **#866** over 96
/// cells, and 34 more in `w-seam2`'s GRID S — is **true only while no store
/// reads a value the call keeps alive**, and this field is what lets the emitter
/// tell. `work/w-seam2/grid3/` is the twelve-cell probe that separated it and
/// `work/w-seam2/grid2/` is where it fired first, on seven cells at once.
///
/// `live_args` counts the argument slots **including the receiver at 0**, and
/// the receiver is exempt: `this` is the store base and is copied to `r31`
/// regardless, and storing it transfers on every measured cell (`p6`, `p11`, and
/// every `w3` cell of GRID S). It is the slots `>= 1` that break the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreRunPrefix {
    /// The run's op stream, exactly as [`crate::func::body::BodyShape::StoreRun`]
    /// carries it: one `[Load(base), Load(formal) | Lit(k), StoreInd { off,
    /// width }]` group per statement, in source order.
    pub ops: Vec<IlOp>,
    /// How many argument slots the call occupies, **receiver included**. Slot
    /// `i` holds `params[i]` by the production's own gate, so this is exactly
    /// "which formals are still live at the `bl`".
    pub live_args: usize,
}

impl CallSeq {'''


def main():
    p = os.path.join(ROOT, "crates/c2-il/src/func/mod.rs")
    s = open(p).read()
    assert s.count(DOC_OLD) == 1
    s = s.replace(DOC_OLD, DOC_NEW)
    old = "    pub store_run: Vec<IlOp>,\n}\n\nimpl CallSeq {"
    assert s.count(old) == 1
    s = s.replace(old, "    pub store_run: Option<StoreRunPrefix>,\n}\n\n" + PREFIX)
    open(p, "w").write(s)
    print("ok crates/c2-il/src/func/mod.rs")


if __name__ == "__main__":
    main()
