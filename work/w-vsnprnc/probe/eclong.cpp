// eclong — extern "C", IDENTITY passthrough, names LONGER than the 8-byte COFF
// inline name field. Isolates the name length from the linkage.
extern "C" {
int callee_long_name_5(int, int, int, int, int);
int forward_long_name_5(int a, int b, int c, int d, int e) { return callee_long_name_5(a, b, c, d, e); }
}
