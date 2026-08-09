// **w-blockir — THE FLOAT ARRAY-WALK COUNTED LOOP**, and the fourth loop class
// this port emits.
//
// Every function below is `src/system/synth_xbox/IPP_basicmath_xbox.cpp`
// **verbatim**, with the header include dropped and nothing else changed. That
// TU is the whole positive population of this class on the 878-TU workload, it
// is 4 functions / 184 `.text` bytes / **zero relocations**, and it converted at
// this lane — `match` 18 → 19. This file compiles to the same four bodies word
// for word (48 / 36 / 48 / 52 B, checked against `work/w-blockir/ref/ipp.dis.txt`
// instruction by instruction), which is what licenses it as a fixture.
//
// # What c2 emits — three sub-shapes, and they are three because the WORDS differ
//
//     A Compound  b[i] OP= a[i]        mr    r11, b        <- the park, ABOVE the guard
//                                      cmplwi cr6, r3, 0   <- wb-loop pass 1
//                                      bclr  12, 26        <- …as a conditional RETURN
//                                      mtctr r3            <- wb-loop pass 2
//                                      sub   r10, a, b     <- the base difference
//                                      lfsx  f0, r10, r11
//                                      lfs   f13, 0(r11)
//                                      fOPs  f0, f0, f13
//                                      stfs  f0, 0(r11)
//                                      addi  r11, r11, 4
//                                      bdnz  .-20
//                                      blr
//     B Scalar    b[i] OP= s           cmplwi/bclr · addi r11,b,-4 · mtctr r3
//                                      lfs f0,4(r11) · fOPs f0,f0,f1
//                                      stfsu f0,4(r11) · bdnz .-12 · blr
//     C Binary    c[i] = a[i] OP b[i]  cmplwi/bclr · mr r11,b · mtctr r3
//                                      sub r10,a,b · sub r9,c,b
//                                      lfsx f0,r10,r11 · lfs f13,0(r11)
//                                      fOPs f0,f0,f13 · stfsx f0,r9,r11
//                                      addi r11,r11,4 · bdnz .-20 · blr
//
// The guard is `wb-loop`'s **pass 1** — the rotated pre-test, realised as a
// conditional return because the loop is the function's tail — and
// `mtctr`/`bdnz` is its **pass 2**. Shape B's `stfsu` is the shape of pass 3 and
// **is not an election of it**: `wb-loop` §4.4 gridded four update-form rivals
// over ten cells and elected none, `w-bdnz` declined the pass by name, and this
// is one transcribed word in a shape with four graded witnesses rather than a
// rule. The reader admits nothing else, so the general question is left exactly
// as open as `w-bdnz` left it.
//
// # `/O1` ONLY — and the `/Ox` cell is the reason, not the caution
//
// At `/Ox` c2 **unrolls this loop four times** behind its own `cmpwi cr6,r3,4`
// pre-test, with a remainder loop that re-derives the walker from
// `slwi r11,r9,2 · add r11,r11,r5`, an `lfsu`, and **688 bytes in one `.text`**
// where `/O1` emits 48 in four COMDATs
// (`work/w-blockir/probe/ipp_ox.dis.txt`). This TU is therefore
// `NotImplemented` at `/Ox` and `Match` at `/O1`, and the mode gate is asked in
// the **reader**, before any body byte, because a gate that lives only in the
// emitter is a fact the census cannot ask (board #1638).
//
// # The three constants this class transcribes, with their witness counts
//
// `docs/whitebox/WB_LOOP_FINDINGS.md` §4.3 states the base-difference reduction
// and then says of walker selection: *"In all five measured cells the walker is
// the array whose access is emitted last, which is circular. `#1767`'s rule
// against a two-point fit applies; not claimed."* This lane's 28-cell grid
// (`work/w-blockir/PROBES.md`) is not circular — it varies declaration order,
// source order, formal count and array count independently — and it still
// produces **three per-shape answers rather than one rule**. They ship as
// transcribed constants with their witness counts stated:
//
//   * the WALKER — the compound assign's destination (6 cells), the
//     later-DECLARED of two right-hand arrays (4), the sole array (4). It does
//     **not** extend to three right-hand arrays: cell `c4` walks the *second* of
//     three and c2 restructures the expression tree to get there;
//   * the PARK's position — above the guard in A, below it in B and C. The
//     register test this lane registered in advance is refuted from both sides
//     by cells `c5`, `c6` and `d5`;
//   * the LOAD ORDER — the other array first. `-=` and `/=` **swap the two
//     loads** (cells `c7`, `c8`), which is why they are in the `_neg` file and
//     not an extra opcode arm here.
//
// # The label counter
//
// `IlFunction::label_slots` returns `None` for this shape, so a TU pairing it
// with a framed function refuses whole — `wblockir_float_walk_then_framed_neg.cpp`
// is that cell and this file is its separating control. The measurement behind
// the `None` is `work/w-blockir/LABEL_LEAD.md`, taken by counterfactual at `/O1`
// and `/Ox`; `docs/LABEL_COUNTER.md`'s published table has been measured wrong
// by three lanes and is mode-dependent, so nothing here is quoted from it. This
// TU itself is **label-free** — 2 of the 9 frontier TUs are, and the scan prints
// which — so the charge cannot be what blocks it either way.

namespace IPP {
    // Shape A, `+=` — `?Add_InPlace@IPP@@YAXIPBMPAM@Z`, 48 B.
    void Add_InPlace(unsigned int size, const float *f1, float *f2) {
        if (size == 0)
            return;
        for (unsigned int i = 0; i < size; i++) {
            f2[i] += f1[i];
        }
    }

    // Shape B, `*=` with an FPR formal — `?MulConstant_InPlace@IPP@@YAXIPAMM@Z`,
    // 36 B, and the only cell in the file that takes the update form.
    void MulConstant_InPlace(unsigned int size, float *f1, float f2) {
        if (size == 0)
            return;
        for (unsigned int i = 0; i < size; i++) {
            f1[i] *= f2;
        }
    }

    // Shape A, `*=` — `?Mul_InPlace@IPP@@YAXIPBMPAM@Z`, 48 B. Separates the
    // operation from the shape: byte-identical to `Add_InPlace` but for the one
    // A-form word, and `fmuls` puts its operand in the **C** field where `fadds`
    // uses **B**, so the two are not one encoder with a varying XO.
    void Mul_InPlace(unsigned int size, const float *f1, float *f2) {
        if (size == 0)
            return;
        for (unsigned int i = 0; i < size; i++) {
            f2[i] *= f1[i];
        }
    }

    // Shape C — `?Mul@IPP@@YAXIPBM0PAM@Z`, 52 B. Two base differences, an X-form
    // store, and the walker is `f2` — the later-declared right-hand array, not
    // the destination and not the last formal.
    void Mul(unsigned int size, const float *f1, const float *f2, float *f3) {
        if (size == 0)
            return;
        for (unsigned int i = 0; i < size; i++) {
            f3[i] = f1[i] * f2[i];
        }
    }
}
