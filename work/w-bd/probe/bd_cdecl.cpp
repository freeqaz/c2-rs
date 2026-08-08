// The cdecl-only half of `bd_cc.cpp`, so the differential can return
// `Port=Match` rather than `NotImplemented`: the port's own class gate refuses
// a non-zero flags byte by design (`control_flow.rs`'s `cf-call-fn-type-id`
// arm), which is itself part of what this rung witnesses.
extern void g1();
void f() { g1(); }
