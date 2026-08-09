// w-memfit — the mmioGetInfo ladder RE-DERIVED AT BASE, past where w-park
// stopped.  Board #401's method; measurement only, nothing here is shipped.
//
// w-park's ladder (work/w-park/cells/lad_getinfo.cpp) ran L0..L4 and recorded
// L3 — a three-slot call with a literal — as a GAP on `call-arg-lit-permuted`.
// **That rung is PAID at this base**: the same file re-censused here reads
// 4/5 in class, L0..L3 all `call-sequence-early-return`, and only L4 blocked.
// w-park shipped the `ArgSite` widening itself, so its own decline inherited
// a price that its own commit had already reduced.  Inherited prices have been
// wrong six times this week; this is the seventh.
//
// What L4 does NOT isolate is the `2C` conversion.  L4's arguments are `void*`
// formals passed to `memcpy`'s `void*` parameters, so no conversion is minted.
// `?mmioGetInfo`'s are `HMMIO` and `LPMMIOINFO` — w-memcpy §2 read a `2C` on
// each of the three arguments off the real obj's IL.  L5 and L6 below put the
// conversion on each side of the intrinsic so the two facts are separated.
//
//   L5   typed pointers, ORDINARY callee    -> is the `2C` alone in class?
//   L6   typed pointers, `memcpy`           -> `?mmioGetInfo`'s own shape
//
// census:  c2rs census lad2.cpp --flags-file ../../dc3-workload/flags.txt --cwd .

struct MMIOINFO_ { int a[18]; };      // 72 bytes, the size mmioGetInfo copies
struct HMMIO__ { int unused; };
typedef HMMIO__ *HMMIO_;
typedef MMIOINFO_ *LPMMIOINFO_;

extern "C" void *memcpy(void *, const void *, unsigned int);
void g3n(void *, const void *, unsigned int);

// L5 — L3 with TYPED pointer formals, so each argument is converted to
// `void*` / `const void*` at the call.  Ordinary callee: isolates the `2C`.
unsigned long L5(HMMIO_ a0, LPMMIOINFO_ a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g3n(a1, a0, 0x48);
    return 0;
}
