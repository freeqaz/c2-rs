// **W-VSNPRNC — the FENCE around the widened `guard_chain_shared_tail`.** Like
// its positive twin this file declares no profile: at `/Ox` the class refuses on
// the mode gate and every cell is `NotImplemented`; the grading happens in
// `scripts/gate.sh`'s `/O1` mode lanes.
//
// Four of these five cells are widths and arities the SHIPPED class let through
// to a **wrong word** or would now. `n0` is the one that was a live
// `Port=Mismatch` in a different guise: the shipped reader refused only a
// width-4 store, so `long long` reached an emitter that had exactly two store
// instructions and neither was `std`.
//
// STRUCTURAL BLIND SPOT: three guards, one guard order, external callees, a
// function's address in slot 0 — held fixed in every cell, exactly as in the
// positive twin, so this file fences the arity and the store and nothing else.

typedef unsigned int usz;
typedef int (*outfn_t)(void);

int *lasterr_n(void);
void report_n(void);
int outfn_n(void);
int helper8_n(outfn_t, char *, usz, usz, void *, void *, void *, void *, void *);
int helper5_i(outfn_t, int *, usz, usz, void *, void *);
int helper5_q(outfn_t, long long *, usz, usz, void *, void *);
int helper5_f(outfn_t, float *, usz, usz, void *, void *);
int helper5_u(outfn_t, unsigned *, usz, usz, void *, void *);

// n0 — a **`long long`** store. c2 emits an eight-byte store; the shipped class
// emitted `sth` and was a `Port=Mismatch` for it. Refused now, because a gap is
// not traded for a wrong word.
int n0(long long *buffer, usz a, usz b, void *c, void *d) {
    int result;
    if (b == 0 || buffer == 0 || a == 0) { *lasterr_n() = 0x16; report_n(); return -1; }
    result = helper5_q(outfn_n, buffer, a, b, c, d);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *lasterr_n() = 0x22; report_n(); return -1;
}

// n1 — a **word** store. The one width the shipped clause refused by name, and
// it must keep refusing under the width-carrying clause that replaced it.
int n1(int *buffer, usz a, usz b, void *c, void *d) {
    int result;
    if (b == 0 || buffer == 0 || a == 0) { *lasterr_n() = 0x16; report_n(); return -1; }
    result = helper5_i(outfn_n, buffer, a, b, c, d);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *lasterr_n() = 0x22; report_n(); return -1;
}

// n2 — an **unsigned** word. `is_int4_type` admits both signs deliberately, so
// this is the second half of n1 and not a duplicate of it.
int n2(unsigned *buffer, usz a, usz b, void *c, void *d) {
    int result;
    if (b == 0 || buffer == 0 || a == 0) { *lasterr_n() = 0x16; report_n(); return -1; }
    result = helper5_u(outfn_n, buffer, a, b, c, d);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *lasterr_n() = 0x22; report_n(); return -1;
}

// n3 — a **float** store: a different instruction entirely, and a class that
// read only the size would take it.
int n3(float *buffer, usz a, usz b, void *c, void *d) {
    int result;
    if (b == 0 || buffer == 0 || a == 0) { *lasterr_n() = 0x16; report_n(); return -1; }
    result = helper5_f(outfn_n, buffer, a, b, c, d);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *lasterr_n() = 0x22; report_n(); return -1;
}

// n4 — **EIGHT formals**, so the call takes nine arguments and the ninth does
// not fit the argument registers. This is the arity ceiling, and it is a
// WITNESS rather than a guess: `work/w-vsnprnc/probe/n8.obj` shows c2 growing
// the frame to 112, spilling `r10` to `84(r1)` at the call site and hoisting
// nothing at all — a different shape, not one more rotate step.
int n4(char *buffer, usz a, usz b, void *c, void *d, void *e, void *f, void *g) {
    int result;
    if (b == 0 || buffer == 0 || a == 0) { *lasterr_n() = 0x16; report_n(); return -1; }
    result = helper8_n(outfn_n, buffer, a, b, c, d, e, f, g);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *lasterr_n() = 0x22; report_n(); return -1;
}
