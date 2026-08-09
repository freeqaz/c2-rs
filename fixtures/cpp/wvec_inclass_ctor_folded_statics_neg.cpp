// **MUST REFUSE — lane `w-vec` (#2503).** `src/system/math/vec.cpp`'s shape
// verbatim, at 11 lines instead of 29 and with two static members instead of
// thirteen: an in-class float constructor and static members of the class's own
// type, one non-zero and one all-zero.
//
// The reference obj at `/O1` reproduces `vec.obj` exactly, minus the `.rdata`
// COMDAT the STL header drags in:
//
//     .drectve .debug$S .XBLD$W .XBLD$W .text .data .bss        (7 sections)
//     ??0WVecV3@@QAA@MMM@Z  (.text COMDAT, 16 B, d0230000 d0430004 d0630008 4e800020)
//     _fltused              (immediately after it, symbol [14])
//     ?sX@WVecV3@@2V1@A     (.data, 12 B, 3f 80 00 00 …)
//     ?sZero@WVecV3@@2V1@A  (.bss, 12 B)
//
// `??0Vector3@@QAA@MMM@Z`'s 16 bytes, `_fltused` in the same relative slot, and
// c1xx has **constant-folded** both initializers — there is no `??__E` dynamic
// initializer and no relocation anywhere in the obj.
//
// ## What it discriminates that the `_data_bss_neg` cell cannot
//
// That cell's `.gl` binds: 1 record, 1 segment, body in class, and the gate
// stops at `unclaimed-gl-symbol`. **This one never gets that far.** At `/O1`
// `IlBundle::decode_causes()` reads
//
//     segments 4 · records_gate 3 · records_wide 3 · bodies-out-of-class 2
//     downstream_evaluated FALSE · first = "bind-record-count-ne-segments"
//
// — four `.ex` bodies against one emitted COMDAT, so `Bindings::per_record`
// refuses before any body is looked at, and the post-binding gates have no
// answers on this TU at all. On `vec.cpp` itself the same axis reads **811
// segments against 36 records the gate's framing can see** (369 under the
// window-free one), with the walk stopping earlier still, at
// `gl-stop-26-introduced`.
//
// **The surplus is the emit set**, factor A, and it is why a TU whose every
// *emitted* function is byte-exact is not one repair from a match: the port
// parses all 811 bodies or refuses, 573 of them are outside the modeled class,
// and no order of repairs reaches the two that matter without an emit-set
// selection applied *before* the parse gate. `docs/CEILING.md` §11.4 item 6.

class WVecV3 {
public:
    float x, y, z;
    WVecV3() {}
    WVecV3(float a, float b, float c) : x(a), y(b), z(c) {}
    static WVecV3 sX;
    static WVecV3 sZero;
};

WVecV3 WVecV3::sX(1, 0, 0);
WVecV3 WVecV3::sZero(0, 0, 0);
