// w-inlfence — THE FENCE: a callee this TU defines and the port has NO MODEL of.
//
// c2 may **inline** a callee whose body it has, and the port cannot. The
// measured cost of getting this wrong is on the board three times:
//
//   * `int f(int); int use(int a){return f(a);} int f(int a){return a+1;}` —
//     c2's `.text` is *two copies* of `addi r3,r3,1 ; blr` with **no
//     relocations**; the port's `b ?f` mismatched at file offset 8.
//   * `?SplitMs@Timer@@QAAMXZ`, **434 emitted functions in 434 workload TUs** —
//     reference body **31 words** against the port's 13, because
//     `Timer::Split()` and `Timer::Ms()` are `inline` members in the same header
//     (`docs/rungs/2026-08-09-w-fltret.md` §6, board #2082).
//   * `crates/c2-harness/tests/dead_temp_elision.rs`'s **m02**, where a standing
//     test had pinned the resulting wrong body as the expected outcome since
//     `w-inl0` (board #2224).
//
// # The fence yields to what the port ALREADY models, and the controls are here
//
// The port is not silent about every inline. **Mechanism E** (`c2_core::elide`)
// says a call to a callee that emits nothing costs no branch — **1,877 of 1,877
// byte-exact** on the workload — and **mechanism I** (`c2_core::splice`) says a
// call to a callee the port can LOWER is replaced by that callee's own body —
// **723 of 723 byte-exact**. Refusing either would refuse bodies the port
// provably gets right, so the fence fires only on a callee that is **neither**.
//
// That makes the cells' callees load-bearing in a way two earlier drafts of this
// file got wrong: the first gave N1–N3 empty callees and graded mechanism E's
// exemption, the second gave them in-class callees and graded mechanism I's.
// **Every N cell's callee below is a counted loop** — non-empty and out of class
// — which is also the shape of the five surviving `bl`s in
// `src/keygen_xbox.cpp`. The X cells are the exemptions, and they must stay in
// class.
//
//   N1  a void TAIL CALL to a function defined below it — the offset-8
//       mismatch above, in its original form.
//   N2  the STATEMENT form of the member-call sequence (w-mcall's `CallSeq`)
//       with both member callees defined here.
//   N3  **the 444's own class** — the float VALUE TAIL (`SeqTail::CallValueFp`)
//       written the way `src/system/os/Timer.h:137` writes it, with `Split()`
//       and `Ms()` defined in the same TU. w-fltret's F7 is this cell with the
//       callees declared and not defined, and it is a whole-TU `match`.
//   N4  a callee far OVER the size ceiling. `docs/whitebox/
//       WB_INLINE_FINDINGS.md` F1 measures the static ceiling at `(300,308]`
//       bytes of emitted `.text` at favour-size, so c2 declines *this* one — and
//       the port refuses anyway, because the ceiling is a **bracket** and asking
//       "how big is this callee" means lowering it first. Declined as **D3**.
//   N5  a LOOP-BODIED callee over the loop class's own `(56,80]` boundary
//       (F9, 56 cells) — the tighter ceiling, refused for the same reason.
//   N6  **the workload's only live instance**, in miniature: a `CallSeq` of
//       several calls to loop-bodied functions defined in the same TU. This is
//       `?supershuffle@@YAXPAD@Z`'s shape — the ONE function in the whole
//       878-TU workload the fence takes back, and the one the oracle grades
//       `fnbyte-differs` at base.
//
//   X1  **mechanism E's exemption** — a callee defined here that emits nothing.
//       Stays in class.
//   X2  **mechanism I's exemption** — a callee defined here that the port
//       lowers. Stays in class.
//   X3  **direct recursion**, which lands on X2's side rather than on a cell of
//       its own: the callee is the caller, the port lowers it, so the fence
//       yields. `WB_INLINE_FINDINGS.md` F5 measures that c2 never inlines a
//       directly recursive callee, so keeping the call is also right — but that
//       is a *coincidence of two rules agreeing* and NOT an adoption of F5, and
//       it is written here so nobody later reads the exemption as one.
//
// **What this file does NOT contain is a `__forceinline` cell.** F3/F4 of that
// document measure that `__forceinline` bypasses every size test and that
// `/Ob0` overrides even it; both are accept-side facts about c2 and neither
// changes what this fence does, because a `__forceinline` callee defined here is
// refused by the same clause as an ordinary one. A cell would grade nothing this
// file does not already grade, and it is named here so its absence is a
// decision.
//
// The positive half is `winlfence_opaque_callee.cpp`.
//
// Board rows #2220–#2226; `docs/rungs/2026-08-09-w-inlfence.md`.

struct S {
    char *b;
    void  m();
    void  n();
};

struct T {
    char *b;
    void  Split();
    float Ms();
    float SplitMs();
};

void wif_n_loop(char *p);

// N1 — a void tail call to a function this TU defines.
void wif_n_use_local(char *p) {
    wif_n_loop(p);
}
void wif_n_loop(char *p) {
    for (int i = 0; i < 64; ++i) {
        p[i] = (char)(p[i] * 3 + i);
    }
}

// N2 — the statement form of the sequence, both callees defined here.
void wif_n_seq(S *s) {
    s->m();
    s->n();
}
void S::m() {
    char *p = b;
    for (int i = 0; i < 64; ++i) {
        p[i] = (char)(p[i] * 3 + i);
    }
}
void S::n() {
    char *p = b;
    for (int i = 0; i < 64; ++i) {
        p[i] = (char)(p[i] * 5 + i);
    }
}

// N3 — the 444's class: the float value tail with the callees defined here.
float T::SplitMs() {
    Split();
    return Ms();
}
void T::Split() {
    char *p = b;
    for (int i = 0; i < 64; ++i) {
        p[i] = (char)(p[i] * 7 + i);
    }
}
float T::Ms() {
    return (float)b[0] * 3.5f;
}

// N4 — a callee far over the (300,308] static ceiling.
int wif_n_big(int a) {
    a += a * 3;
    a += a * 5;
    a += a * 7;
    a += a * 11;
    a += a * 13;
    a += a * 17;
    a += a * 19;
    a += a * 23;
    a += a * 29;
    a += a * 31;
    a += a * 37;
    a += a * 41;
    a += a * 43;
    a += a * 47;
    a += a * 53;
    a += a * 59;
    a += a * 61;
    a += a * 67;
    a += a * 71;
    a += a * 73;
    return a;
}
int wif_n_use_big(int a) {
    return wif_n_big(a);
}

// N5 — a loop-bodied callee over the loop class's (56,80] boundary.
void wif_n_use_loop(char *p) {
    wif_n_loop(p);
}

// N6 — `?supershuffle`'s shape: a sequence of calls to local loop bodies.
void wif_n_supershuffle(char *p) {
    wif_n_loop(p);
    wif_n_loop(p);
    wif_n_loop(p);
}

// X1 — mechanism E's exemption. `wif_n_empty` emits nothing; the fence yields.
void wif_n_empty() {
}
void wif_n_use_empty() {
    wif_n_empty();
}

// X2 — mechanism I's exemption. The port lowers `wif_n_leaf`; the fence yields.
int wif_n_leaf(int a) {
    return a + 1;
}
int wif_n_use_leaf(int a) {
    return wif_n_leaf(a);
}

// X3 — direct recursion. The callee is the caller and the port lowers it, so
// this lands on X2's side; see the header.
void wif_n_recurse() {
    wif_n_recurse();
}
