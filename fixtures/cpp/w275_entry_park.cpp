// **BOARD #275 — the ENTRY-BLOCK PARK.** A permuted call behind a guarded early
// return: c2 hoists part of the argument shuffle *above* the guards, into the
// entry block, and the guards then compare whatever register their formal has
// landed in.
//
// `?mmioGetInfo` in `src/xdk/nuispeech/mmio.cpp` is `pk2` below plus a literal
// argument and a `memcpy`, and it is the head of the frontier by `.text` byte
// fraction (#502). Lane `w-clear` found this shape emitting **WRONG BYTES** —
// 30 `Port=Mismatch` of 54 cells — and refused it; lane `w-mmio` measured it
// over **886 cells** across three frozen grids and this is the sub-class all
// three agree about.
//
//     int f(void *a, void *b){ if(!a) return 5; g(b, a); return 0; }
//
//       mflr/stw/stwu
//       mr     r11,r3        <- THE PARK: the guard's own scrutinee
//       mr     r3,r4         <- and the ascending prefix of the chain
//       cmplwi cr6,r11,0     <- the guard reads the PARKED register, because
//       bf     26,+12           the entry block overwrote its home
//       li     r3,5
//       b      -> epilogue
//       mr     r4,r11        <- only the cycle-closing move is left here
//       bl     ?g
//       li     r3,0
//       addi/lwz/mtlr/blr
//
// **Three clauses, and each one has a cell in this file that separates it.**
//
//  1. SPLIT — the entry block ascends by destination, the call site descends
//     (`moves_descending`, already shipped for the *unguarded* permutation).
//     `pk3up` hoists two moves and `pk3dn` hoists one, off the same three
//     formals: `pk3dn`'s chain writes `r3, r5, r4` and the descent at `r4` is
//     where the split falls. That single cell was **all** the evidence board
//     #1414 had; here it is one of 51 witnesses over 10 register triples.
//
//  2. ANCHOR — the cycle is broken at the guard's **scrutinee**, not at the
//     cycle's lowest slot. #1414 says the lowest and scores **394 of 832**.
//     `pk3hi` guards the *highest* formal of its cycle and parks `r5`; `pk3up`
//     guards the lowest and parks `r3`. Same production, opposite ends.
//
//  3. COMPARE — the first guard reads its formal's **home** unless the entry
//     block overwrote it; every later guard reads wherever the value went.
//     `pk3hi`'s single guard reads `r5` **after** `mr r11,r5` has run, because
//     `r5` still holds the value; `pk2g2`'s second guard reads `r3`, the
//     register its formal was *moved into*, while its home `r4` is still live.
//     One map gets one of those wrong.
//
// The **unguarded** permutation is `ctl` and it is byte-exact from before this
// rung: it breaks the same cycle at the *other* end (`mr r11,r4`) and emits no
// entry block at all. Keeping it here is the point — a blanket rule in either
// direction loses one of the two.

void g2(void *, void *);
void g3(void *, void *, void *);

// The `?mmioGetInfo` shape, minus its literal: a two-formal swap behind one
// guard on the formal whose home the swap overwrites.
int pk2(void *a0, void *a1) {
    if (a0 == 0) return 5;
    g2(a1, a0);
    return 0;
}

// TWO guards over the same park. The second reads `r3` — where `a1` was moved
// to — and not `r4`, which still holds it.
int pk2g2(void *a0, void *a1) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g2(a1, a0);
    return 0;
}

// A three-cycle whose chain ASCENDS (`r3, r4, r5`): two moves hoisted, only the
// cycle-closing move left at the call.
int pk3up(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a1, a2, a0);
    return 0;
}

// The same three formals, the other rotation: the chain writes `r3, r5, r4` and
// the DESCENT at `r4` moves the split one instruction earlier — one hoisted,
// two left at the call. This is board #1414's `[2,0,1]`.
int pk3dn(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a2, a0, a1);
    return 0;
}

// The anchor cell: the guard is on the cycle's HIGHEST formal, and the park is
// `mr r11,r5` — not `mr r11,r3`, which is what board #1414's rule predicts.
// Its chain descends throughout, so nothing is hoisted past the park, and the
// guard compares `r5` rather than `r11`.
int pk3hi(void *a0, void *a1, void *a2) {
    if (a2 == 0) return 5;
    g3(a2, a0, a1);
    return 0;
}

// A cycle that does not touch slot 0, guarded on the formal it starts at.
int pk3mid(void *a0, void *a1, void *a2) {
    if (a1 == 0) return 5;
    g3(a0, a2, a1);
    return 0;
}

// **CONTROL — the same swap with NO guard.** A different cycle break (`r11`
// takes `r4`, not `r3`) and no entry block. Byte-exact before this rung and it
// must stay so.
int ctl(void *a0, void *a1) {
    g2(a1, a0);
    return 0;
}
