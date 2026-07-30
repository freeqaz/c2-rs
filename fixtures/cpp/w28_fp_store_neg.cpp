// **Negative** — the floating-point store leaf's boundary. Every function here
// MUST be out of class, and the file must never mismatch.
//
// `docs/CODEGEN_FP_ARGS.md` §3. The positive class is
// `fixtures/cpp/w28_fp_store.cpp`: one `stfs`/`stfd` and a `blr`. Each function
// below is one token away from it and emits **more than one instruction**, so
// admitting any of them is wrong bytes rather than a coverage gap.
//
// ## 1. A conversion on the stored value — and the asymmetry is c2's, not C's
//
// The narrowing direction pays a real instruction through the FP scratch
// register, and the widening one pays nothing at all:
//
//   void s_narrow(S* s, double v) { s->f = v; }
//       fc000818  frsp f0,f1
//       d0030004  stfs f0,4(r3)          <- and the store is from f0, not f1
//   void s_widen (S* s, float v)  { s->d = v; }
//       d8230008  stfd f1,8(r3)          <- nothing added
//
// Both are refused. The narrowing one obviously must be; the *free* one is the
// interesting refusal, and it is a conservatism with a stated reason rather than
// an oversight. Telling the two apart means deciding a direction from two type
// triples, the widening case has been captured at exactly one offset and one
// width pair, and the IL spells both with the same `2C <TYPE> 00` that is free
// between the width-4 integer classes — the shape `GAPS.md` §6 keeps recording:
// one field carrying two facts, indistinguishable until something separates
// them. `frsp`'s own use of f0 is the separator here, and it says the two
// directions are different constructs.
//
// ## 2. A pooled floating-point literal
//
//   void s_lit(S* s) { s->f = 1.5f; }
//       3d600000  lis  r11,0        REFHI __real@3fc00000 + PAIR
//       c00b0000  lfs  f0,0(r11)    REFLO __real@3fc00000 + PAIR
//       d0030004  stfs f0,4(r3)
//
// Three instructions, an `.rdata` COMDAT and four relocations — the W13b constant
// machinery, which `codegen::function_gate` refuses under function-level linking
// (`/Gy`, which `/O1` implies) in any case. An integer literal in the same
// position IS admitted (`void f(S* s){ s->a = 7; }` is `li r11,7 ; stw r11`), so
// this is the one place the FP and GPR store paths differ in what a *literal*
// costs, and it is why the FP path refuses one rather than sharing the integer
// path's `emit_load_imm`.
//
// ## 3. A computed value
//
// The GPR path already refuses this (the computation lands in the scratch
// register first) and the FP path must too, for a sharper reason: an FP chain in
// a store's value position would have to allocate from the same rotating pool
// `float_leaf_text` owns, and `f0` is both that pool's first slot and `frsp`'s
// destination above.
//
// Freestanding, include-free, leaf-only. Compiled by `c2rs bench` and by every
// `scripts/mode_lane.sh` lane.

struct S { int i; float f; double d; float arr[4]; char c; float g; };

// 1. conversions, both directions
void s_narrow(S* s, double v)         { s->f = v; }
void s_widen (S* s, float v)          { s->d = v; }
void s_narrow2(S* s, double v)        { s->g = v; }
void s_from_int(S* s, int v)          { s->f = (float)v; }
void s_to_int(S* s, float v)          { s->i = (int)v; }

// 2. pooled literals
void s_lit(S* s)                      { s->f = 1.5f; }
void s_lit0(S* s)                     { s->f = 0.0f; }
void s_litd(S* s)                     { s->d = 2.5; }

// 3. computed values
void s_add(S* s, float u, float v)    { s->f = u + v; }
void s_mul(S* s, float u, float v)    { s->f = u * v; }
void s_neg(S* s, float v)             { s->f = -v; }
