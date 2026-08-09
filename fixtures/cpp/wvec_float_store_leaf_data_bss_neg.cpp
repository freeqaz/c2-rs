// **MUST REFUSE — lane `w-vec` (#2502).** `wvec_float_store_leaf.cpp` with
// **one thing added**: two file-scope objects, one initialized and one not.
//
// This is `src/system/math/vec.cpp`'s obligation with the 811-body emit-set
// question removed, so that the composition question is the *only* one left.
// The reference obj at the workload's `/O1` is **7 sections**:
//
//     .drectve .debug$S .XBLD$W .bss .XBLD$W .data .text
//     ?wvec_z@@3PAMA (.bss, 12 B)  ?wvec_g@@3PAMA (.data, 12 B)
//     ?wvec_store@@YAXPAMMMM@Z (.text COMDAT, the same 16 bytes)  _fltused
//
// Every one of those names is already in `coff::PORT_WRITER_SECTIONS`, so
// factor **C** — the section *vocabulary* — is satisfied and always was. What
// does not exist is a writer path that **composes** them: `emit_comdat_obj`'s
// `.data` is per-function and COMDAT and it refuses a data object on a float
// function outright; `emit_data_obj`'s whole class is "defines no functions";
// `emit_empty_obj` is the four-section shell. Three emitters, and the shape
// between them is unreachable.
//
// ## The refusal is upstream of all of that, and it is ONE named clause
//
// `IlBundle::decode_causes()` on this TU at `/O1` reads
//
//     segments 1 · records_gate 1 · bodies-out-of-class 0
//     downstream_evaluated true · first = "unclaimed-gl-symbol"
//
// — the binding succeeds, the body is in class, and the gate stops because
// `?wvec_g@@3PAMA` and `?wvec_z@@3PAMA` are `.gl` symbols no record claimed and
// no function accounts for. **That is the fence, and it is doing its job**: the
// port would emit four sections where c2 emits seven, and a refusal is strictly
// better than that obj.
//
// ## Why this cell and `wvec_inclass_ctor_folded_statics_neg.cpp` are BOTH here
//
// They stop at **different** causes — `unclaimed-gl-symbol` here,
// `bind-record-count-ne-segments` there — and neither implies the other. A
// single cell would have made `vec.cpp` look one repair away from converting
// when it is at least five.

void wvec_store(float *p, float a, float b, float c) {
    p[0] = a;
    p[1] = b;
    p[2] = c;
}

float wvec_g[3] = {1, 0, 0};
float wvec_z[3];
