// **W-VSNPRNC — THE SPLICED LITERAL: the formals in order with one constant
// inserted, tail-called.**
//
// `src/xdk/LIBCMT/vsnprnc.cpp`'s second function, `vsprintf_s`, is `p0` with an
// intra-TU target — twelve bytes, `mr r7,r6 ; li r6,0 ; b <callee>`. That TU
// does **not** convert (its leaf tail-calls its own framed function and the
// inline fence refuses the TU wholesale), so this file is where the class is
// graded.
//
// The clause this widens, `call-arg-lit-permuted`, was refused with the reason
// *"`g3(a,7,b)` is `mr r5,r4 ; li r4,7` and the same list over a real cycle is
// not characterized at all"* — a refusal that named the right bytes and declined
// to emit them for want of a grid. GRID-L (`work/w-vsnprnc/GRID-L.md`) is that
// grid: 18 cells over four families, arity 2…7, graded against the real
// `c2.dll` at the workload's own flags, one lowering with no exceptions —
//
//     the moves run in DESCENDING destination order and the `li` is LAST.
//
// ## THE FUNCTION ORDER IS THE LABEL TEST
//
// A wrong compiler-label charge on the LAST function of a TU moves nothing after
// it and the negative cell is inert (w-blockir §6). Every leaf here charges
// **0**, so the leaves come FIRST and a framed function comes LAST:
//
//     p0…p3   spliced-literal LEAVES, charge 0   <- a wrong charge shifts p4
//     p4      a FRAMED call                      <- carries the live $M/$T triple
//
// If any leaf charged the counter, `p4`'s `$M`/`$T` numbers would all shift and
// this fixture would fail. With `p4` first they could not.
//
// ## STRUCTURAL AXES, and they are STRUCTURAL
//
// `p0`…`p3` vary **arity (4, 2, 6, 2) and the literal's slot (3, 1, 0, 2)** — so
// they vary the *number of moves* (1, 1, 6, 0) and *where the `li` lands* — not
// just the constant's value. `p3` is the zero-move corner, which the shipped
// in-place path already handled and which is here as the control.
//
// ## STRUCTURAL BLIND SPOT of this fixture
//
// Every cell has exactly **one** literal, **int** formals throughout, an
// **external** callee, and a **tail** call. It cannot see a rule that depends on
// a second literal, on a non-`int` formal, on an intra-TU callee (the inline
// fence refuses those TUs anyway), or on a framed call site. It also cannot see
// the one cell that would separate "descending destination" from "a move whose
// source a pending literal overwrites" (board #1484) — in this whole family the
// two rules agree, and GRID-L says so in as many words.
//
// ## WHAT IS NOT HERE, AND WHY — `guard_chain_shared_tail`
//
// This lane also widened that class on four axes (arity 3–7, byte/halfword
// store, the `p[0]` subscript, and the sunk arm spelling). **None of it is
// fixture-graded, and it cannot be**: the class is `/O1`-only in the parser, the
// fixture path defaults to `/Ox`, and a fixture declaring `/O1` mismatches on a
// shape as simple as `int pz(int a,int b){return g(a,b);}` — **at master, before
// this lane**. No fixture has ever declared `/O1`, which is why that was
// invisible. The consequence is worth stating plainly: the shipped
// `wextdata_guard_chain_shared_tail.cpp` pair grades **`NotImplemented` at its
// own profile** and has never graded the class it was written for. Those axes
// are graded against the real `c2.dll` in `work/w-vsnprnc/GRID-{N,S,X}.md` and
// by unit tests against two reference objs instead.

int splice_target_5(int, int, int, int, int);
int splice_target_3(int, int, int);
int splice_target_7(int, int, int, int, int, int, int);
int framed_target_1(int);

// p0 — FOUR formals, the literal at slot 3: ONE move, and the `li` lands in the
// register that move just read. `vsprintf_s`'s twelve bytes.
//   mr r7,r6 ; li r6,0 ; b
int p0(int a, int b, int c, int e) {
    return splice_target_5(a, b, c, 0, e);
}

// p1 — TWO formals, the literal at slot 1: one move again, but at the bottom of
// the register file, so the move writes r5 and the `li` writes r4. This is the
// cell the shipped fence asserted must REFUSE.
//   mr r5,r4 ; li r4,7 ; b
int p1(int a, int b) {
    return splice_target_3(a, 7, b);
}

// p2 — SIX formals, the literal at slot 0: SIX moves, the whole argument file
// shifts up one, and the `li` lands in r3 last. The far edge of the family, and
// the only cell here whose literal is negative.
//   mr r9,r8 ; … ; mr r4,r3 ; li r3,-1 ; b
int p2(int a, int b, int c, int d, int e, int f) {
    return splice_target_7(-1, a, b, c, d, e, f);
}

// p3 — the literal LAST: ZERO moves, every formal already home. The control —
// the shipped in-place path handles this one and it must keep handling it.
//   li r5,3 ; b
int p3(int a, int b) {
    return splice_target_3(a, b, 3);
}

// p4 — a FRAMED call, LAST, so every leaf above it has somewhere to shift if its
// label charge is wrong. This is the cell that makes the four above a label test
// rather than four independent bodies.
int p4(int a) {
    return framed_target_1(a) + 1;
}
