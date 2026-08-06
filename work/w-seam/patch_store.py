#!/usr/bin/env python3
"""patch_store.py — apply lane w-seam's ONE test addition to
`crates/c2-core/src/codegen/leaf/store.rs`.

This exists because `work/w-seam/mutate.sh`'s restore trap runs
`git checkout --` on the files it mutates, which reverts anything uncommitted.
The first run of that script aborted on a bad pattern and took this lane's
uncommitted edits with it.  Keeping the addition in a re-runnable file rather
than in a shell transcript is the repair, and it is the same "one fact, one
locator" rule the crate's own docs keep applying.

Idempotent: it refuses if the test is already present.
"""

import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                    "..", ".."))
S = os.path.join(ROOT, "crates", "c2-core", "src", "codegen", "leaf", "store.rs")

ANCHOR = '''        assert_eq!(
            order::schedule(&split),
            None,
            "nsw = 3 is outside the layout's exact region"
        );
    }
}'''

ADDITION = r'''        assert_eq!(
            order::schedule(&split),
            None,
            "nsw = 3 is outside the layout's exact region"
        );
    }

    /// **Board #844 / #866 — the run text a FRAMED body would need is exactly
    /// this run text, and the `blr` is why no framed body can have it.**
    ///
    /// Lane `w-seam` compiled every configuration below in four body kinds at
    /// the workload's own `/O1 /Oi /EHsc /GR` and compared the *run text* — the
    /// disassembly with the frame words, the `bl` and the callee-saved copies
    /// stripped — against the leaf's, character for character
    /// (`work/w-seam/gridt.out`, `work/w-seam/gridt2.out`):
    ///
    /// ```text
    ///   L    void f(S*,int,int){ <run> }                  the control
    ///   P2   void f(S*,int,int){ <run> gx(); gy(); }      a FRAME (9 frame words)
    ///   R    S*  f(S*,int,int){ <run> gx(); return s; }   `this` live across it
    ///
    ///   GRID T   60 selected / 60 reached / 60 GRADED / 0 out-of-regime
    ///            P2 12/12 IDENT   R 12/12 IDENT
    ///   GRID T2  36 selected / 36 reached / 36 GRADED / 0 out-of-regime
    ///            34 IDENT, 2 DIFFER — both `D11-argcall`
    /// ```
    ///
    /// So [`order::schedule`] and [`alloc::allocate`], fitted entirely on leaf
    /// bodies, **transfer unchanged into a framed body when the run precedes
    /// the call**, and the `mr r31,r3` a live-across-the-call object needs is
    /// *additive* — it is inserted into the run without moving one other word.
    ///
    /// **The boundary is the call's ARGUMENT, and it is measured.** When the
    /// trailing call takes one (`gx(u)`), the run does not transfer at all:
    /// c2 parks the object in a **volatile** `r10`, the store base changes
    /// mid-run, and the constants re-rank to `r11`/`r9` where the leaf takes
    /// `r11`/`r10`. Any framed seam has to gate on that.
    ///
    /// This test pins the leaf side of three of those cells against the
    /// reference bytes, and pins the **structural** fact that makes them
    /// unreachable from a framed body: [`scheduled_gpr_run_text`] appends
    /// [`encode_blr`] unconditionally, so its text is a whole body and nothing
    /// can bracket it with a frame. A lane that composes the two has to change
    /// that line and will land here.
    #[test]
    fn the_scheduled_run_text_is_a_whole_body_and_ends_in_blr() {
        let mk = |ops: Vec<IlOp>, params: Vec<u32>| {
            let mut f = func_with(params, ops);
            f.mangled_name = "?w_seam@@YAXPAUS@@HH@Z".into();
            f
        };
        let lit_group = |b: u32, off: i32, k: i32| {
            vec![IlOp::Load(b), IlOp::Lit(k), IlOp::StoreInd { off, width: 4 }]
        };
        // Three formals — `void f(S* s, int u, int v)` — so the pool floor is
        // r6 and three producers still fit r11/r10/r9.
        let p3 = vec![0x0101u32, 0x0201, 0x0301];

        // `C5-const-3x1`: { s->f0=7; s->f8=9; s->fc=11; }
        let mut ops = lit_group(0x0101, 0, 7);
        ops.extend(lit_group(0x0101, 32, 9));
        ops.extend(lit_group(0x0101, 48, 11));
        assert_eq!(
            store_leaf_text(&mk(ops, p3.clone()), OptMode::O1).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, // li  r11,7
                0x39, 0x40, 0x00, 0x09, // li  r10,9
                0x39, 0x20, 0x00, 0x0B, // li  r9,11
                0x91, 0x63, 0x00, 0x00, // stw r11,0(r3)
                0x91, 0x43, 0x00, 0x20, // stw r10,32(r3)
                0x91, 0x23, 0x00, 0x30, // stw r9,48(r3)
                0x4E, 0x80, 0x00, 0x20, // blr   <- the leaf-only word
            ],
            "C5-const-3x1: the framed P2/R cells emit the first six words of \
             this between a stwu and a bl"
        );

        // `C11-const-inter`: { s->f0=7; s->f8=9; s->f1=7; s->f9=9; } — the
        // REVERSE-source-order tie (clause 4). Both constants are at 2 uses and
        // the LATER one takes r11.
        let mut ops = lit_group(0x0101, 0, 7);
        ops.extend(lit_group(0x0101, 32, 9));
        ops.extend(lit_group(0x0101, 4, 7));
        ops.extend(lit_group(0x0101, 36, 9));
        assert_eq!(
            store_leaf_text(&mk(ops, p3.clone()), OptMode::O1).unwrap().unwrap(),
            vec![
                0x39, 0x40, 0x00, 0x07, // li  r10,7
                0x39, 0x60, 0x00, 0x09, // li  r11,9   <- the LATER constant
                0x91, 0x43, 0x00, 0x00, // stw r10,0(r3)
                0x91, 0x63, 0x00, 0x20, // stw r11,32(r3)
                0x91, 0x43, 0x00, 0x04, // stw r10,4(r3)
                0x91, 0x63, 0x00, 0x24, // stw r11,36(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "C11-const-inter"
        );

        // `D9-run7`: seven stores, two producers at 4 and 3 uses.
        let mut ops: Vec<IlOp> = Vec::new();
        for i in 0..4 {
            ops.extend(lit_group(0x0101, i * 4, 7));
        }
        for i in 4..7 {
            ops.extend(lit_group(0x0101, i * 4, 9));
        }
        let t = store_leaf_text(&mk(ops, p3), OptMode::O1).unwrap().unwrap();
        assert_eq!(t.len(), 4 * 10, "two `li`s, seven stores and the blr");
        assert_eq!(
            &t[..8],
            [0x39, 0x60, 0x00, 0x07, 0x39, 0x40, 0x00, 0x09],
            "li r11,7 ; li r10,9 — 4 uses outranks 3"
        );
        // **The structural fact this rung is about.** Every accepted run's text
        // ENDS in `blr` — it is a whole body, so there is no seam that can put
        // it in the middle of a framed one (board #844).
        assert_eq!(
            &t[t.len() - 4..],
            &encode_blr()[..],
            "the run text is leaf-only by construction"
        );
    }
}'''


def main():
    s = open(S, encoding="utf-8").read()
    if "the_scheduled_run_text_is_a_whole_body_and_ends_in_blr" in s:
        print("already present — nothing to do")
        return 0
    if s.count(ANCHOR) != 1:
        print("anchor appears %d times — refusing" % s.count(ANCHOR))
        return 1
    open(S, "w", encoding="utf-8").write(s.replace(ANCHOR, ADDITION))
    print("store.rs patched")
    return 0


if __name__ == "__main__":
    sys.exit(main())
