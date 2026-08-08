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
//   N6  `expr-call-in-expr-recv-load-whole`
//         the VALUE TAIL: `s->a(); return s->get();`. `SeqTail::CallValue`
//         marshals a receiver into slot 0 *and* a post-op region, and the two
//         have never been graded together. This cell keeps the key it has at
//         base, which is the rung's D7 working as designed: a member arm that
//         declines re-raises the block the body already reported, so a refusal
//         is never re-keyed.
//
// N1–N3 are receiver productions this rung declines; N4–N6 are sequence
// positions it declines. Both directions matter: a reader that admitted every
// receiver and one that admitted every position would each look like this file
// passing.
//
// Board rows #1960–#1963; `docs/rungs/2026-08-08-w-mcall.md`.

struct S {
    void a();
    void b();
    int get();
    float f();
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

// N6 — the value tail.
int wmcall_neg_value_tail(S *s) {
    s->a();
    return s->get();
}
