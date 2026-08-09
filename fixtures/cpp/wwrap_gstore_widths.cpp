// w-wordwrap — the store-opcode table, as three more ACCEPTED cells.
//
// GRID T (`work/w-wordwrap/probe/gtype.cpp`) compiled all sixteen scalar types
// and found four store opcodes and one shape: `lis 11,0` · `st? 3,0(11)` ·
// `blr`, twelve bytes every time. The width is the class's ONE free field, so
// the three widths the accepted cell does not carry are graded here rather than
// asserted in a table nobody compiled.
//
//     stb 3,0(11)  986b0000     sth 3,0(11)  b06b0000     std 3,0(11)  f86b0000
//
// `fnbyte-exact` reads **3** on this file.

unsigned char g_uc;
unsigned short g_us;
unsigned long long g_ull;

void SetUC(unsigned char x) { g_uc = x; }
void SetUS(unsigned short x) { g_us = x; }
void SetULL(unsigned long long x) { g_ull = x; }
