// **w-lineage** — the MUST-REFUSE side of the served mixed-kind store run.
//
// The whole TU must read `Port=NotImplemented` at **every** lane, never `Match`
// and never `mismatch`. Each function is a mixed run that `wlin_mixed_bind.cpp`
// differs from only in **where the address's stores are written**, and each has
// a measured counterexample behind it.
//
//   N_mirror  the address's stores go through the FORMAL's own path instead of
//             through the bind. **The ALLOCATION is right on this shape** — GRID
//             L grades it 30 of 30 by reading c2's register fields, the way all
//             twelve dead keys were graded — **and the ORDER is not**. Both
//             producers then share one base symbol, docs/SYMBOL.md's pin no
//             longer fixes the order, and real c2 INTERLEAVES the stores where
//             the port emits source order:
//
//               c2    li 10,3 ; addi 11,3,16 ; stw 10,0 ; stw 11,16 ;
//                                              stw 10,4 ; stw 11,20
//               port                          ; stw 10,0 ; stw 10,4 ;
//                                              stw 11,16 ; stw 11,20
//
//             Serving it put 11 of GRID L's 30 cells at `Port=Mismatch`
//             (`work/w-lineage`, the reverted experiment). Board #1298.
//
//   N_alias   the store root is a SECOND bind naming the SAME sub-object. This
//             is the only class that takes the `d` bonus, and it is the class
//             every grid on record confounded with the next one — so the bonus
//             attaches to two NAMES FOR ONE OBJECT, which `alloc::Root` cannot
//             state (`base` is a token and `offsets` is None at this seam).
//
//   N_two     the store root is a second bind naming a DIFFERENT sub-object.
//             c2 answers `const` at `cu = ru + 2`, where every rule on record
//             says `prod`, and `const` even at `(1,1)`. `H-LIN` and its four
//             twins are 10 wrong of 75 on this and `N_alias`'s class.
struct Pair { Pair* n0; Pair* n1; };
struct Box {
    int g0; int g1; int g2; int g3;
    Pair hub;
    Pair spare;
};

void N_mirror(Box* b) {
    Pair& h = b->hub;
    b->g0 = 3;
    b->g1 = 3;
    b->hub.n0 = &h;
    b->hub.n1 = &h;
}

void N_alias(Box* b) {
    Pair& h = b->hub;
    Pair& k = b->hub;
    b->g0 = 3;
    b->g1 = 3;
    b->g2 = 3;
    b->g3 = 3;
    k.n0 = &h;
    k.n1 = &h;
}

void N_two(Box* b) {
    Pair& h = b->hub;
    Pair& k = b->spare;
    b->g0 = 3;
    b->g1 = 3;
    b->g2 = 3;
    b->g3 = 3;
    k.n0 = &h;
    k.n1 = &h;
}
