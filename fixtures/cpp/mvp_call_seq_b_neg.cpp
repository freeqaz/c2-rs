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
// ? `setup_move`, `setup_perm`, `setup_comp` — the FIRST call needs argument
//   marshalling while something is saved, and where the save moves go is
//   **measured to be two rules**: a save whose source register the marshalling
//   overwrites is hoisted in front of the whole marshalling, one whose source it
//   leaves alone is emitted after it. Both halves in one capture,
//   `void f(int a,int b,int c,int d){ g2(a,d); v1(b); v2(c); }`:
//
//     mr r31,r4 ; mr r4,r6 ; mr r30,r5 ; bl ?g2
//
//   and a "save as late as possible" reading is refuted by
//   `void f(int a,int b,int c,int d,int e){ g3(a,d,e); v1(b); }`, where
//   `mr r31,r4` precedes *both* marshalling moves although only the second
//   touches r4. Deciding it needs "which registers does the first call's
//   marshalling write", i.e. a second implementation of what the emitter does —
//   `docs/GAPS.md` §6 #9's shape — so it is refused
//   (`callseq-saved-with-first-call-setup`) rather than modeled. Cost on the
//   878-TU workload: 0 functions.
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

// the first call marshals while something is saved
void setup_move(int a, int b, int c) { g2(a, c); v3(b); }
void setup_perm(int a, int b, int c) { g2(b, a); v3(c); }
void setup_comp(int a, int b) { v1(a + 1); v2(b); }
void setup_self(int a) { v1(a + 1); v2(a); }

// a computed argument out of a saved register
void comp_after(int a, int b) { v1(a); v2(b + 1); }
