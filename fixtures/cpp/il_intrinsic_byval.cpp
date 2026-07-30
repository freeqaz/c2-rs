// **Negative** — the two `0x40` selectors that are still UNNAMED, ids
// **222 / 0xDE** and **223 / 0xDF**. They are the 5th/6th most common selector on
// the real dc3 workload (1758 sites each across Dir/App/Game; 2491 blocked
// functions attributed to `expr-intrinsic-0xDF`), and this fixture pins their
// *trigger* and their *literal* without claiming their semantics.
//
// They always occur as a nested pair, 222 inside 223's argument region — which is
// why only `0xDF` ever appears as a blocking bucket:
//
//   33 86 41 74 80 df 00 00 00 | 40 86 46 80 20         223 -> the class type
//     33 86 41 74 >04<  55 86 41 74                       sizeof, as an argument
//     33 86 41 74 80 de 00 00 00 | 40 86 46 80 20       222 -> the same type
//       9b 86 43 8e 20 <tok>  55 86 43 8f 20              a slot reference
//       33 86 41 74 >04<  55 86 41 74                     sizeof again
//     4C
//     55 86 46 80 20                                      222's result
//     26 <copy-ctor> … BD … 4C                            the copy construction
//     33 86 41 74 01  55 86 41 74
//   4C
//
// **Two facts, both pinned by this file (one construct varied at a time):**
//
// 1. **The literal is `sizeof(class)`.** `v_c4` (4 bytes) emits `>04<` in both
//    slots; `v_c12` (12 bytes) emits `>0c<`. Nothing else about the two bodies
//    differs.
// 2. **The trigger is a non-trivial COPY CONSTRUCTOR, not the destructor and not
//    `/EHsc`.** `v_ctor` (copy ctor, no dtor) produces the pair; `v_dtor` (dtor,
//    no copy ctor) does not, and neither does `v_pod`. Captured under this repo's
//    default fixture flags, which do **not** include `/EHsc` — so the earlier
//    reading that these were EH-unwind bookkeeping is wrong.
//
// The dc3 witness agrees: every 222/223 site in `Dir.cpp` wraps a 4-byte class
// (`Symbol`) whose copy is handed to a by-value parameter — e.g.
// `?SystemConfig@@YAPAVDataArray@@VSymbol@@00@Z`, `DataArray *SystemConfig(Symbol,
// Symbol, Symbol)` — with the `Symbol(const char *)` constructor called inside
// 223's argument region.
//
// **Still UNKNOWN, and named as such:** which of the two does what. Both return
// the same class type; 222 takes (slot, size) and 223 takes (size, 222's result,
// the copy construction, 1). "222 addresses the caller-owned slot and 223 commits
// the copy into it" fits every byte here, but so does the reverse, and nothing in
// this fixture distinguishes them — the emission is a whole calling sequence, not
// two separable pieces. **The fixture that would separate them** is one where the
// two nest asymmetrically: a by-value argument that is *itself* a by-value
// parameter being forwarded twice in one call
// (`void f(C4 a, C4 b); void g(C4 x) { f(x, x); }`), where a slot-addressing
// operation must appear twice and a commit operation once, or vice versa.
//
// docs/IL_CAST_CONVERT.md §1.5 read these as "plausibly pointer-to-member
// formation/adjustment". They are not: pointer-to-member formation uses no `0x40`
// at all (it is a `9b`/`27`/`5c`/`44` composition — see the `take`/`pmf_*` probes
// noted in docs/IL_INTRINSIC_CALL.md §5).
//
// **Harness note.** c1xx emits only 2 `4C 4F 11` body markers for the 5 functions
// here (against 5 function tails), so the census reports 2 bodies, not 5.
// `split_function_bodies` is LO-anchored, and these copy-construction bodies open
// on something else. That divergence is specific to this shape — on the real
// workload `Dir.cpp` has 5239 LO markers against 5243 tails (0.08 %) — but it does
// mean this fixture's *census* line undercounts; read the `.ex` with
// `--keep-il` to see all six selector sites.

struct C4 {
    int a;
    C4(const C4 &);
    ~C4();
};
struct C12 {
    int a, b, c;
    C12(const C12 &);
    ~C12();
};
struct CtorOnly {
    int a;
    CtorOnly(const CtorOnly &);
};
struct DtorOnly {
    int a;
    ~DtorOnly();
};
struct Pod4 {
    int a;
};

void t4(C4);
void t12(C12);
void tc(CtorOnly);
void td(DtorOnly);
void tp(Pod4);

void v_c4(C4 x) { t4(x); }        // 223/222, literal 04
void v_c12(C12 x) { t12(x); }     // 223/222, literal 0c
void v_ctor(CtorOnly x) { tc(x); }// 223/222 — copy ctor alone is enough
void v_dtor(DtorOnly x) { td(x); }// no pair — a dtor alone is not the trigger
void v_pod(Pod4 x) { tp(x); }     // no pair
