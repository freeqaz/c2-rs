// w-inlfence — THE FENCE: a callee this TU also DEFINES.
//
// c2 may **inline** a callee whose body it has, and the port cannot. The
// measured cost of getting this wrong is on the board twice:
//
//   * `int f(int); int use(int a){return f(a);} int f(int a){return a+1;}` —
//     c2's `.text` is *two copies* of `addi r3,r3,1 ; blr` with **no
//     relocations**; the port's `b ?f` mismatched at file offset 8.
//   * `?SplitMs@Timer@@QAAMXZ` — 434 emitted functions in 434 workload TUs, the
//     reference body **31 words** against the port's 13, because
//     `Timer::Split()` and `Timer::Ms()` are `inline` members in the same
//     header (`docs/rungs/2026-08-09-w-fltret.md` §6, board #2082).
//
// Every cell here is `callee-defined-in-tu` — one clause, deliberately, because
// **the fence is one predicate and pretending it is seven would be counting the
// same refusal seven times.** What the cells separate is the *shape* the fence
// has to see it through, and three of them are shapes
// `docs/whitebox/WB_INLINE_FINDINGS.md` §7 says c2 would NOT have inlined:
// those are the price of the conservative direction and they are cells so the
// price is graded rather than asserted.
//
//   N1  a void TAIL CALL to a function defined below it — the offset-8
//       mismatch above, in its original form.
//   N2  the STATEMENT form of the member-call sequence (w-mcall's `CallSeq`)
//       with both member callees defined here.
//   N3  **the 444's own class** — the float VALUE TAIL (`SeqTail::CallValueFp`)
//       written the way `src/system/os/Timer.h:137` writes it, with `Split()`
//       and `Ms()` defined in the same TU. w-fltret's F7 is this cell with the
//       callees declared and not defined, and it is a whole-TU `match`; this is
//       the same source with two `{}` added, and it must refuse.
//   N4  DIRECT RECURSION. `WB_INLINE_FINDINGS.md` F5 measures that c2 never
//       inlines a directly recursive callee, so the port *could* keep this call
//       — and does not. Declined as **D2** in the rung: the accept side is not
//       offered, and a self-call's REL24 against its own COMDAT is a byte
//       question nothing here has captured.
//   N5  a callee far OVER the size ceiling. `WB_INLINE_FINDINGS.md` F1 measures
//       the static ceiling at `(300,308]` bytes of emitted `.text` at
//       favour-size, so c2 declines this one — and the port refuses anyway,
//       because the ceiling is a **bracket** and asking "how big is this callee"
//       means lowering it first, which the port cannot do for a body it does not
//       accept. Declined as **D3**.
//   N6  a LOOP-BODIED callee over the loop class's own `(56,80]` boundary
//       (F9, 56 cells) — the tighter ceiling, refused for the same reason, and
//       the shape of `?shuffle1`/`?shuffle3`…`?shuffle6` in
//       `src/keygen_xbox.cpp`, where c2 inlines the 60-byte one and calls the
//       other five.
//   N7  **the workload's only live instance**, in miniature: a `CallSeq` of
//       several calls to loop-bodied functions defined in the same TU. This is
//       `?supershuffle@@YAXPAD@Z`'s shape — the ONE function in the whole
//       878-TU workload the fence takes back, and the one the oracle grades
//       `fnbyte-differs` at base.
//
// **What this file does NOT contain is a `__forceinline` cell.** F3/F4 measure
// that `__forceinline` bypasses every size test and that `/Ob0` overrides even
// it; both are *accept-side* facts about c2 and neither changes what this fence
// does, because a `__forceinline` callee defined here is refused by the same
// clause as an ordinary one. A cell would grade nothing this file does not
// already grade, and it is named here so its absence is a decision.
//
// The positive half is `winlfence_opaque_callee.cpp`.
//
// Board rows #2220–#2226; `docs/rungs/2026-08-09-w-inlfence.md`.

struct S {
    void m();
    void n();
};

struct T {
    void  Split();
    float Ms();
    float SplitMs();
};

void wif_n_void();

// N1 — a void tail call to a function this TU defines.
void wif_n_use_local() {
    wif_n_void();
}
void wif_n_void() {
}

// N2 — the statement form of the sequence, both callees defined here.
void wif_n_seq(S *s) {
    s->m();
    s->n();
}
void S::m() {
}
void S::n() {
}

// N3 — the 444's class: the float value tail with the callees defined here.
float T::SplitMs() {
    Split();
    return Ms();
}
void  T::Split() {
}
float T::Ms() {
    return 0.0f;
}

// N4 — direct recursion. c2 never inlines it; the port refuses it anyway.
void wif_n_recurse() {
    wif_n_recurse();
}

// N5 — a callee far over the (300,308] static ceiling.
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

// N6 — a loop-bodied callee over the loop class's (56,80] boundary.
void wif_n_loop(char *p) {
    for (int i = 0; i < 64; ++i) {
        p[i] = (char)(p[i] * 3 + i);
    }
}
void wif_n_use_loop(char *p) {
    wif_n_loop(p);
}

// N7 — `?supershuffle`'s shape: a sequence of calls to local loop bodies.
void wif_n_supershuffle(char *p) {
    wif_n_loop(p);
    wif_n_loop(p);
    wif_n_loop(p);
}
