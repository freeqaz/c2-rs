// eclongframed — extern "C", long names, a FRAMED body (a non-tail call), to
// see whether the refusal is about leaves specifically.
extern "C" {
int callee_long_name_5(int, int, int, int, int);
int forward_long_framed(int a, int b, int c, int d, int e) { return callee_long_name_5(a, b, c, d, e) + 1; }
}
