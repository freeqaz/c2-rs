// W4b2 (integer passthrough tail call): `return g(a)` — the argument is already
// in the incoming register (a→r3 = g's first arg reg), so the reference emits a
// bare 5-section leaf `b g` (REL24 at .text+0x0), byte-identical to the void
// tail call `void f(){g();}` but int-returning. The int analog of mvp_call.cpp.
// The port models it as an integer tail call with an empty arg-setup prefix →
// Port=Match. See docs/CODEGEN_PPC_MVP.md (int tail-call family).
int g(int);
int f(int a) { return g(a); }
