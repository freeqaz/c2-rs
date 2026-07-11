// W4b2-vi (identity-fold integer tail call): `return g(a) + 0`. The `+0` reaches
// c2's IL as a real post-op literal (`33 86 41 74 00 02`, LIT 0 + ADD), but the
// optimizer folds it away — the reference obj is byte-identical to `return g(a)`
// (5-section leaf `b g`, REL24 at .text+0x0), NOT a framed 6-section `.pdata`
// obj. The positive parser recognizes a net-identity post-op as an integer tail
// call (`g(a)+0 == g(a)`) and routes it to the leaf path → Port=Match. This
// closes the W4b2-vi mis-emit leak (a `FramedCall{add_k:0}` would have emitted a
// frame the reference elides). See docs/CODEGEN_PPC_MVP.md (int tail-call family).
int g(int);
int f(int a) { return g(a) + 0; }
