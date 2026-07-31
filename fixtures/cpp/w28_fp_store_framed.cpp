// **An FP store leaf sharing a translation unit with a FRAMED function** — the
// pair that neither the FP rung's corpus nor the framed rung's could contain,
// and that mis-emit #12 was found in. It is byte-exact now; this file was
// `w28_fp_store_framed_neg.cpp` and its own comment predicted the promotion.
//
// ## The twelfth live wrong-bytes emit
//
// A framed function's `$M`/`$T` compiler labels are numbered from a counter that
// **every** function in the TU consumes, so a function ahead of it with the
// wrong stride makes six wrong bytes in an obj that still links.
// `IlFunction::label_slots` gave the FP **store** leaf 1 where c2 gives it a
// TU that is one wider — `is_float` had been split into
// `touches_floating_point` (for `_fltused`) and `label_slots` was left reading
// `float_leaf`, so the store leaf got the marker right and the stride wrong.
// `docs/GAPS.md` §6 instance #2 exactly: *fixed in the one shape where the bug
// had been found.*
//
// ## …and the rule that repaired it was itself wrong from two FP functions on
//
// The repair read **"anything that touches floating point consumes 2 — the
// stride goes with the register file"**. That fits the capture it was taken from
// (one leaf ahead of one framed function) and predicts 4 slots for two FP
// functions where c2 gives 3, and 6 for three where c2 gives 4.
//
// Re-measured **seed-free**, as the difference between two framed functions'
// labels in one TU — so the `.gl` seed cancels and nothing depends on matching
// mangled-name lengths (`/Ox /GS- /c`; every row is `+1` under `/Gy`, and the
// `/Gy` pre-pass is exactly `3 x funcs.len()` on all eleven):
//
//   fr1;                      fr2    delta 4    leaf slots 0
//   fr1; int_store;           fr2    delta 5    leaf slots 1
//   fr1; fp_store;            fr2    delta 6    leaf slots 2
//   fr1; fp_store fp_store;   fr2    delta 7    leaf slots 3   <- not 4
//   fr1; int_store fp_store;  fr2    delta 7    leaf slots 3
//   fr1; fp_store int_store;  fr2    delta 7    leaf slots 3
//   fr1; int_store int_store; fr2    delta 6    leaf slots 2
//   fr1; fp_arith;            fr2    delta 6    leaf slots 2
//   fr1; fp_arith fp_arith;   fr2    delta 7    leaf slots 3
//   fr1; fp_store fp_arith;   fr2    delta 7    leaf slots 3
//   fr1; fp_store x3;         fr2    delta 8    leaf slots 4   <- not 6
//
// > **Every function consumes 1 slot, plus ONE extra for the translation unit if
// > any function touches floating point.**
//
// The extra slot is `_fltused`. The two facts `is_float` carries — where
// `_fltused` goes and where the extra slot goes — are now the *same* fact rather
// than two readers of one field, which is what stopped the third instance of this
// bug.
//
// **This comment used to add a generalization, and it is REFUTED**
// (`docs/LABEL_COUNTER.md` §2.1): *the extra slot is the one TU-level external an
// FP-touching function introduces, so the rule is one slot per TU-level external,
// the same rule `docs/CODEGEN_FRAMED_CALLS.md` §4.4 measured as two for the
// `__savegprlr_N`/`__restgprlr_N` pair.* Every number above survives — `_fltused`
// is +1 and the GPR helper pair is +2 — but the reason is wrong in **both**
// directions: a newly pooled FP constant costs **+2** and mints no external at
// all, a string literal costs **0** while minting one, and a materialised signed
// relational costs **+2** and mints nothing. The reason is what licenses the next
// class, so widen the counter from §1.1's measured surcharge table and from
// nothing else — this rule would have admitted a pooled constant at +0.
//
// A per-function method cannot express a per-TU quantity, and that is the
// structural reason the wrong rule could not be stated correctly where it lived:
// the `+1` is applied by `c2_core::coff::plan_labels`, which has the whole
// function list.
//
// The FP **arithmetic** leaf used to be the negative half of this pair, on the
// same over-charge — `label_slots` answered `None` for every float leaf, so one
// predicate refused every (FP leaf, framed family) pair there is. It is admitted
// now (`leaf-float-led` = `leaf-double-led` = **1**) and both widths are rows
// below. What still refuses is the leaf that POOLS A CONSTANT, whose surcharge is
// +2 per *newly* pooled `(bits,width)` — a per-TU question again — and whose
// `.rdata`-beside-`.pdata` section order no capture settles:
// `w28_fp_store_framed_neg.cpp` and `wunw_float_neg.cpp`.

struct S { int i; float f; double d; };
void g1();
void g2();
void v1(int);
void v2(int);

// The FP half: a store leaf at each width. Two of them, which is the row the
// old rule got wrong — it wanted 4 slots for this pair and c2 gives 3.
void fp_store_f(S* s, float v)  { s->f = v; }
void fp_store_d(S* s, double v) { s->d = v; }

// The framed half: Class A many-calls, which consumes 4 (packed) / 5 (`/Gy`)
// and is the only reason the counter is observable at all.
void seq2()                     { g1(); g2(); }

// …and Class B, so the pair is graded against a framed function that also saves
// callee-saved registers (`docs/ROADMAP.md` §6l).
void seqB(int a, int b)         { v1(a); v2(b); }

// An integer store leaf, stride 1 — the control. It is what makes this file a
// test of the FP stride rather than of "any leaf beside a framed function".
void int_store(S* s, int v)     { s->i = v; }

// The FP **arithmetic** leaf at both widths — rows 8 and 9 of the table above,
// and the half this file's negative used to hold. Constant-free, so `leaf-float-led`
// / `leaf-double-led` = 1 each, and placed AFTER the framed functions so the pair
// is graded in both orders across this file and `wunw_float_beside_framed.cpp`.
float  fp_arith_f(float a, float b)   { return a * b; }
double fp_arith_d(double a, double b) { return a * b; }
