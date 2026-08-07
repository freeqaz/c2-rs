// **W-ALIGN negative cell — the alignment the writer CANNOT express, and the
// reader must keep refusing.**
//
// `__declspec(align(16))` spells the `.gl` tag **`CA`** (wide, width field
// `8A` = 16) and c2 gives the object ALIGN_16 — `Characteristics` nibble **5**.
// `coff::container::placement_align` models 1/2/4/8 and nothing else, so
// reading `CA` as an alignment would hand the writer a nibble it cannot honour.
//
// `align_of_type_tag` therefore stops at `88`. This cell exists so that stopping
// point is GRADED rather than merely commented: it must stay `NotImplemented`
// at every mode lane, and a later widening that adds `8A` without teaching
// `placement_align` about 16 turns this fixture from a refusal into a mismatch.
//
// The three gates before it all PASS — mark `81`, frame `00 02`, linkage `01`,
// size varint `10` = 16, attr `00`. The refusal is the alignment tag alone,
// which is the same shape board #1110 described for `C6` and the reason the
// grid had to contain a cell on the far side of the boundary rather than only
// cells inside it.

__declspec(align(16)) struct L { virtual void f(); L(const char* s, int r); int a; };
L gL("abc", 0);
