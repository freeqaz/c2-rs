// W4b2-i (honest rejection): `return g(a + 1)` is a tail call with ARGUMENT
// setup — the `+1` lives INSIDE the argument, before the `55` call-end marker,
// NOT as a framed post-op. It must NOT be mistaken for framed `g(a) + 1`. The
// reference compiles it (5-section tail call `addi r3,r3,1 ; b g`); the port
// does not model arg-setup codegen (rung W4b2-iv) → NotImplemented, never a
// mis-emitted framed obj. Anchoring proof: parse_framed_call searches only
// AFTER the `55` call-end marker. See docs/CODEGEN_PPC_MVP.md (W4b2-i).
int g(int);
int f(int a) { return g(a + 1); }
