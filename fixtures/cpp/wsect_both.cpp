// w-sect / board #174 — BOTH sections in one obj, six sections total, with the
// two data sections split around the C1 watermark:
//   .drectve .debug$S .XBLD$W(C2) .bss .XBLD$W(C1) .data
// The C1 watermark's own SectionNumber moves from 4 to 5 because of the `.bss`
// ahead of it, and the symbol table interleaves accordingly — the `.bss` group
// sits between `__C2_11886` and the C1 section symbol.
int b1;
int d1 = 1;
