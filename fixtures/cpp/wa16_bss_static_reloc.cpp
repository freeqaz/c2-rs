// **W-ALIGN16 / board #1148 — a LIVE WRONG EMIT that was on master, found by a
// grid built for something else, and now a graded refusal.**
//
// This TU was `mismatch` against real c2 on an **unmodified tree**, at alignment
// 4. Not a gap, not a refusal — wrong bytes. Board **#232**'s shape, and #232's
// reason for being invisible: no fixture in the corpus could generate it, so
// every scan read `mismatch 0` over it.
//
// **What c2 does that `emit_data_obj` did not.** Rule S1 puts `.bss` *between*
// the two `.XBLD$W` watermarks, and that is right when the `.bss` object has
// EXTERNAL linkage. When it holds an internal-linkage object, c2 puts `.bss`
// **before both of them**:
//
//     extern:  .drectve .debug$S .XBLD$W  .bss     .XBLD$W .data     <- S1
//     static:  .drectve .debug$S .bss     .XBLD$W  .XBLD$W .data     <- c2
//
// **Why nobody had seen it.** `wsect_drop_static.cpp` records that an
// uninitialized *unreferenced* static is dropped by c2 entirely, and
// `wsect_data_linkage.cpp`'s header concludes from that: *"mixed linkage is
// unreachable in a `.bss` of a functionless TU"*. True of the cells that
// existed. The route around the drop is to **reference** the static — a `.data`
// initializer holding its address keeps it alive — and that is this file's third
// line. It is one line of C++ and it had never been written.
//
// **The same scope error hits Rule Y1.** `OBJ_DATA_BSS_SHAPE.md` §6.2's static
// and mixed-linkage `.bss` rows are real objs — from TUs **with functions**,
// which is what keeps *their* statics alive. `emit_data_obj` only ever runs on
// functionless TUs, so it was reading both S1 and Y1 outside every cell that
// fitted them. With a real functionless mixed-linkage `.bss`
// (`work/w-align16/diag/cells/D07_mixed_bss_reloc.cpp`) c2 emits the EXTERNAL
// `.bss` symbol *after the following section's group* and puts the static at
// offset 0 — neither Y1's order nor its walk. Y1's extern-only half is untouched
// and keeps its 89 real sections; whether its mixed row still holds for a TU
// with functions is open and this lane did not test it.
//
// `emit_data_obj` now refuses any `.bss` holding an internal-linkage object.
// **The fix is a refusal and not a reorder on purpose**: the correct order is a
// three-cell observation, and Rule S1 belongs to board #174 with its own grid.
//
// This cell is at alignment 4, so it grades the refusal **without** depending on
// one byte of #1120. `work/w-align16/cells/A11_static_align16.cpp` is the same
// shape at 16 and is how it was found.

struct A{int a;};
static A g;
A* p = &g;
