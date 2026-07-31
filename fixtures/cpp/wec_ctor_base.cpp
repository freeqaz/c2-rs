// WEC (positive) — the empty constructor that delegates to ONE base sub-object,
// beside the two things whose LABEL COUNTER it shares a translation unit with.
//
// One case per admitted row, and the TU itself is a case: the generated
// destructor and the framed call at the bottom are here so that this fixture,
// compiled with `/EHsc`, grades the `eh-bare` +1 label surcharge — the
// destructor consumes TWO counter slots there and one without it, and a
// following framed function's `$M`/`$T` numbers move with it. That is six wrong
// bytes in an obj that still links, and `work/WEC/live/t1.cpp` (this file's
// first two functions alone) was a live `mismatch` before the surcharge landed.
//
//   scripts/mode_lane.sh /O1                       -- no EH: the dtor is 1 slot
//   scripts/mode_lane.sh /O1 /EHsc                 -- EH:    the dtor is 2

struct B0 { B0(); B0(int); B0(int, int); int x; };
struct B1 { B1(); B1(int); ~B1(); int x; };
struct MemA { ~MemA(); int a; };

int g(int);

// --- the base HAS a destructor: `eh-bare`, and it pays the +1 at /EHsc -------
struct Ka : B1 { Ka(); };
Ka::Ka() {}

// --- the base has NO destructor: `eh-none`, byte-identical `.text`, +0 -------
struct Kb : B0 { Kb(); };
Kb::Kb() {}

// --- a formal of the constructor's own, unused: inert ------------------------
struct Kc : B1 { Kc(int a); };
Kc::Kc(int a) {}

// --- ONE forwarded argument: already in r4, no marshalling -------------------
struct Kd : B1 { Kd(int a); };
Kd::Kd(int a) : B1(a) {}

// --- TWO forwarded arguments: the identity over the argument slots, and the
//     argument region is in REVERSE source order -----------------------------
struct Ke : B0 { Ke(int a, int b); };
Ke::Ke(int a, int b) : B0(a, b) {}

// --- a forwarded argument beside an unused trailing formal ------------------
struct Kf : B1 { Kf(int a, int b); };
Kf::Kf(int a, int b) : B1(a) {}

// --- the generated empty destructor: `eh-bare` and a LEAF. Two counter slots
//     at /EHsc, one without. -------------------------------------------------
struct One { ~One(); MemA m; };
One::~One() {}

// --- and a framed function behind all of them, whose `$M`/`$T` numbers are
//     what every surcharge above is worth. ----------------------------------
int fr(int a) { return g(a) + 1; }
