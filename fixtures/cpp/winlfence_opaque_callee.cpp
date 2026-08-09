// w-inlfence — THE ACCEPT SIDE OF THE INLINE FENCE: an OPAQUE callee.
//
// `w-fltret` admitted 444 emitted functions through the member call's value
// tail and `fnbyte-exact` moved by **zero**, because c2 **inlines** both
// callees of `?SplitMs@Timer@@QAAMXZ` — its reference body is 31 words where
// the port emits 13 (`docs/rungs/2026-08-09-w-fltret.md` §6, board #2082). That
// rung's own finding is the sentence this file grades: *"the class is byte-exact
// exactly where the callees are OPAQUE, and on this workload they never are."*
//
// So this fixture is the accept case stated positively. Every callee below is
// **declared and not defined** — a true undefined external — which is the one
// categorical fact about c2's inliner that needs no threshold at all:
// **c2 cannot inline a body it does not have.** `docs/whitebox/
// WB_INLINE_FINDINGS.md` measures the decision on 320 obj cells and its §7
// refuses to offer the accept side of any *size* rule; nothing here uses one.
//
// Every cell is a whole-TU byte-exact match against real `c2.dll`.
//
//   F1  a void TAIL CALL to an external — the oldest call class in the port,
//       and the one whose exposure to this question nobody had written down.
//   F2  the STATEMENT form of the member-call sequence (`BodyShape::CallSeq`,
//       w-mcall) with two external member callees.
//   F3  the VALUE TAIL (`SeqTail::CallValueFp`, w-fltret) — the 444's own
//       class, with the callees opaque. This is the cell that says the fence
//       took nothing that was working.
//   F4  the same sequence with an explicit ARGUMENT: Class B, two saved GPRs.
//   F5  **the near-miss control.** This TU *defines* `wif_local_leaf`, and F5
//       calls the external `wif_local_leaf_x` — a name of which the defined one
//       is a strict PREFIX. The fence is a set membership on the whole mangled
//       name, and a prefix or substring test would refuse this cell; it is
//       graded here so that the day someone "optimizes" the test into a
//       `starts_with`, a fixture fails instead of a TU silently going quiet.
//   F6  a defined function that is never called at all, so the TU carries a
//       name in its defined set that no call edge touches. The set being
//       non-empty must not by itself refuse anything.
//
// The negative half is `winlfence_local_callee_neg.cpp`.
//
// Board rows #2220–#2226; `docs/rungs/2026-08-09-w-inlfence.md`.

struct O {
    void  Poll();
    void  Step();
    float Level();
    float Fv(int a);
};

void wif_ext_void();
int  wif_local_leaf_x(int a);

// F6 — defined here, called by nothing.
int wif_local_leaf(int a) {
    return a + 1;
}

// F1 — a void tail call to an external.
void wif_tail_opaque() {
    wif_ext_void();
}

// F2 — the statement form of the member-call sequence, external callees.
void wif_seq_opaque(O *o) {
    o->Poll();
    o->Step();
}

// F3 — the value tail, external callees: the 444's class with the callee opaque.
float wif_value_tail_opaque(O *o) {
    o->Poll();
    return o->Level();
}

// F4 — the value tail with an explicit argument.
float wif_value_tail_arg_opaque(O *o, int k) {
    o->Poll();
    return o->Fv(k);
}

// F5 — the near-miss: the callee's name has a name this TU defines as a PREFIX.
int wif_prefix_control(int a) {
    return wif_local_leaf_x(a);
}
