// **W-ALIGN16 / board #1120 — the cell #1110 priced and `w-align` left on the
// table: ALIGN_16, taken.**
//
// `__declspec(align(16))` spells the `.gl` tag **`CA`** (wide, width field
// `8A` = 16) and c2 gives the object `Characteristics` nibble **5**. `w-align`
// measured that and REFUSED it, because `coff::container::placement_align`
// modelled 1/2/4/8. It models 16 now, in all three of the functions that share
// that promotion table — `placement_align`, `align_nibble` and
// `data::section_nibble` — plus `bump_layout`, which rounds a `.bss` cursor to
// it with no textual change at all.
//
// This is the `data_tu` consumer. `wa16_dyninit_plain_align16.cpp` is the other
// one; `w-align` §5 correction 2 is why both are here rather than one.
//
// Size 16 and alignment 16 coincide in this cell **on purpose it is the CONTROL
// half of a pair**: `wa16_data_scalar_align16.cpp` is size 4 at alignment 16, so
// the two together say the tag is the alignment and not the size. A grid that
// only ever varies them together cannot tell those apart, and
// `IL_TYPE_TAGS.md` §1 tabulates this very field under the heading "size".

__declspec(align(16)) struct A{int a;};
A g;
