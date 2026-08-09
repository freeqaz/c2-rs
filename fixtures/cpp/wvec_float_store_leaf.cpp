// **Positive — lane `w-vec` (#2501).** THE CONTROL for the two `_neg` cells
// beside it, and the cell that settles half of a published price.
//
// `src/system/math/vec.cpp` is the last T1 ALL-EXACT-NO-MATCH TU in the
// workload (`docs/CEILING.md` §11.3): both functions its reference obj emits —
// `??0Vector3@@QAA@MMM@Z` and `??0Vector4@@QAA@MMMM@Z` — are `fnbyte-exact`
// against real c2, and the whole obj grades `vocab-gap`. `w-nc` priced what
// remained as **`_fltused` plus seven non-instruction sections**.
//
// **The `_fltused` half was already paid**, by lane `w-blockir`, and this file
// is the proof rather than the assertion. `??0Vector3`'s entire body is
//
//     d0230000  stfs 1, 0(3)      d0430004  stfs 2, 4(3)
//     d0630008  stfs 3, 8(3)      4e800020  blr
//
// which is this function's body, instruction for instruction, and this obj is
// **byte-exact** — 5 sections, 15 symbols, `_fltused` at [14] immediately after
// `?wvec_store3@@YAXPAMMMM@Z`'s own [13]. So the port already emits, exactly,
// the code and the TU-level float marker `vec.cpp` needs.
//
// What it does **not** emit is that obj with data sections composed into it,
// which is what `wvec_float_store_leaf_data_bss_neg.cpp` is for. Keeping the
// two apart is the point: with one file the reader cannot tell "the port has no
// `_fltused`" from "the port cannot place a `.data` beside a `.text` COMDAT",
// and the published price named the first when the live blocker is the second.

void wvec_store3(float *p, float a, float b, float c) {
    p[0] = a;
    p[1] = b;
    p[2] = c;
}
