// w-fltret — the ladder, as three probe bodies read off real c2 BEFORE a line
// of `crates/` was written.
//
//   L1  the MEMBER value tail, INTEGER   `s->a(); return s->get();`
//   L2  the FREE   value tail, FLOAT     `gv(); return gf();`
//   L3  the MEMBER value tail, FLOAT     `a(); return f();`   <- Timer::SplitMs
//
// L3's shape is `float Timer::SplitMs() { Split(); return Ms(); }`
// (`src/system/os/Timer.h:137`), 434 emitted in 434 TUs on the workload — the
// body this lane's whole population is.

struct S {
    int m;
    void a();
    int get();
    float f();
    double d();
    int SplitMs_int();
};

void gv();
float gf();
double gd();
int gi();

// -- controls the rest are read against ------------------------------------
// C1: the free INT value tail, already in class since #35 step 2.
int c1_free_int() {
    gv();
    return gi();
}

// -- L1: the member INT value tail ------------------------------------------
int l1_member_int(S *s) {
    s->a();
    return s->get();
}

// L1b: the same with the implicit `this` — the workload's spelling.
int S::SplitMs_int() {
    a();
    return get();
}

// -- L2: the free FP value tail ----------------------------------------------
float l2_free_float() {
    gv();
    return gf();
}

double l2_free_double() {
    gv();
    return gd();
}

// -- L3: Timer::SplitMs, transcribed -----------------------------------------
float l3_member_float(S *s) {
    s->a();
    return s->f();
}

double l3_member_double(S *s) {
    s->a();
    return s->d();
}

// -- the conversion cells D6 refuses -----------------------------------------
// The callee returns `double` and the body returns `float`: an `frsp`.
float l4_narrowing(S *s) {
    s->a();
    return (float)s->d();
}

// The callee returns `float` and the body returns `double`: free at the
// argument boundary, UNGRADED at the return boundary.
double l4_widening(S *s) {
    s->a();
    return s->f();
}

// -- the post-op cell --------------------------------------------------------
int l5_postop(S *s) {
    s->a();
    return s->get() + 3;
}
