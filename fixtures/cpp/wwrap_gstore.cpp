// w-wordwrap — the file-scope-global store leaf, the ACCEPTED cell.
//
// `src/system/rndobj/wordwrap.cpp`'s `?WordWrap_SetOption@@YAXI@Z` verbatim:
// twelve bytes, `lis 11,0 ; stw 3,0(11) ; blr`, four relocations. The smallest
// unconverted body on the frontier at this lane's base (board #2625), published
// under `expr-jump` on a body with no jump in it (#2387, #1416).
//
// **This TU cannot reach `match` and is not expected to.** The object it stores
// to is a non-COMDAT `.bss`, which `coff::writer::emit_obj_multi` refuses BY
// NAME — its placement on a function-bearing TU is ungraded (see
// `c2_il::IlDataDef::uninitialized`). What IS graded here, against real c2's own
// obj, is the FUNCTION: `fnbyte-exact` compares the twelve bytes and all four
// relocation records, and reads **1** on this file where it reads 0 on every
// `wwrap_gstore_*_neg.cpp` beside it.

unsigned int g_uOption;

void WordWrap_SetOption(unsigned int option) { g_uOption = option; }
