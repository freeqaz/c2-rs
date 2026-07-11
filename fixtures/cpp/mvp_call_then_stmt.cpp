// W4b2-v (positive-parse rejection): a terminal-looking void call followed by
// a second statement. After the void call's `4C 4B` the positive whole-body
// parse requires only the return plumbing; here a `B9` LOAD (the `return a+1`
// statement) stands there instead, so parse_segment rejects. The old gate
// checked only that `4C 4B` followed the call and mis-emitted a bare `b g`,
// dropping the trailing computation. Reference compiles it; the port does not
// model a call sequenced before a returned expression → NotImplemented. See
// docs/CODEGEN_PPC_MVP.md (W4b2-v).
extern void g();
int f(int a) { g(); return a + 1; }
