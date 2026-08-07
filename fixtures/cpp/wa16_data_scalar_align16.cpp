// **W-ALIGN16 / board #1120 — the object that is SMALLER than its own
// alignment, which is what makes "the tag is the size" untenable at 16.**
//
// `sizeof(g)` is **4** and c2 gives its `.bss` `Characteristics` nibble **5**
// (ALIGN_16). The `.gl` record spells tag `CA`, size varint `04`: the width
// field and the size field are two different fields carrying two different
// numbers in the same record, and only a reader that takes the alignment from
// the *tag* gets this obj right.
//
// `IL_TYPE_TAGS.md` §1 tabulates the tag under the heading **size**, which is
// true for scalars — where a type's size *is* its alignment — and is exactly
// the reading this cell kills. `w-align` killed it at 4 with a size-8 and a
// size-68 cell at tag `C6`; this is the same kill at 16 and in the other
// direction, size *below* the alignment rather than above it.
//
// Pairs with `wa16_data_align16.cpp` (size 16, alignment 16). Neither cell
// alone separates the two fields.

__declspec(align(16)) int g;
