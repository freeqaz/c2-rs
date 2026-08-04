// **W-ORDER NEGATIVE** — the neighbour that must stay REFUSED, and the reason
// it also bounds what the emission-order planner can currently be graded on.
//
// `?f` tail-calls `?g` and `?g` is **defined in this TU**, so the ordering edge
// here is a real branch rather than an unwind action. c2 emits `?g, ?f`
// (`work/w-order/p/d2_plain_call_fwd.cpp`), the `.ex` order is `?f, ?g`, and
// `coff::plan_text_order` gets it right — but the TU never reaches the emitter:
// `c2_il::IlBundle::functions()` refuses a bundle whose callee is defined here,
// so this is `NotImplemented` at every lane.
//
// **That is worth stating rather than hiding**, because it is w-frame's row
// F-c applied to this rung: today, *every* ordering edge the port can actually
// reach is a `26` unwind action, and the planner's call-edge half — and its
// cycle refusal, which needs two functions naming each other — have **no
// coverage under the GRADED profile**. They are pinned by thirteen portable
// assertions in `coff/order.rs` and by nothing the oracle has ever seen. The
// moment `functions()` admits a locally-defined callee, this file's shape
// becomes live and the planner is what stands between it and a wrong emit.
//
// If this file ever censuses in class without `plan_text_order` being graded
// against real objs on the call-edge rows, that widening has outrun its
// evidence.

extern void h();

void g();

void f() { g(); }

void g() { h(); }
