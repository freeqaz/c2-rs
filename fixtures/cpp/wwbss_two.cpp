// w-wordwrap2 — TWO objects in the shared non-COMDAT `.bss`, the ACCEPTED cell
// at the measured bound (`MAX_OBJECTS_PER_SECTION` = 2, board #184).
//
// The whole class is here and nowhere else in one file, because the three rules
// this cell separates all coincide at n = 1 and `wwrap_gstore.cpp` is n = 1:
//
//   * the STORAGE walk is `.gl` record order, bumped by `placement_align`
//     (`max(t, 1 if n<2 else 4 if n<64 else 8)`) — Rules A1 and A3';
//   * the SYMBOL group is the REVERSE of that order — Rule Y1's external
//     clause. Cell `p2` of `work/w-wordwrap2/probe/grid_b.txt` separates it
//     from ascending address, from descending address and from declaration
//     order at once;
//   * the section's alignment nibble is the MAX over the objects of each one's
//     own `placement_align` — Rule B1. The `unsigned long long` is 8-aligned
//     and the `unsigned int` is 4-aligned, so this section's characteristics
//     are `0xC0400080` where `wwrap_gstore.cpp`'s are `0xC0300080`. A writer
//     that took the FIRST object's nibble, or the last one's, is right on one
//     of the two orders and wrong here.
//
// It is also the shape `src/system/rndobj/wordwrap.cpp` itself has — two eager
// external objects in one 588-byte `.bss`, `?g_LineBreakTable` at `+0x0` and
// `?g_uOption` at `+0x248`, symbols in the opposite permutation. Cell `p9` is
// that obj reproduced from two leaf functions; this fixture is the port graded
// against the same shape.

unsigned long long g_ll;
unsigned int g_i;

void SetLL(unsigned long long x) { g_ll = x; }
void SetI(unsigned int x) { g_i = x; }
