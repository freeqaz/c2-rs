// W13 — the fail-closed neighbours. Every function here is *tree-shaped source*
// that c2 does NOT lower as the corresponding tree of FP instructions, or that
// drags in obj structure (an .rdata COMDAT, a non-volatile save) the W13a leaf
// model has no model for. A naive "one FP instruction per source operator"
// selector produces plausible, wrong bytes for every one of them.
//
// Grouped by the rule they violate; see docs/CODEGEN_W13_FLOAT.md §6.

// --- N1: multiply-add contraction. A `*` feeding a `+`/`-` is ALWAYS fused. ---
float n_fma(float a, float b, float c)               { return a * b + c; }
float n_fma_comm(float a, float b, float c)          { return c + a * b; }
float n_fms(float a, float b, float c)               { return a * b - c; }
float n_fnms(float a, float b, float c)              { return c - a * b; }
float n_fma2(float a, float b, float c, float d)     { return a * b + c * d; }
double n_dfma2(double a, double b, double c, double d) { return a * b + c * d; }
float n_fma_tree(float a, float b, float c, float d) { return (a + b) * c + d; }

// --- N2: a subtracted product re-ranks the tree (the subtrahend is emitted
// first, the minuend is folded into the fmsub). ---
float n_rank(float a, float b, float c, float d) {
  return ((a + b) * (c + d)) - ((a + c) * (b + d));
}

// --- N3: FP constants (W13b) — an .rdata COMDAT, a REFHI/REFLO pair and a
// GPR address temp. ---
float n_k_add(float a)                     { return a + 1.0f; }
double n_k_dadd(double a)                  { return a + 1.0; }
float n_k_ret()                            { return 1.5f; }
float n_k_two(float a)                     { return (a + 1.0f) * (a + 2.0f); }

// --- N4: constants c2 SYNTHESIZES. `x + x` strength-reduces to `x * 2.0f` and
// `x / k` to `x * (float)(1/k)`, so an expression with no literal in the source
// still needs an .rdata constant. ---
float n_self_add(float a)                  { return a + a; }
float n_div_k(float a)                     { return a / 3.0f; }
double n_ddiv_k(double a)                  { return a / 3.0; }

// --- N5: identity folds. `+ 0.0f` and `* 1.0f` vanish entirely. ---
float n_plus_zero(float a)                 { return a + 0.0f; }
float n_times_one(float a)                 { return a * 1.0f; }

// --- N6: integer <-> FP conversion — a stack round-trip through the red zone
// (`std`/`lfd`, `fctiwz`/`stfd`/`lwz`) plus `fcfid`. ---
float n_i2f(int a)                         { return (float)a; }
int   n_f2i(float a)                       { return (int)a; }

// --- N7: more live FP values than the 14-register volatile pool — c2 saves
// f31/f30/f29 into the red zone. ---
float n_spill(float a, float b, float c, float d, float e, float f, float g,
              float h) {
  return ((a + b) * (c + d)) + ((e + f) * (g + h)) + ((a + c) * (b + d))
       + ((e + g) * (f + h)) + ((a + d) * (b + c)) + ((e + h) * (f + g))
       + ((a + e) * (b + f)) + ((c + g) * (d + h));
}
