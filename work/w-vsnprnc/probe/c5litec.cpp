// c5litec — the same shape, extern "C", with names longer than the 8-byte COFF
// inline name field (w-extdata's INLINE_NAME_MAX clause).
extern "C" {
int callee_long_name_5(int, int, int, int, int);
int forward_long_name_4(int a, int b, int c, int e) { return callee_long_name_5(a, b, c, 0, e); }
}
