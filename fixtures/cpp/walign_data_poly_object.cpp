// **W-ALIGN — board #1110's OWN probe cell, and the boundary it does not
// cross.**
//
// This is the two-line source #1110 was measured on: `?g@@3UA@@A`, a plain
// file-scope data object, refused by `align_of_type_tag` on tag `0xC6` with no
// RTTI in the *reader's* way. The arm reads it now — `gl_data_objects_ordered`
// on this TU went **1 of 12 records → 2 of 12** (12 = the sections c2 emits),
// the object joining the `$initializer$` that was the only one before.
//
// **And it still emits nothing, on purpose.** Defining the virtual function
// elsewhere is not enough to keep a vftable out: this TU's obj has 12 sections
// including four `.rdata$r` and the `??_R*` graph, which is board #1107's
// `.rdata$r` writer — DECLINED three times, priced at seven independent
// refusals of which zero are fully paid. The `.gl` reader was never the whole
// price and this fixture is what keeps that honest: a lane that "fixes" the
// remaining `.gl` gates and expects this cell to convert will find it does not.
//
// So this is a GRADED REFUSAL. It must stay `NotImplemented` at every mode lane
// until #1107 is actually paid. Its value is that the row it moves — the reader
// half — is now visible in `crates/c2-il/tests/in_init_probe.rs`'s `gl-data`
// line, where before it needed a throwaway spike (`w-rdata3` §4).
//
// `DATA_ATTR = 0xA0` (#1109) and the `00 04` read-only frame refuse the other
// ten records of this TU and are deliberately untouched.

struct A{virtual void f();int a;};
A g;
