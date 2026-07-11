// W4b2-v (positive-parse rejection): two framed calls — `g(a) + g(a + 1)`. The
// literal lives inside the SECOND call's arguments, after the first call-end; a
// naive post-`55` literal search finds it and never notices the second `BD`
// CALL. The positive whole-body parse requires exactly ONE call: after the
// first call-end it expects the `33 <int> k 02` post-op but reaches a second
// `26 <tok> BD …` and rejects. Reference compiles it; the port models a single
// call only → NotImplemented. See docs/CODEGEN_PPC_MVP.md (W4b2-v).
int g(int);
int f(int a) { return g(a) + g(a + 1); }
