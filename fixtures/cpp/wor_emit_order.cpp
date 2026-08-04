// **W-ORDER — the `.text` EMISSION ORDER.** Board row X-d, and it was a live
// `Port=Mismatch` on master reachable from four lines of ordinary C++.
//
// The port emitted a TU's functions in `.ex` order, which is (usually) source
// order. c2 does not: it emits a function only once every function it
// **references and defines** has already been emitted, rescanning the `.ex`
// order to a fixpoint. Get it wrong and the obj still links, every relocation
// still resolves, and the `.text` offsets, the inter-function padding, the
// symbol table order and every `$M`/`$T` number are all shifted — six-plus
// wrong bytes that only a byte compare sees.
//
// **The edge is NOT in the obj.** In `zc`/`bd`/`dd` below, `??0D`'s only
// relocation is a `bl` to `??0B`, which is *undefined* here. Nothing anywhere
// in the object mentions `??1B` — no branch, no relocation, no symbol — and c2
// still emits `??1B` first, because `??0D`'s IL carries a `26` **unwind
// action** naming it. A planner built from the relocation list gets every other
// probe in `work/w-order/p/` right and this one wrong.
//
//     .ex order   z  ,  ??0D ,  ??1B
//     obj order   z  ,  ??1B ,  ??0D          <- the plain leaf does NOT move
//
// The leaf is the **stability control** and it is doing real work: it separates
// the fixpoint scan from "defer every caller to the end", which fits the first
// four probes just as well and predicts `??1B, ??0D, z`.
//
// ---- and the SECOND defect this file pins, at `/EHsc` ---------------------
//
// `c2_il::IlFunction::eh_bare` charges an empty base-delegating constructor one
// extra label-counter slot at `/EHsc`. It is **not** charged when the `26`
// unwind action's target is defined in this TU **with an empty body** — which
// is exactly `??1B` here. Measured seed-free and in-TU over nine probes
// (`work/w-order/p/h*.cpp`, table on `PortC2::label_lead_of`). Before that
// correction this file emitted `$M`/`$T` numbers one too high in all six
// `/EHsc` lanes of `scripts/lanes.txt` and was byte-exact in the other six —
// which is what the `/EHsc` axis is in that registry for.
//
// `wor_emit_order_led.cpp` is the separating fixture for the surcharge: same
// reordering, target NOT empty, slot charged.

int z(int a) { return a + 1; }

struct B { B(); ~B(); int x; };
struct D : B { D(); };

// Defined FIRST and emitted LAST: `??0D` waits for `??1B`.
D::D() {}

// Defined LAST and emitted FIRST. Its body is a bare `blr`, which is also what
// suppresses the `/EHsc` surcharge on `??0D` above.
B::~B() {}
