// **Negative** — an FP store leaf sharing a translation unit with a FRAMED
// function. Every function here is individually in class, the TU as a whole must
// refuse, and it must never mismatch.
//
// ## The twelfth live wrong-bytes emit, and why only a merge could find it
//
// A framed function's `$M`/`$T` compiler labels are numbered from a counter that
// **every** function in the TU consumes, so a function ahead of it with the
// wrong stride makes six wrong bytes in an obj that still links.
// `IlFunction::label_slots` gave the FP **store** leaf 1, and c2 gives it 2.
// MEASURED as the three-way capture that separates the two rules (`/Ox /GS- /c`,
// one leaf ahead of one framed function, reading the framed function's labels):
//
//   void lead(S* s, int v)      { s->i = v; }     $M2558 $M2559 $T2560
//   void lead(S* s, float v)    { s->f = v; }     $M2559 $M2560 $T2561
//   float lead(float a, float b){ return a * b; } $M2559 $M2560 $T2561
//
// The stride goes with the **register file**, not with the body shape: anything
// that touches floating point consumes 2.
//
// This is the eleventh mis-emit's own field, one consumer later. `is_float` was
// split into `touches_floating_point` (for `_fltused`) and `label_slots` was
// left reading `float_leaf` — so the FP store leaf got the marker right and the
// stride wrong. `docs/GAPS.md` §6 instance #2 exactly: *fixed in the one shape
// where the bug had been found.*
//
// **Neither side's corpus could contain it.** The counter has an observable
// effect only when a framed function follows, and until Class A many-calls
// (#35 step 2) landed there was no framed shape that could share an in-class TU
// with an FP store: the FP rung's fixtures have no framed function and the
// framed rung's have no floating point. The pair exists only in the merge, and
// it emitted `$M2564/$M2563/$T2565` against the reference's
// `$M2565/$M2564/$T2566`.
//
// ## Why this file REFUSES rather than matching
//
// With the stride told truthfully, `bundle.rs`'s TU-level gate refuses — it
// admits a framed function only when every other function in the TU has a stride
// of exactly 1, because `coff::plan_labels` advances by 1 for every non-framed
// function. Teaching the planner a per-function stride would admit this pair and
// is a change to the framed side's label model, not to the FP classes; it is
// ranked in `docs/CODEGEN_FP_ARGS.md` §5. Until then this is an honest refusal,
// and this file is what keeps it from silently becoming an emit again.
//
// `c2rs census` reports every function here **in class** — that is the
// per-function verdict, and it is correct. The refusal is at the translation
// unit, which is where the label counter lives.

struct S { int i; float f; double d; };
void g1();
void g2();

// The FP half: a store leaf at each width, stride 2.
void fp_store_f(S* s, float v)  { s->f = v; }
void fp_store_d(S* s, double v) { s->d = v; }

// The framed half: Class A many-calls, which consumes 4 (packed) / 5 (`/Gy`).
void seq2()                     { g1(); g2(); }

// An integer store leaf, stride 1 — the control. It is what makes this file a
// test of the FP stride rather than of "any leaf beside a framed function".
void int_store(S* s, int v)     { s->i = v; }
