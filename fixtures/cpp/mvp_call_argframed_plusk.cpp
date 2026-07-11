// W4b2-v (positive-parse rejection): in-argument arithmetic AND a framed
// post-op — `g(a + 1) + 1`. The `+1` inside the argument lands before the `55`
// call-end; a post-`55`-only search would mis-accept this as framed `g(a)+1`
// and silently drop the in-arg `+1`. The positive whole-body parse requires the
// call argument region to be EXACTLY the single passthrough LOAD, so the
// in-arg LIT+ADD makes parse_segment reject. Reference compiles it; the port
// models neither arg-setup nor this compound → NotImplemented. See
// docs/CODEGEN_PPC_MVP.md (W4b2-v).
int g(int);
int f(int a) { return g(a + 1) + 1; }
