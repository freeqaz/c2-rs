// W4b2-v (positive-parse rejection): two terminal-looking void calls. Only the
// FIRST `4C 4B` void call-end is where a bare tail call ends; a second call
// stands where the return plumbing must be, so the positive whole-body parse
// (c2_il::func::parse_segment) reaches a second `26 <tok> BD …` and rejects.
// The old neighborhood gate checked only around the first call and mis-emitted
// a single `b g`, silently dropping the second call. The reference compiles it
// fine; the port has no model for two sequenced calls → NotImplemented, never a
// mis-emit. See docs/CODEGEN_PPC_MVP.md (W4b2-v).
extern void g();
void f() { g(); g(); }
