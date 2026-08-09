// w-fltret — THE MEMBER CALL IN THE SEQUENCE'S VALUE TAIL, positive cells.
//
// w-mcall shipped the statement half (`p->m(); p->n();` → `BodyShape::CallSeq`)
// and declined the value tail as its clause **D3**, filed *unsized* because no
// census key separated it. w-callprice priced the clause on the emitted column
// at **447 emitted over 13 constructs**, the highest-yielding clause inside
// `eat_member_stmt_call`, and named it recommendation **R2**
// (`docs/rungs/2026-08-09-w-callprice.md` §5.2, §7).
//
// Every cell here is a whole-TU byte-exact match against real `c2.dll`, at the
// fixture profile AND at the workload's `/O1` — the emitted words are the ones
// `CallSeq` already emits, because c2 emits the SAME instruction stream for the
// float body and the int body:
//
//     ; float v_float(O *o) { o->Poll(); return o->Level(); }
//       00010  7c7f1b78   mr    r31,r3
//       00014  48000001   bl    ?Poll@O@@QAAXXZ
//       00018  7fe3fb78   mr    r3,r31
//       0001c  48000001   bl    ?Level@O@@QAAMXZ
//       00020  38210060   addi  r1,r1,96
//
// (`work/w-fltret/probe/v1.cod`, c2's own `/FAsc` listing.) The only difference
// in the obj is the undefined external `_fltused`, which is why the float tail
// is `SeqTail::CallValueFp` and not `CallValue { add_k: 0 }` with a note.
//
//   F1  `float`  value tail, explicit pointer receiver — the shape of
//       `float Timer::SplitMs() { Split(); return Ms(); }`
//       (`src/system/os/Timer.h:137`, **434 emitted in 434 TUs**), written with
//       an explicit receiver.
//   F2  `double` value tail — the other real width. `lfd`-class result, and
//       still nothing emitted after the `bl`.
//   F3  `int` value tail — w-mcall's `wmcall_seq_neg.cpp` **N6**, now PAID.
//       Kept here so the cell that was a negative is a positive by name.
//   F4  THREE statements then the float tail — the loop runs, not just the
//       two-call special case.
//   F5  the float tail with an EXPLICIT ARGUMENT — Class B with **two** saved
//       GPRs (`r31` and `r30`), and the argument marshalling is the shipped one.
//   F6  the FREE-FUNCTION spelling of the same tail. It stops at
//       `eat_return_plumbing`'s `result-type` gate at base (census key
//       `result-type-0x41`, **810 bodies / 1 emitted** over the 878-TU
//       workload) and is admitted by the same arm.
//   F7  **the workload's own spelling** — a member function calling its own
//       methods, so the receiver is the IMPLICIT `this` and the formals list
//       has no `2D` run at all. This is `SplitMs` written the way `Timer.h`
//       writes it.
//   F8  the `int` tail with a literal POST-OP (`return s->g() + 3;` →
//       `addi r3,r3,3`). The post-op region and a receiver in slot 0 had never
//       been graded together — that is the exact wording of w-mcall's D3 — so
//       it is graded here.
//
// **F7 is the load-bearing cell** and F1 is its explicit-receiver control: the
// 439-emitted census key this rung was commissioned against is dominated by one
// implicit-`this` member function, and a fixture that only had F1 would be
// grading the shape nobody writes.
//
// Board rows #2080–#2087; `docs/rungs/2026-08-09-w-fltret.md`.

struct O {
    void   Poll();
    float  Level();
    double DLevel();
    int    ILevel();
    float  Fv(int a);
};

struct T {
    void  Split();
    float Ms();
    float SplitMs();
};

void  wfr_g1();
float wfr_gf();

// F1 — the float value tail, explicit receiver.
float wfr_float(O *o) {
    o->Poll();
    return o->Level();
}

// F2 — the double value tail.
double wfr_double(O *o) {
    o->Poll();
    return o->DLevel();
}

// F3 — the int value tail (w-mcall N6, paid).
int wfr_int(O *o) {
    o->Poll();
    return o->ILevel();
}

// F4 — three statements, then the float tail.
float wfr_float3(O *o) {
    o->Poll();
    o->Poll();
    return o->Level();
}

// F5 — the float tail with an argument: two saved GPRs.
float wfr_float_arg(O *o, int k) {
    o->Poll();
    return o->Fv(k);
}

// F6 — the free-function spelling.
float wfr_free_float() {
    wfr_g1();
    return wfr_gf();
}

// F7 — the workload's own spelling: implicit `this`.
float T::SplitMs() {
    Split();
    return Ms();
}

// F8 — the int tail with a literal post-op.
int wfr_int_postop(O *o) {
    o->Poll();
    return o->ILevel() + 3;
}
