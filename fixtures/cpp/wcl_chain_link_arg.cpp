// WCL — an argument on a LATER link of a chained member call: `p->a()->b(k)`.
//
// WCH shipped the chain with every link nullary, which is Class A: each call's
// result is already in r3, where the next call's `this` belongs, so nothing is
// ever live across a `bl`. One argument on a later link changes both halves of
// that. The argument IS live across the previous `bl`, so the body becomes
// Class B with a callee-saved formal; and it goes to **r4**, because slot 0 is
// the receiver the `bl` just produced.
//
//   int f(O* p, int k) { return p->Next()->gia(k); }        52 B, F = 96
//     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
//     mr r31,r4 ; bl ?Next ; mr r4,r31 ; bl ?gia
//     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
//
// The marshalling order is the OPPOSITE of every other call's in the port — see
// `c2_core::codegen::calls::link_setup_text`, which has the free-function
// captures beside these.
//
// Every function here must be in class: `c2rs census` N/N.

struct I {
    int gi();
    int ga(int);
    int gb2(int, int);
    int gb3(int, int, int);
    int g7(int, int, int, int, int, int, int);
    void va(int);
    I* pa(I*);
};

struct O {
    I* Next();
    I* NextA(int);
    I* NextB(int, int);
    O* Self();
    O* SelfA(int);
    int oa(int);
};

// ---- one formal argument on the outer link ----------------------------------
// The formal's REGISTER moves with the formals in front of it while the
// destination stays r4, which is the axis a one-parameter fixture cannot see.
int a1(O* p, int k) { return p->Next()->ga(k); }
int a2(O* p, int j, int k) { return p->Next()->ga(k); }
int a3(O* p, int j, int k) { return p->Next()->ga(j); }
int a4(O* p, int i, int j, int k) { return p->Next()->ga(k); }
int a5(int z, O* p, int k) { return p->Next()->ga(k); }

// …discarded rather than returned: the same body with a void tail.
void a6(O* p, int k) { p->Next()->va(k); }

// ---- two formal arguments, both orders --------------------------------------
// Two callee-saved GPRs (r31 then r30, in parameter order), and the slot list
// transposes with the source while the save assignment does not.
int b1(O* p, int j, int k) { return p->Next()->gb2(j, k); }
int b2(O* p, int j, int k) { return p->Next()->gb2(k, j); }
// The same value in two slots: two ordinary moves out of one saved register,
// which is NOT the dead-`mr r11` shape a slot-0 argument list refuses.
int b3(O* p, int j) { return p->Next()->gb2(j, j); }

// ---- literals, alone and mixed ----------------------------------------------
// A constant costs no register, so a link whose arguments are all literals
// leaves the body CLASS A — WCH's three-word prologue with one `li` added.
int c1(O* p) { return p->Next()->ga(7); }
int c2(O* p) { return p->Next()->gb2(7, 8); }
int c3(O* p) { return p->Next()->ga(-1); }
// Mixed: the `li` interleaves in slot order rather than being grouped.
int c4(O* p, int j) { return p->Next()->gb2(j, 9); }
int c5(O* p, int j) { return p->Next()->gb2(9, j); }
int c6(O* p, int j, int k) { return p->Next()->gb3(j, 5, k); }
int c7(O* p, int j, int k) { return p->Next()->gb3(5, j, k); }
int c8(O* p, int j, int k) { return p->Next()->gb3(j, k, 5); }
// SEVEN explicit arguments is the last slot list that fits: slot 0 is the
// receiver, so these occupy r4..r10 and the eighth would be stack-homed. The
// accepting side of `mcall-chain-link-arg-overflow`.
int c9(O* p) { return p->Next()->g7(1, 2, 3, 4, 5, 6, 7); }

// ---- depth ------------------------------------------------------------------
// A third link is one more `bl` and moves the marshalling to the last call.
int d1(O* p, int k) { return p->Self()->Next()->ga(k); }
int d2(O* p, int j, int k) { return p->Self()->Next()->gb2(j, k); }
// The argument on the INNERMOST link is the shipped permutation path, not this
// one — kept here because the two now coexist in one body.
int d3(O* p, int k) { return p->NextA(k)->gi(); }
int d4(O* p, int j, int k) { return p->NextA(j)->ga(k); }
int d5(O* p, int j, int k) { return p->NextB(j, k)->gi(); }
// …and the middle link of a three-link chain, which is the innermost call.
int d6(O* p, int k) { return p->SelfA(k)->Next()->gi(); }

// ---- the receiver is `this` --------------------------------------------------
// `this` is params[0] exactly as a declared formal is, so the argument's
// register still follows the formals in front of it.
struct H {
    O* Nx();
    int q(int k);
    int r(int j, int k);
};
int H::q(int k) { return Nx()->Next()->ga(k); }
int H::r(int j, int k) { return Nx()->Next()->gb2(j, k); }

// ---- a pointer-valued link argument -----------------------------------------
// The value class is a register either way; the `mr` is the same word.
I* e1(O* p, I* q) { return p->Next()->pa(q); }
