// **W-ORDER3 / board #1178 — the MIXED-linkage functionless `.bss` is a GRADED
// REFUSAL, and it is refused with the answer in hand, not for want of a cell.**
//
// A boundary with no cell on its far side is a boundary nothing checks. Lane
// `w-order3` widened `coff::data::emit_data_obj` to place a **static-only**
// `.bss` at slot `A` (before both `.XBLD$W` watermarks, see
// `wa16_bss_static_reloc.cpp`); this file is the next shape along and it stays
// `NotImplemented`.
//
// **What real c2 does here is measured, and it is not a symbol-ORDER question.**
// Both objects are in `.gl` order and at ascending addresses, `h`@0 and `g`@4 —
// but the EXTERNAL's symbol record is not written in the `.bss` group at all:
//
//     sec[3] .bss      <- slot `A`; the STATIC created the section
//     sec[4] .XBLD$W(C2)
//     sec[5] .XBLD$W(C1)
//     sec[6] .data
//
//     sym[ 5] .bss + aux      sym[ 7] h            val=0 sec=3 STATIC
//     sym[ 8] .XBLD$W + aux   sym[10] __C2_11886
//     sym[11] ?g@@3UA@@A      val=4 sec=3 EXTERNAL   <- OUTSIDE the group,
//                                                       at the slot-`B` position
//
// That reads exactly like the first-contributor model the slot rule is built on:
// the static materialises the section early, and the external's record is
// written later, where the extern pass runs — which is where the whole group
// would have gone had the external created the section.
//
// **It is still one obj.** `MAX_OBJECTS_PER_SECTION` is 2 (board #184), so every
// cell available here has n = 2, and at n = 2 that reading is indistinguishable
// from several orderings that would differ at n = 3. The shape also has **zero**
// instances on the 878-TU workload (`work/w-order3/census_order.py`: 0 of 871
// objs put a `.bss` before `.XBLD$W:C2` at all). So a guess buys nothing and
// costs the one thing the correctness rule forbids.
//
// A later lane that widens `emit_data_obj` to mixed linkage without teaching it
// the external's record position turns this file from a refusal into a
// **mismatch** in every mode lane — which is the difference between this file
// and a comment nobody runs.

struct A{int a;};
A g;
static A h;
A* p = &h;
