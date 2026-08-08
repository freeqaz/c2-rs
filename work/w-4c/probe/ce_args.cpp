// CONFIRMATION 1 for lane `w-4c` — the CALL-END `4C` of a call WITH ARGUMENTS.
//
// Board **#1318** measured `0x4C` payload-free at 26,701 of 26,701 sites and
// declined to ship the width, because every one of those sites is a
// ZERO-ARGUMENT call: `26 <callee> BD <TYPE> <flags> <id> 4C`. The `4C` that
// closes a call carrying arguments is 2.46 M of the 3.5 M `BD` tokens and was
// not in that population at all.
//
// So this probe is that population, hand-built so the argument COUNT is the only
// thing that moves. All four callees return `int` and take `int`s, so the return
// TYPE, the flags byte and the argument annotation TYPE are byte-identical
// across the four rows and the argument region's LENGTH is the only variable.
// The token stream is `26 <g> BD 86 41 74 00 <id> (<arg> 55 86 41 74)* 4C`, and
// `4C`'s successor is read out by `read_ce.py`.
//
// cdecl only and `int` only, deliberately: it is the port's own accepted class,
// so this probe can reach `Port=Match` and the claim becomes *the accepting
// parser read this `4C` as one byte and the obj is byte-exact against real
// c2.dll*, not *a script decoded it that way*.

extern int g0();
extern int g1(int a);
extern int g2(int a, int b);
extern int g3(int a, int b, int c);

// 0 arguments — board #1318's own population, present as the control row.
int c0() { return g0(); }

// 1, 2, 3 arguments — the population #1318 excluded.
int c1(int x) { return g1(x); }
int c2(int x, int y) { return g2(x, y); }
int c3(int x, int y, int z) { return g3(x, y, z); }

// A literal argument: the `33 <TYPE> <payload>` operand stream, so the token
// before the closing `55 <TYPE> 4C` is not a load.
int c1lit() { return g1(7); }

// An argument that is itself an expression, so the argument region carries a
// `02` the walk must step before it can reach the `4C`.
int c1add(int x, int y) { return g1(x + y); }

// TWO calls in one statement: two `4C`s in one operand stream, the second of
// which is reached only by stepping over the first.
int c2seq(int x) { return g1(x) + g1(x); }

// A call as an ARGUMENT to a call: a nested `BD … 4C` strictly inside another
// call's argument region. This is the shape the workload walk excludes from
// anchor A by construction, and it is here so it is at least witnessed once.
int cnest(int x) { return g1(g1(x)); }
