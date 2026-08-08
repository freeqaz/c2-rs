// w-bd CONFIRMATION 1 — the `BD` CALL token's payload, captured at THIS master.
//
// `docs/IL_CALL_GRAMMAR.md` §2.2 records the flags byte from an `e19.cpp` that
// is not in this tree. This is that probe rebuilt, so the width is witnessed by
// a capture taken now rather than inherited from a doc: three externals that
// differ ONLY in calling convention, with a byte-identical return type, so the
// flags byte is the only field that can move.
//
// The three return-TYPE widths (3, 4, 5 bytes) are exercised too, because the
// width rule under test is `<TYPE> <1 byte> <varint>` and a fixed-offset reader
// would agree with it at exactly one type width.
extern int cd(int);                 // __cdecl   — flags 0x00
extern int __stdcall sc(int);       // __stdcall — flags 0x00 (a no-op on PPC)
extern int __fastcall fc(int);      // __fastcall— flags 0x04
extern int va(const char *, ...);   // varargs   — flags 0x40

extern void v0();                   // void   return: 3-byte TYPE
extern void *p0();                  // void*  return: 4-byte TYPE

struct Wide { int a, b, c, d, e, f, g, h; };
extern Wide *w0();                  // a TU-created pointer type: 5-byte TYPE

int  f_cd(int a) { return cd(a); }
int  f_sc(int a) { return sc(a); }
int  f_fc(int a) { return fc(a); }
int  f_va(int a) { return va("x", a); }
void f_v0()      { v0(); }
void f_p0()      { p0(); }
void f_w0()      { w0(); }
