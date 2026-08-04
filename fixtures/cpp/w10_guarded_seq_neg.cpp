// **W10 NEGATIVE** — the four shapes next door to the guarded call sequence
// that must stay REFUSED. Every one of them was compiled with the real
// toolchain at both `/O1` and `/Ox` before it was written down
// (`work/w-cross/p/probe{2,3,5}.cpp`), and every one emits something the port
// would get wrong rather than something it would merely not emit.
//
// If any of these ever censuses in class, the W10 gate has over-accepted.
//
// ---- 1. `nelse` — an `else` arm. THE MODE-DEPENDENT LAYOUT. ----------------
//
// This is the one that was built, graded, and taken back out, and it is the
// finding this rung is worth reading for:
//
//     /O1                                /Ox and /O2
//     52 B                               68 B
//     cmpwi cr6,r3,0                     cmpwi cr6,r3,0
//     bt    26,+12                       bt    26,+28
//     bl    ?v0                          bl    ?v0
//     b     +8        <- intra-section   bl    ?v2      <- the JOIN, duplicated
//     bl    ?v1                          addi  r1,r1,96 <- the EPILOGUE, too
//     bl    ?v2                          lwz   r12,-8(r1)
//     addi  r1,r1,96                     mtlr  r12
//     …                                  blr
//                                        bl    ?v1
//                                        bl    ?v2
//                                        addi  r1,r1,96
//                                        …
//
// `/Ox` **tail-duplicates the join block and the whole epilogue** and emits no
// `b` at all. That **refutes `docs/OPT_MODE.md`** and `c2_core::codegen::OptMode`'s
// own doc comment, both of which state that the two modes "differ in exactly
// one rule … never a different opcode, never a different operand order — only a
// register field". Here they differ in **block structure**.
//
// The duplication has a size threshold, and it is bracketed by exactly one cell
// on each side: at `/Ox`, a **one**-call join duplicates and a **two**-call
// join emits `/O1`'s shared `b` (`probe5.cpp`, `j1` against `j2`/`j3`). Fitting
// a threshold there is fitting a c2 cost model, which `docs/CFG_SHAPE.md` §3.5
// declined for the fold table for the identical reason. So the `3A <join>` is
// refused by name in `c2_il`'s `guarded_seq`, and board **#191**'s
// intra-section `b` stays open with a much sharper characterization than it
// had. The positive file's `n2`/`n3` are the control that says the
// mode-dependence belongs to the `else` and not to the guard: a ONE-ARMED guard
// does not duplicate at any join length.
//
// ---- 2. `npark` — a guarded arm that needs an entry-block scratch park -----
//
// `a2(b,a)` is a two-slot permutation with a cycle, so `a` must be parked. c2
// hoists **both** `mr r11,r3` and `mr r3,r4` ABOVE the compare and leaves only
// `mr r4,r11` in the arm, and the compare then reads **r11** rather than r3:
//
//     mr    r11,r3 ; mr r3,r4 ; cmpwi cr6,r11,0 ; bt 26,+12 ; mr r4,r11 ; bl ?a2
//
// The rule that decides which moves hoist and which stay is not one rule:
// `probe3::P0` parks in **r10** (r11 being taken by a local) and leaves
// `mr r4,r5` in the arm, where `probe2::s4` hoists its `mr r3,r4`. Three cells,
// zero tests. `work/w-cross/PREREG.md` §3.2 clause 2 forbids fitting it, so a
// guarded arm takes at most one argument and the entry block is empty by
// construction.
//
// ---- 3. `nsaved` — a formal live across the guarded call ------------------
//
// `b` is read after the first call, so it goes to a callee-saved r31 with a
// `std`/`ld` pair around the frame — and the save move lands in the entry
// block, before the compare. Whether the compare then reads r3 or r31 depends
// on whether the entry block also clobbers r3 (`probe3::S0` reads r3,
// `probe3::P2` reads r31), which composes with the park rule above. Refused
// together with it.
//
// ---- 4. `nbare` — a guard with no unguarded call after it -----------------
//
// `void nbare(int a){ if(a) v0(); }` is **not framed at all**. It is fold band
// 2 plus a tail call — `cmpwi cr6,r3,0 ; bnelr cr6 ; b ?v0 ; blr`, 16 B, **no
// `.pdata`** (`probe2::e0`). Emitting the 44-byte framed body there is a
// wrong-bytes obj that still links, which is why the refusal is named in the
// parser rather than left to the tail-call escape — that escape would have
// handed the guarded call to `tail_call_shape` and silently dropped the branch.

extern void v0();
extern void v1();
extern void v2();
extern void a1(int);
extern void a2(int, int);

void nelse(int a) { if (a != 0) v0(); else v1(); v2(); }
void npark(int a, int b) { if (a != 0) a2(b, a); v1(); }
void nsaved(int a, int b) { if (a != 0) v0(); a1(b); }
void nbare(int a) { if (a != 0) v0(); }
