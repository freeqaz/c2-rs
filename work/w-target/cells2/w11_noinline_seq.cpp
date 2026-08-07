// GRID-W2 cell w11_noinline_seq — the same question on the Seq shape, which is
// the 634-function family SPLICE-0-PORT's `S3-seq-setup-frame-only` clause
// opened. If c2 obeys `noinline` here too, the exposure is not confined to
// `tail`.
struct B { B(); };
struct D { B b; __declspec(noinline) D(); };
D::D() {}

void ext_anchor();
void anchor() { ext_anchor(); }
