// w-fnbyte probe 2 — the same signature reached through the plainest shape the
// port accepts: a void call whose callee is DEFINED IN THIS TU and empty.
// The port's tail-call class emits `b ?g@@YAXXZ`; c2 at /O1 (/Ob2) may inline.
void g() {}
void f() { g(); }
