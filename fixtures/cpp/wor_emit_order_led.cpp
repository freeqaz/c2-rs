// **W-ORDER, the separating cell** — X-d's own reproducer
// (`work/w-cross/alarm/explicit.cpp`), which reorders exactly like
// `wor_emit_order.cpp` and pays the `/EHsc` label surcharge that one does not.
//
// Two facts have to be separated and one file cannot do it:
//
//   1. the emission ORDER — `??1M` before `??0D`, in both files;
//   2. the `/EHsc` eh-bare surcharge on `??0D` — **suppressed** in
//      `wor_emit_order.cpp`, **charged** here.
//
// The difference is the unwind target's *body*. There, `B::~B(){}` is a bare
// `blr`. Here `M` derives from `Bd`, so `M::~M(){}` is the delegating
// `b ??1Bd@@QAA@XZ` — defined in this TU, not empty, and the slot is charged
// (`work/w-order/p/hg_target_delegating.cpp`, seed-free: 7 slots between the
// anchor and `??0D` at `/EHsc` against 5 without `/EH`).
//
// A predicate of "the unwind target is defined here", which fits every probe
// that has an *empty* target, suppresses the slot in both files. It was written
// that way first and turned five byte-exact objs into mismatches; the `/EHsc`
// lanes caught it in one build.
//
// This is also the TU `docs/rungs/2026-08-04-w-cross-sep26.md` §3 left open:
// its `.gl` contains **no `0x26` byte at all**, so `d0d8a98` provably could not
// have caused it, and the sweep's generator had never written an out-of-line
// sibling destructor. `scripts/sweep.d/63-emit-order.py` now generates the
// whole family.

struct Bd { Bd(); ~Bd(); int b0; };
struct M : Bd { M(); ~M(); };
struct D : M { D(); };

D::D() {}
M::~M() {}
