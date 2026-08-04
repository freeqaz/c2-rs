// **W11's boundary — every one of these is REFUSED, and every refusal is a
// measurement rather than a taste.** Censuses 0/6.
//
// The companion to `w11_early_return.cpp`. A class boundary that is only
// asserted in a doc comment is a boundary the corpus cannot check, and the two
// defects this project has paid for twice (#232, X-c) were both a fixture family
// that was exhaustive on the axis it varied and silent on the one it held fixed.
//
// Each row below was compiled with the real toolchain at `/O1` **and** `/Ox`
// (`work/w-conv/p/probe1.cpp`, `probe2.cpp`, `probe3.cpp`) and emits something
// this rung does not model. Emitting the in-class shape for any of them would be
// a wrong-bytes obj that still links — which is strictly worse than a gap.

void v0();
void v1();

// ---- 1. TWO EXITS WITH THE SAME VALUE — c2 MERGES THE ARMS -----------------
//
// Not "a second copy of the arm". c2 emits **one** arm and branches
// **backwards** into it with the sense inverted:
//
//     0018  48000014  b   +20        the first arm -> the epilogue
//     001c  2f040000  cmpwi cr6,r4,0
//     0020  409afff4  bf  26,-12     <- BACKWARD, into the first arm
//
// and it costs a **sixth** compiler-label slot where every in-class cell costs
// five, so the wrong bytes would be in the symbol table too. Holds at `/Ox`.
int m2(int a, int b) { if (a != 0) return 5; if (b != 0) return 5; v0(); return 0; }

// ---- 2. …including a collision with the SEQUENCE's own literal -------------
//
// The same variable — *do two exits produce the same value?* — read at the other
// end. Here the arm vanishes entirely and the branch skips the call straight to
// the shared `li r3,0`: 44 B against the 52 B of the distinct-literal form. One
// refusal, two witnesses, and this row is why the check includes the tail.
int m0(int a) { if (a != 0) return 0; v0(); return 0; }

// ---- 3. A GUARD PLACED AFTER A CALL ---------------------------------------
//
// `a` now has to survive the `bl`, so c2 parks it in **r31** — and then folds
// the whole `if` branchlessly rather than branching at all:
//
//     mflr/stw/std r31/stwu ; mr r31,r3 ; bl ?v0
//     subfic r11,r31,0 ; li r10,5 ; subfe r11,r11,r11 ; and r3,r11,r10
//
// Callee-saved plus fold band 1. Both refused elsewhere, and this class has no
// entry block at all.
int e7(int a) { v0(); if (a != 0) return 5; return 0; }

// ---- 4. AN ARM CONTAINING A CALL ------------------------------------------
//
// c2 emits it (`bt 26,+16 ; bl ?v1 ; li r3,5 ; b -> epi`), so this is a
// *narrowing*, not an impossibility. One cell per mode, and W10's three-witness
// rule says an unmeasured arm shape is refused rather than fitted.
int ac(int a) { if (a != 0) { v1(); return 5; } v0(); return 0; }

// ---- 5. A W10 GUARDED CALL IN THE SAME BODY -------------------------------
//
// c2 composes the two productions happily. The emitter could too — the guarded
// call's `bc` is just another branch — but two block plans in one body is a
// second production and nothing has graded their interleaving. Refused in the IL
// parser AND backstopped in `call_seq_text`, because a silent interleave is a
// layout nobody measured.
int x6(int a, int b) { if (a != 0) return 5; if (b != 0) v0(); v1(); return 0; }

// ---- 6. VOID GUARDS OVER ONE TRAILING CALL — NOT A FRAMED BODY AT ALL ------
//
// The trap W10 hit one production over, and the reason this rung hands the rest
// of the body to the SHARED sequence loop instead of a copy. c2 emits
//
//     cmpwi cr6,r3,0 ; bnelr cr6 ; cmpwi cr6,r4,0 ; bnelr cr6
//     cmpwi cr6,r5,0 ; bnelr cr6 ; b ?v0 ; blr
//
// — three `bclr` folds and a tail branch, **32 B with no `.pdata`**. Emitting
// the framed body here would drop three branches into an obj that still links.
void w3(int a, int b, int c) {
    if (a != 0) return;
    if (b != 0) return;
    if (c != 0) return;
    v0();
}
