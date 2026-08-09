// **w-bdnz — a COUNTED LOOP followed by a FRAMED function**: the TU shape a
// wrong label charge breaks on, and the shape **neither standing instrument can
// produce**.
//
// This is `whash_loop_then_framed.cpp`'s form, one class over, and it is here
// for the reason that one is: `scripts/expr_sweep.sh` generates single-function
// TUs and `scripts/mode_cross.sh` crosses that same corpus with the lane
// registry, so **neither can emit a two-function TU of mixed frame class** and
// both would grade a wrong label charge green.
//
// # What breaks without the gate — and this lane's reason is NOT the inherited
// one
//
// `coff::plan_labels` advances the compiler-label counter once per function in
// `.text` order and charges **1** for a leaf. `IlFunction::label_slots` returns
// `None` for this shape instead, so `IlBundle::functions` refuses this whole TU.
//
// The three existing loop classes return `None` because `w-loop` measured that
// *which* of a loop's `+1..+4` charges applies cannot be read off the emitted
// bytes — `do/while`, `for(;;)`+`break` and a backward `goto` emit identical
// words and charge +1, +3, +1. **That argument does not apply to this class**,
// and lane `w-bdnz` measured why rather than inheriting it
// (`work/w-bdnz/LABEL_LEAD.md`, w-json's counterfactual form over real
// `c2.dll`):
//
//     the first function's body          /O1 $M   lead     /Ox $M   lead
//     leaf-none, 0 locals                  2556     --       2550     --
//     straight line, 2 locals              2558     +2       2552     +2
//     THIS CLASS (`for`, s -= k)           2563     +7       2558     +8
//     the `while` spelling                 2563     +7       2558     +8
//     the `do/while` spelling              2562     +6       2556     +6
//     HashString's pointer walk            2564     +8       2559     +9
//     this class with `*=`                 2563     +7       2558     +8
//     this class with `unsigned`           2563     +7       2558     +8
//
// The `for` and `while` spellings emit **byte-identical text and charge
// identically**, and `do/while` — which charges differently — is not in the
// class at all, because c2 does not convert it. So the confound the other three
// classes cite is absent here. What replaces it is sharper:
//
// **THE CHARGE IS MODE-DEPENDENT — +7 at `/O1` and +8 at `/Ox` on the same
// source — and `label_slots` has no mode parameter.** This class accepts BOTH
// modes, so any `Some(k)` would be right at one and put `?z9`'s `$M`/`$M`/`$T`
// triple one low at the other: six wrong bytes in an obj that still links,
// board #263's shape.
//
// `docs/LABEL_COUNTER.md` §4.2.1's `for` row, read literally, predicts a lead of
// **+1**. It is **six low** at `/O1` for this class. w-json measured its §1.1
// surcharge two low for a back-edge class; this is the same finding at a wider
// margin, and it is why the commission said to measure rather than to quote.
//
// # What this fixture asserts
//
// `Port=NotImplemented` over the whole TU, at every lane, with `mismatch 0` —
// the port declines rather than emitting a `$M` it cannot justify. Its
// separating control is `wbdnz_ctr.cpp`, eleven of these loops with **no** framed
// function beside them, byte-exact at `/O1` and `/Ox`.
//
// **MUST-FAIL MUTATION, verified**: replacing that `None` with `Some(1)` turns
// this TU from `NotImplemented` into a live `mismatch` against real `c2.dll`
// while the control stays `match`.
int gz(int);
int p_sub(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int z9(int a) { return gz(a) + 7; }
