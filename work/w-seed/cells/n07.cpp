// GRID-N n07 — CONTROL. AN EXTERNAL nothing-body: `da_ext` is declared here and
// defined in another TU, so this bundle cannot read its body at all.
//
// This is `elide.rs` condition 1 (`c22_extern_callee`, `w-inl0`'s m07) asked of
// the SEED rather than of the link. A seed is a strictly stronger claim than a
// link — it asserts unconditionally that a body emits nothing — so the same-TU
// condition has to be re-checked against it and not assumed to carry over.
//
// Registered: nothing is admitted, c2 keeps its REL24 for `?use` at BOTH flag
// settings, and the relocation count is printed. Without this cell "the port
// emitted no branch" and "nothing in this cell emitted anything" would be the
// same observation.
struct S { int a; };

void da_ext(S* p);

void use(S* p) { da_ext(p); }
