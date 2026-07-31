// WCL negative neighbours — one case per refusal row, each with its own census
// key. `c2rs census` must report **0/N**: every one of these is a chain with an
// argument on a later link, i.e. this rung's own production, stopped by a gate
// this rung declares rather than by a decode failure.

struct I {
    int gi();
    int ga(int);
    int gb2(int, int);
    int gb3(int, int, int);
    int g8(int, int, int, int, int, int, int, int);
    int gf(float);
    long long gl(long long);
};

struct O {
    I* Next();
    I* NextA(int);
    I* NextB(int, int);
    O* Self();
};

extern int g_i;

// ---- the argument's own shape ------------------------------------------------

// COMPUTED — `addi r4,r31,1`, the operand stream rebased onto the callee-saved
// register rather than computed by the leaf selector. A second lowering of
// `select_text`, not a use of it.  `mcall-chain-link-arg-computed`
int n_computed(O* p, int k) { return p->Next()->ga(k + 1); }
int n_computed2(O* p, int j, int k) { return p->Next()->gb2(j, k + 1); }

// NOT A FORMAL — a global has no register to have been saved into; c2 loads it,
// and nothing in the save plan put it anywhere.  `mcall-chain-link-arg-nonformal`
int n_global(O* p) { return p->Next()->ga(g_i); }

// PAST THE REGISTER FILE — slot 0 is the receiver, so eight explicit arguments
// reach slot 8 and the last is stack-homed: a store, not a move. Spelled with
// literals so the *formals* list stays inside its own eight and this row grades
// the argument bound rather than `callseq-over-eight-formals`.
// `mcall-chain-link-arg-overflow`
int n_wide_slots(O* p) { return p->Next()->g8(1, 2, 3, 4, 5, 6, 7, 8); }
// …and seven is the last one that fits, which is the accepting side of the same
// bound — it lives in the positive fixture's `c8` neighbourhood, not here.

// A LITERAL WIDER THAN THE `addi` IMMEDIATE — `lis`+`ori`, two words, not one.
// `mcall-chain-link-arg-lit-wide`
int n_wide_lit(O* p) { return p->Next()->ga(70000); }

// ---- the frame class ---------------------------------------------------------

// THREE live formals is the `__savegprlr_29` helper class: the prologue collapses
// to a `bl`, the epilogue becomes a tail branch, and there is a second REL24
// site.  `callseq-three-plus-saved`
int n_three(O* p, int i, int j, int k) { return p->Next()->gb3(i, j, k); }

// NINE formals — the receiver itself is stack-homed and reading it is `lwz`.
// `callseq-over-eight-formals`
int n_nine(int a, int b, int c, int d, int e, int f, int g, int h, O* p) {
    return p->Next()->ga(a);
}

// ---- the innermost call, beside a save ----------------------------------------

// A NON-IDENTITY PERMUTATION at the innermost call while a formal is saved: c2
// breaks the cycle through the callee-saved register instead of r11, which is a
// different algorithm and not the measured interleave.
// `callseq-saved-with-first-call-setup`
int n_perm(O* p, int j, int k) { return p->NextB(k, j)->ga(k); }

// A COMPUTED innermost argument beside a save — under `/Ox` the intermediate
// goes to a fresh descending register, which is the file the saves live in.
// `callseq-saved-with-first-call-setup`
int n_inner_computed(O* p, int j, int k) { return p->NextA(j + 1)->ga(k); }

// ---- the value class ----------------------------------------------------------

// A `float` argument is the FP register file, not r4 — a different marshalling
// entirely, and the TU gains `_fltused`.
float n_float(O* p, float x) { return (float)p->Next()->gf(x); }

// A `long long` argument is a GPR **pair** on this ABI.
long long n_wide_value(O* p, long long x) { return p->Next()->gl(x); }

// ---- what comes after the chain -----------------------------------------------

// A post-op on the result is a `-then-` sibling and keeps its own key; it is the
// tail that changes, not the argument.
int n_postop(O* p, int k) { return p->Next()->ga(k) + 1; }
int n_second(O* p, int k) { return p->Next()->ga(k) + p->Next()->ga(k); }
