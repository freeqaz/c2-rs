// W-UNW-1: a leaf AHEAD of a framed function — two facts in one TU.
//
// 1. The `bl` displacement. MSVC writes `disp = −(the branch's own .text
//    offset)`, and `framed_call_text` hardcoded the offset of the ONE framed
//    body it could ever emit: `4BFFFFF5` (disp −0xC). With `lf` in front, `f`
//    starts at 0x08 and its `bl` sits at 0x14, so c2 writes `4BFFFFED`. That
//    was a live wrong-bytes emit the instant the single-function gate came off,
//    and no fixture could have caught it while the gate was there.
// 2. The label stride of a leaf is exactly 1 counter slot, so `f`'s labels are
//    `$M(seed+1)`/`$M(seed+2)`/`$T(seed+3)` rather than `$M(seed)`.
int g(int);
int lf(int a) { return a + 1; }
int f(int a) { return g(a) + 1; }
