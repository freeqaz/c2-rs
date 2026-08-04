// W11 probes: the guarded EARLY RETURN in a framed body.
// Every frontier TU with a framed multi-block function has this shape.
void v0();
void v1();
void v2();
void g1(void*);

// e0 — one guard, one early return, one trailing call, int result
int e0(int a) { if (a) return 5; v0(); return 0; }
// e1 — pointer scrutinee (mmioGetInfo's compare form)
int e1(void* p) { if (!p) return 5; v0(); return 0; }
// e2 — two chained guards on two formals (mmioGetInfo's exact shape)
int e2(void* p, void* q) { if (!p) return 5; if (!q) return 11; v0(); return 0; }
// e3 — the guarded arm returns, the fallthrough returns a different literal
int e3(int a) { if (a) return 5; v0(); v1(); return 0; }
// e4 — void result
void e4(int a) { if (a) return; v0(); v1(); }
// e5 — guard whose arm has a call then a return
int e5(int a) { if (a) { v1(); return 5; } v0(); return 0; }
// e6 — three trailing calls after the guard
int e6(int a) { if (a) return 5; v0(); v1(); v2(); return 0; }
// e7 — the guarded return is the LAST thing (no trailing call)
int e7(int a) { v0(); if (a) return 5; return 0; }
