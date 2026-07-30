// **Negative** — an FP *arithmetic* leaf sharing a translation unit with a
// framed function. The FP **store** half of this pair is byte-exact and lives in
// `w28_fp_store_framed.cpp`; this is the part still refused, and why.
//
// A float leaf's label stride is 2 without pooled constants and 4 or 6 with
// them, and `IlFunction` does not carry the constant count — so `label_slots`
// reports it as **undetermined** rather than as a number that would be wrong
// for a leaf with a constant, and the TU-level gate refuses. That is a
// different uncertainty from the one the store leaf had: the store leaf's value
// was *wrong*, this one is *unknown*, and the port must not answer an unknown
// with a default.
//
// **This one is now measured, and it is still refused** — deliberately. The
// eleven-row table in `w28_fp_store_framed.cpp` includes two rows of FP
// *arithmetic* leaves and they follow the same rule as the stores: one slot
// each, plus one for the TU. So the leaf below is 2 and could be admitted. What
// admitting it needs is for `IlFunction` to carry whether a float leaf pooled a
// constant, which is the FP seam's record and not the framed side's to
// restructure inside a merge. Ranked as a handoff instead. The leaf *with* a
// pooled constant stays genuinely unknown — its `.rdata` COMDAT and symbol may
// take slots of their own, and no capture here has one ahead of a framed
// function.
//
// Both functions census **in class** — that is the per-function verdict and it
// is correct — while the TU as a whole is `Port=NotImplemented`. The refusal
// lives at the translation unit, which is where the label counter lives, so it
// adds nothing to `census_gate.rs`'s recorded residual: that instrument asks the
// *per-function* gate, and both functions pass it. The gap between the two
// grains is the known per-TU/per-function one `docs/GAPS.md` §6 records, not a
// new error term.

void g1();
void g2();

// The FP half: an arithmetic leaf. Measured at 2 slots like every other
// FP-touching function, but `IlFunction::label_slots` cannot tell it from a
// pooled-constant one, so it answers "undetermined" and the TU refuses.
float fp_arith(float a, float b) { return a * b; }

// The framed half, which is what makes the counter observable.
void seq2() { g1(); g2(); }
