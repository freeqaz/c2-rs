// GRID-T cell t03_seq_identity_tail — FIRES — the Seq shape: one call, an identity tail (SavedFormal), the 634-function family
struct B { B(); };
struct D { B b; D(); };
D::D() {}

void ext_anchor();
void anchor() { ext_anchor(); }
