// w-fltret — the `msc-value-*` clause probe.
//
// `mcall_tail`'s route RE-ARMS `tail-void-body-does-not-end-at-the-call` after a
// failed sequence attempt (w-mcall PREREG §2.2, deliberate), so a tag written
// inside `eat_member_value_call` is buried whenever the body's FIRST statement
// is the member call. It survives on the other route — a FREE call first, then
// the member value tail — which is what every cell here is.
struct S {
    void a();
    int get();
    float f();
    double d();
    long long wide();
};
void gv();

// msc-value-fp-result-converted (narrowing: a real `frsp 1,1`)
float p2_narrow(S *s) {
    gv();
    return (float)s->d();
}

// msc-value-fp-result-converted (widening: nothing emitted, refused anyway)
double p2_widen(S *s) {
    gv();
    return s->f();
}

// msc-value-fp-postop
float p2_fp_postop(S *s) {
    gv();
    return s->f() + 1.0f;
}

// msc-value-result-type (an 8-byte integer result)
long long p2_wide(S *s) {
    gv();
    return s->wide();
}

// the POSITIVE control on this route: it must be in class
float p2_ok(S *s) {
    gv();
    return s->f();
}

// the integer positive control on this route
int p2_ok_int(S *s) {
    gv();
    return s->get();
}
