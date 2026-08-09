// w-mcall — the MEMBER CALL IN STATEMENT-SEQUENCE POSITION, negative cells.
//
// Six cells, six DISTINCT census keys, one clause each. Probe-verified per cell
// against `work/w-mcall/probe/p4.cpp` and `p5.cpp` before the file was written:
// a `_neg` file whose cells share a key proves one clause and looks like six.
//
//   N1  `expr-call-in-expr-recv-object-whole`
//         a NAMED-OBJECT receiver. `gO.a()` pushes the object's *symbol*, so the
//         receiver is a relocation (`lis`/`addi`) and not a register move.
//         `mcall_tail::eat_receiver_this` is the ONE locator for that decision
//         and the sequence reader carries no second copy of it.
//
//   N2  `expr-call-in-expr-chained-then-plain-call-whole`
//         a CHAINED receiver — `l->Next()->Val()`. The receiver is the previous
//         call's result, which is `SeqCall::link_args`' regime and a different
//         marshalling (the explicit arguments start at slot 1).
//
//   N3  `body-0x67`
//         a VIRTUAL call. `67`/`9A` is a different opcode pair and a different
//         dispatch; nothing in this rung reads it.
//
//   N4  `expr-brfalse`
//         a GUARDED sequence (W10). That class is Class A only and hoists its
//         entry block; no obj in this repo grades it with a receiver in slot 0,
//         so `parse_call_sequence_from` excludes `guard`/`early` by name.
//
//   N5  `call-ret-fp:mid`
//         a discarded `float` result. The TU still has to carry `_fltused` —
//         `docs/GAPS.md` §6 instance #14, a live wrong-bytes emit when it was
//         found — and the member arm asks `CallRet::discarded` exactly where the
//         free-function arm does.
//
//   N6  `tail-argument-not-in-the-operand-vocabulary`
//         a member call whose ARGUMENT is a nested call — `s->take(t->get())`.
//         The argument operand vocabulary is the largest `prod` tag on this
//         whole family's emitted column (**8,909 emitted over 4,088 distinct
//         functions**, w-callprice §5), and it is a *lowering*, not an
//         admission: a call standing as an operand is w-value's class and
//         w-mcall's own decline **D1**.
//
//         **This slot used to hold the VALUE TAIL** — `s->a(); return
//         s->get();`, w-mcall's decline **D3**, filed unsized. Lane `w-fltret`
//         PAID it (`docs/rungs/2026-08-09-w-fltret.md`), the cell became
//         `call-sequence-value`, and **this file went on grading
//         `Port=NotImplemented` exactly as before** — a `_neg` fixture's graded
//         property is a whole-TU refusal, so it cannot see one of its own cells
//         being converted. The needle is re-taken here rather than deleted,
//         which is w-park's precedent: retire a cell only when it is FULLY
//         paid, and replace it so the file keeps six live declines.
//
// N1–N3 are receiver productions this rung declines; N4–N6 are sequence
// positions it declines. Both directions matter: a reader that admitted every
// receiver and one that admitted every position would each look like this file
// passing.
//
// Board rows #1960–#1963; `docs/rungs/2026-08-08-w-mcall.md`. N6 re-taken at
// board **#2085**.

struct S {
    void a();
    void b();
    int get();
    float f();
    void take(int v);
};
struct L {
    L *Next();
    void Val();
};
struct V {
    virtual void v();
};
extern S gO;
void wmcall_neg_free();

// N1 — a named-object receiver.
void wmcall_neg_object() {
    gO.a();
    gO.b();
}

// N2 — a chained receiver.
void wmcall_neg_chain(L *l) {
    l->Next()->Val();
    wmcall_neg_free();
}

// N3 — a virtual call.
void wmcall_neg_virtual(V *v) {
    v->v();
    v->v();
}

// N4 — a guarded sequence.
void wmcall_neg_guarded(S *s, int c) {
    if (c) {
        s->a();
    }
    s->b();
}

// N5 — a discarded floating-point result.
void wmcall_neg_fp(S *s) {
    s->f();
    s->a();
}

// N6 — a nested call in an argument slot (the slot the value tail used to hold;
// that clause is paid, see the header).
void wmcall_neg_argvocab(S *s, S *t) {
    s->take(t->get());
    s->b();
}
