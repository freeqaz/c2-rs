// W4b2-v (positive-parse rejection): a framed call with a two-literal post-op —
// `g(a) + 1 + 2`. The framed class is exactly one literal `+ k`; here a second
// `33 <int> 02 02` follows where the result-type must be, so the positive
// whole-body parse rejects. Reference compiles it; the port models a single
// `+ k` post-op only → NotImplemented. See docs/CODEGEN_PPC_MVP.md (W4b2-v).
int g(int);
int f(int a) { return g(a) + 1 + 2; }
