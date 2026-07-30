// **Negative** — the neighbours of Class B (`mvp_call_seq_b.cpp`), each one step
// outside it and each refused in the IL parser so the census and the emission
// gate cannot disagree.
//
// ? `three_live` — three formals have to survive a call, and at three saved GPRs
//   c2 stops open-coding the stores: the prologue becomes `mflr r12 ;
//   bl __savegprlr_29 ; stwu r1,-112(r1)` and the epilogue a **tail branch**
//   `b __restgprlr_29` with no `blr` and no `stw r12,-8(r1)` at all. Measured:
//
//     void f(int a,int b,int c,int d){ v1(a); v2(b); v3(c); v1(d); }   60 B
//       mflr r12 ; bl __savegprlr_29 ; stwu r1,-112(r1)
//       mr r31,r4 ; mr r30,r5 ; mr r29,r6 ; bl … ; addi r1,r1,112
//       b __restgprlr_29
//
//   That needs a second REL24 site, two extra `/Gy` label slots
//   (`CODEGEN_FRAMED_CALLS.md` §4.4) and its own symbol-table position (§4.3),
//   all at once — `callseq-three-plus-saved`.
//
// ? `perm2`, `perm3`, `perm_mix` — the FIRST call PERMUTES its arguments while
//   something is saved. This is not the hoist/trail interleaving the positive
//   fixture models with a save moved around an unchanged r11 walk: when a
//   permuted argument's value is also callee-saved, c2 **breaks the cycle
//   through the callee-saved register and never emits r11 at all**, because the
//   save has to happen anyway. Three witnesses, none containing r11:
//
//     void f(int a,int b){ g2(b,a); v1(a); v2(b); }          a->r31, b->r30
//       mr r30,r4 ; mr r31,r3 ; mr r4,r3 ; mr r3,r30 ; bl ?g2
//     void f(int a,int b,int c){ g2(b,a); v1(a); v2(c); }    a->r31, c->r30
//       mr r31,r3 ; mr r3,r4 ; mr r4,r31 ; mr r30,r5 ; bl ?g2
//     void f(int a,int b,int c){ g3(a,c,b); v1(a); v2(b); }  a->r31, b->r30
//       mr r30,r4 ; mr r4,r5 ; mr r5,r30 ; mr r31,r3 ; bl ?g3
//
//   The hoist/trail model gets **11 of 17** probes wrong here, and it got them
//   wrong on a shape the narrower first gate had been refusing — found by
//   gridding the permutations before shipping, not by a fixture. Which saved
//   register serves as the temp when several are saved is not determined by
//   three captures, so the boundary is the measured edge
//   (`callseq-saved-with-first-call-setup`) and not a fit. Cost on the 878-TU
//   workload: 0 functions.
//
// ? `setup_comp`, `setup_self` — a COMPUTED first-call argument beside a save.
//   Under `/Ox` a chain intermediate goes to a fresh *descending* register,
//   which is the very file the saves live in, so the marshalling's write set is
//   not `{r3}` and the interleaving is not the measured one.
//
// ? `comp_after` — a later call's argument is COMPUTED from a saved formal, which
//   c2 emits as `addi r3,r31,1`: the operand stream rebased onto the callee-saved
//   register. That is a second lowering of `select_text`, not a use of it —
//   `callseq-saved-computed-arg`.
//
// Decode is all-or-nothing per TU, so the whole file must refuse.

extern void v0();
extern void v1(int);
extern void v2(int);
extern void v3(int);
extern void g2(int, int);
extern void g3(int, int, int);

// three saved GPRs -> the __savegprlr_29 helper class
void three_live(int a, int b, int c, int d) { v1(a); v2(b); v3(c); v1(d); }

// the first call PERMUTES while something is saved: c2 breaks the cycle through
// the callee-saved register, not r11
void perm2(int a, int b) { g2(b, a); v1(a); v2(b); }
void perm_mix(int a, int b, int c) { g2(b, a); v1(a); v2(c); }
void perm3(int a, int b, int c) { g3(a, c, b); v1(a); v2(b); }

// a COMPUTED first-call argument beside a save
void setup_comp(int a, int b) { v1(a + 1); v2(b); }
void setup_self(int a) { v1(a + 1); v2(a); }

// a computed argument out of a saved register
void comp_after(int a, int b) { v1(a); v2(b + 1); }
