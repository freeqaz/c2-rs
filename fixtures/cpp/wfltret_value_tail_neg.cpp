// w-fltret — the value tail's BOUNDARY, negative cells.
//
// Nine cells. Eight distinct clause keys, and the one PAIR that shares a key is
// named as a pair rather than counted twice — w-bdnz found two `_neg` cells that
// fired on an earlier clause than the one they named, and a confounded cell
// passes the fixture gate exactly like a correct one (the graded property,
// whole-TU `Port=NotImplemented`, holds either way).
//
//   N1  a NARROWING conversion on the returned real — `float f(){ …; return
//       o->D(); }`. c2 emits `frsp fr1,fr1` for it (measured,
//       `work/w-fltret/probe/v3.cod`), so admitting it would be **wrong bytes,
//       not a gap**. The IL wears an explicit `2C <TYPE> 00` between the `4C`
//       and the `41`, and `eat_member_value_call` requires the `41` to stand
//       immediately after the `4C`.
//
//   N2  the WIDENING direction of the same conversion — `double f(){ …; return
//       o->F(); }`. **It shares N1's clause and N1's census key**, and it is
//       here anyway because it is the direction that costs c2 **nothing**: the
//       fence that refuses N1 cannot tell the two apart without a width model
//       this rung does not build, so decline **D6** gives up a free conversion
//       and this cell is the record of exactly which one.
//
//   N3  an FP POST-OP — `return o->F() + 1.0f;`. Not `addi`: c2 emits
//       `lfs fr0,__real@3f800000(r11)` from the `.rdata` FP pool and `fadds`.
//       `SeqTail::CallValueFp` has no `add_k` field, which is the structural
//       half of this fence.
//
//   N4  a DISCARDED `float` result. The TU still owes `_fltused` —
//       `docs/GAPS.md` §6 instance #14 — and `CallRet::discarded` still refuses
//       it. w-mcall's N5 one file over is the same clause; this rung measured
//       the key at **ZERO bodies and ZERO emitted over the whole 878-TU
//       workload** and declines it as a fence class with no cases rather than
//       as a population.
//
//   N5  a CHAINED receiver in the value tail. `mcall_chain`'s route, not
//       `CallSeq`'s — and that route carries **all 1,472 bodies / 105 emitted**
//       of `expr-call-in-expr-chained-then-type-real-whole`, which
//       w-callprice's R2 summed into its 544 without separating. Decline **D5**.
//
//   N6  a NAMED-OBJECT receiver in the value tail. `gO.get()` pushes the
//       object's *symbol*, and w-callprice #2026 read off c2's own listing that
//       an address-taken stack local wears the identical `26 <sym>` a
//       relocation does: admitting it through `IlOp::SymAddr` emits a
//       relocation where c2 emits a frame offset. `seq_call_arg_slots`' blanket
//       refusal is KEPT.
//
//   N7  a VIRTUAL call in the value tail — `67`/`9A`, a different opcode pair
//       and a different dispatch.
//
//   N8  a GUARDED sequence (W10) with a float value tail. That class is Class A
//       only and hoists its entry block, so `parse_call_sequence_from` excludes
//       `guard`/`early` by name and this rung did not change that.
//
//   N9  a value tail whose receiver is a FIELD — `p->q.m()`. A different
//       receiver production with its own lowering, declined by
//       `mcall_tail::eat_receiver_this`, which is the ONE locator for that
//       decision and of which this reader carries no second copy.
//
// N1–N3 are the RESULT's boundary, N4 is the obligation this rung does not pay,
// N5–N7 and N9 are receiver productions, N8 is a sequence position. Both
// directions matter: a reader that admitted every result form and one that
// admitted every receiver would each look like this file passing.
//
// Board rows #2080–#2087; `docs/rungs/2026-08-09-w-fltret.md`.

struct O {
    void   Poll();
    float  F();
    double D();
    int    I();
};
struct Q {
    float get();
};
struct P {
    Q     q;
    P    *Next();
    float Val();
    void  Poll();
};
struct V {
    virtual float v();
    void          Poll();
};
extern O gO;

// N1 — a narrowing conversion on the returned real.
float wfr_neg_narrow(O *o) {
    o->Poll();
    return o->D();
}

// N2 — the widening direction; shares N1's clause and N1's key.
double wfr_neg_widen(O *o) {
    o->Poll();
    return o->F();
}

// N3 — an FP post-op.
float wfr_neg_postop(O *o) {
    o->Poll();
    return o->F() + 1.0f;
}

// N4 — a discarded float result.
void wfr_neg_discard(O *o) {
    o->F();
    o->Poll();
}

// N5 — a chained receiver in the value tail.
float wfr_neg_chain(P *p) {
    p->Poll();
    return p->Next()->Val();
}

// N6 — a named-object receiver in the value tail.
float wfr_neg_object(O *o) {
    o->Poll();
    return gO.F();
}

// N7 — a virtual call in the value tail.
float wfr_neg_virtual(V *v) {
    v->Poll();
    return v->v();
}

// N8 — a guarded sequence with a float value tail.
float wfr_neg_guarded(O *o, int c) {
    if (c) {
        o->Poll();
    }
    return o->F();
}

// N9 — a field receiver in the value tail.
float wfr_neg_field(P *p) {
    p->Poll();
    return p->q.get();
}
