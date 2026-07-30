// **Positive** — where `_fltused` goes when the floating point is a STORE.
// Every function here must emit, and the whole obj must be byte-exact.
//
// A translation unit that touches floating point carries an undefined external
// `_fltused`, the CRT's float-support hook, placed immediately after the **first
// FP-touching function's complete symbol group** and before the next function's
// section symbol (`docs/OBJ_GY_SHAPES.md` §1, `docs/CODEGEN_FP_ARGS.md` §4).
//
// ## Why this file exists
//
// `coff::Function::is_float` carried two facts at once — "this body does FP
// arithmetic, so its label stride is 2" and "this TU needs `_fltused`" — and
// every function that had ever set it satisfied both, because the only FP class
// the port had was the W13 arithmetic leaf. The FP **store** leaf satisfies only
// the second: it is a store leaf, stride 1, and it needs the marker. The port
// emitted every one of `w28_fp_store.cpp`'s fourteen objs **one symbol short**,
// `Port=Mismatch @ offset 12` — the COFF header's `NumberOfSymbols`.
//
// `GAPS.md` §6's recurring shape in the obj shell rather than in an instruction,
// and it was invisible from inside the old corpus for the usual reason: an
// all-FP-arithmetic TU and an FP-touching TU were the same set of files.
//
// ## What this file discriminates that `w28_fp_store.cpp` cannot
//
// That file is *all* FP stores, so the marker lands after function 1 whether the
// rule is "the first FP-touching function" or "every FP function" or "the first
// function, period". Here the FP is **not** first, is interleaved with integer
// functions, and appears in two different FP classes:
//
//   a_int    integer leaf         — the marker must NOT precede this
//   b_fps    FP store leaf        — the marker goes HERE, after this group
//   c_int    integer leaf
//   d_fps    a second FP store    — and NOT again here
//   e_leaf   a W13 arithmetic leaf — nor here
//
// Captured verbatim: `?a_int@…` at symbol 13, `?b_fps@…` at 16, `_fltused` at
// **17**, then `.text`/`?c_int@…` at 18/20. A rule keyed on the arithmetic leaf
// alone puts it after `e_leaf` instead, twelve symbols late.

struct S { int i; float f; };

int   a_int (int x)             { return x + 1; }
void  b_fps (S* s, float v)     { s->f = v; }
int   c_int (int x)             { return x + 2; }
void  d_fps (S* s, float v)     { s->f = v; }
float e_leaf(float x, float y)  { return x * y; }
