// **Class B — values live across calls** (`docs/GAPS.md` #35 step 2, rung 4;
// `docs/CODEGEN_FRAMED_CALLS.md` §2.2). A framed many-call body in which one or
// two formals have to survive a `bl`, so c2 copies them into callee-saved GPRs
// behind an inline `std`/`ld` pair.
//
// The prologue/epilogue delta against Class A, byte-exact at `/O1 /GS- /c`:
//
//   void f(int a,int b){ v1(a); v2(b); }               52 B, F = 96, 1 saved
//     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
//     mr r31,r4 ; bl ?v1 ; mr r3,r31 ; bl ?v2
//     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
//
//   void f(int a,int b,int c){ v1(a); v2(b); v3(c); }  72 B, F = 112, 2 saved
//     … std r30,-24(r1) ; std r31,-16(r1) ; stwu r1,-112(r1)
//     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r3,r31 ; bl ?v2 ; mr r3,r30 ; bl ?v3
//     … ld r30,-24(r1) ; ld r31,-16(r1) ; blr
//
// **The rule this rung had to establish** — `CODEGEN_FRAMED_CALLS.md` §6 lists
// "which values become callee-saved, and in what order" as the half that refused
// to yield a rule — is: a formal read by any call *after the first* is copied
// into a callee-saved register, and the file is allocated **descending from r31
// in PARAMETER order**. `use_order` below is the capture that refutes first-use
// order: it emits the *same* two save moves as `two_saved` and differs only in
// which one each later call reads back.
//
// The `/Gy` label stride is **5**, unchanged by the saved registers — measured on
// a TU of two Class B functions ($M2571/$M2572/$T2573 then $M2576/$M2577/$T2578),
// which is worth stating because §4.4 found the `__savegprlr_N` class does move
// it to 7.
//
// The neighbours that must keep refusing are in `mvp_call_seq_b_neg.cpp`.

extern void v0();
extern void v1(int);
extern void v2(int);
extern void v3(int);
extern void g2(int, int);
extern int i1(int);

// --- one saved GPR --------------------------------------------------------
void one_saved(int a, int b) { v1(a); v2(b); }
// A formal read by the FIRST call as well is still saved — `mr r31,r3` before a
// `bl` whose argument is already in r3.
void live_twice(int a) { v1(a); v2(a); }
// One saved value, read by three later calls.
void live_thrice(int a, int b) { v1(a); v2(b); v3(b); v1(b); }

// --- two saved GPRs -------------------------------------------------------
void two_saved(int a, int b, int c) { v1(a); v2(b); v3(c); }
// THE REFUTATION ROW: `c` is used before `b`, and still takes r30.
void use_order(int a, int b, int c) { v1(a); v2(c); v3(b); }
// The first call's argument is itself live, so a=r31 and b=r30.
void first_arg_live(int a, int b) { v1(a); v2(a); v3(b); }

// --- a later call marshalling several arguments OUT of the saved file ------
// Sources are r31/r30 and destinations r3/r4, two disjoint sets, so there is no
// cycle to break: the moves go out highest-destination-first (`mr r4,r31` then
// `mr r3,r30`).
void multi_swap(int a, int b) { v0(); g2(b, a); }
void multi_id(int a, int b) { v0(); g2(a, b); }
void multi_then_more(int a, int b) { v0(); g2(b, a); v1(a); }

// --- the first call marshalling its OWN argument beside the saves ---------
// A save whose source register the marshalling overwrites is hoisted in front of
// it; one whose source it leaves alone trails it. `hoist_r3` is the first half
// (`mr r31,r3 ; mr r3,r4 ; bl ?v1`) and `hoist_split` has both halves at once.
void hoist_r3(int a, int b) { v1(b); v2(a); }
void hoist_split(int a, int b, int c) { v1(b); v2(a); v3(c); }
void lit_first(int a, int b) { v1(7); v2(a); v3(b); }
// The IDENTITY permutation is not marshalling — it writes nothing, so both saves
// trail it.
void id_perm(int a, int b, int c) { g2(a, b); v1(a); v2(c); }

// --- literals and tails beside the saves ----------------------------------
void lit_between(int a, int b) { v1(a); v2(5); v3(b); }
int tail_lit(int a, int b) { v1(a); v2(b); return 7; }
int tail_call_k(int a, int b) { v1(a); return i1(b) + 1; }

// --- an FP formal consumes an argument SLOT but no GPR --------------------
// The two effects cancel: the GPR argument register is still the formal's index.
// Witnessed here in a framed Class B prologue (`mr r31,r5` for the third formal),
// not only in the leaf/tail-call probes that first measured it.
void fp_first(float x, int a, int b) { v0(); v1(a); v2(b); }
void fp_middle(int a, float x, int b) { v1(a); v2(b); }
