// **W-ALIGN16 / board #1120 — the SECOND consumer of the promotion table.**
//
// `coff::dyninit::align_nibble(16, 16)` = **5**, reached through `dyninit_tu`
// rather than `data_tu`. `w-align` §11 correction 2 is the reason this file
// exists beside `wa16_data_align16.cpp`: #1110 priced ALIGN_16 through
// `emit_data_obj` alone, and a lane that only tested that path would have
// shipped `align_nibble`'s 16 arm ungraded.
//
// The class is **not polymorphic** — a user constructor is what makes this a
// dynamic-initializer TU, not a vtable. `walign_dyninit_align16.cpp` is the
// polymorphic form of the same cell and both convert, which is what says the
// alignment arm and the vtable are independent here.
//
// At `/Ox` and `/Od` this is `codegen-gap`, not a match, and that is correct
// rather than a regression: neither profile implies `/GF`, so `"abc"` is a
// non-COMDAT `$SG<n>` `.rdata` placed before `.text` and `emit_dyninit_obj`
// declines to place it — the same reason `wr1c_dyninit_extern.cpp` refuses
// there. It converts at the workload's own `/GR /O1 /Oi /EHsc` and at `/O2`.

__declspec(align(16)) struct L{L(const char* s,int r);int a;};
L gL("abc", 0);
